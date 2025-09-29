use eframe::egui;
use crate::engine::WebPage;
use crate::networking::HttpResponse;

pub struct BrowserTab {
    pub title: String,
    pub url: String,
    pub web_page: Option<WebPage>,
    pub loading: bool,
    pub error: Option<String>,
    pub history: Vec<String>,
    pub history_index: usize,
    pub redirects_followed: usize,
    // Track current response for cleanup of temporary files
    current_response: Option<HttpResponse>,
}

impl BrowserTab {
    pub fn new(title: String) -> Self {
        Self {
            title,
            url: "about:home".to_string(),
            web_page: None,
            loading: false,
            error: None,
            history: vec!["about:home".to_string()],
            history_index: 0,
            redirects_followed: 0,
            current_response: None,
        }
    }
    
    pub fn navigate_to(&mut self, url: String) -> bool {
        self.url = url.clone();
        if !url.starts_with("about:") && !self.history.is_empty() && self.history[self.history_index] == url {
            return false; // Already at this URL
        }
        self.history.push(url);
        self.history_index = self.history.len() - 1;
        
        self.load_page()
    }
    
    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }
    
    pub fn can_go_forward(&self) -> bool {
        self.history_index < self.history.len() - 1
    }
    
    pub fn go_back(&mut self) -> bool {
        if self.can_go_back() {
            self.history_index -= 1;
            self.url = self.history[self.history_index].clone();
            return self.load_page();
        }
        false
    }
    
    pub fn go_forward(&mut self) -> bool {
        if self.can_go_forward() {
            self.history_index += 1;
            self.url = self.history[self.history_index].clone();
            return self.load_page();
        }
        false
    }
    
    pub fn reload(&mut self) -> bool {
        self.load_page()
    }
    
    fn load_page(&mut self) -> bool {
        // Clean up any existing temporary files before loading new content
        self.cleanup_temp_files();
        
        self.loading = true;
        self.error = None;
        self.redirects_followed = 0;
        
        // Handle special URLs
        match self.url.as_str() {
            "about:home" => {
                self.title = "NeonSearch Home".to_string();
                self.web_page = Some(WebPage::create_home_page());
                self.loading = false;
                false // No network request needed
            }
            "about:blank" => {
                self.title = "Blank Page".to_string();
                self.web_page = Some(WebPage::create_blank_page());
                self.loading = false;
                false // No network request needed
            }
            _ => {
                // Need to fetch the webpage
                self.title = format!("Loading {}", self.url);
                self.web_page = Some(WebPage::create_loading_page(&self.url));
                true // Network request needed
            }
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) -> bool {
        if self.loading {
            ui.centered_and_justified(|ui| {
                ui.spinner();
                ui.label("Loading...");
            });
            return false;
        }
        
        if let Some(error) = &self.error {
            let mut retry_clicked = false;
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new("❌ Error").size(20.0).color(egui::Color32::RED));
                ui.label(format!("URL: {}", self.url));
                ui.separator();
                ui.label(egui::RichText::new(error).color(egui::Color32::LIGHT_RED));
                ui.separator();
                ui.label("Try:");
                ui.label("• Check your internet connection");
                ui.label("• Make sure the URL is correct");
                ui.label("• Try adding https:// prefix");
                if ui.button("🔄 Retry").clicked() {
                    retry_clicked = true;
                }
            });
            if retry_clicked {
                self.error = None;
                return self.reload(); // Return true if network request needed
            }
            return false;
        }
        
        if let Some(web_page) = &self.web_page {
            web_page.render(ui);
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No content to display");
            });
        }
        false
    }
    
    pub fn handle_network_response(&mut self, result: Result<HttpResponse, String>) {
        self.loading = false;
        
        match result {
            Ok(response) => {
                // Store the response for potential cleanup later
                self.current_response = Some(response.clone());
                if response.is_redirect() {
                    if let Some(location) = response.get_header("Location").cloned()
                        .or_else(|| response.get_header("location").cloned()) {
                        // Basic absolute/relative handling
                        let new_url = if location.starts_with("http://") || location.starts_with("https://") {
                            location
                        } else {
                            // Resolve relative (very naive)
                            if let Ok(base) = reqwest::Url::parse(&self.url) {
                                if let Ok(joined) = base.join(&location) { joined.to_string() } else { location }
                            } else { location }
                        };
                        if self.redirects_followed < 5 { // cap redirects
                            self.redirects_followed += 1;
                            self.url = new_url.clone();
                            // Prepare loading placeholder; actual fetch triggered externally after this returns
                            self.web_page = Some(WebPage::create_loading_page(&new_url));
                            self.title = format!("Redirecting {}", new_url);
                            // Indicate to caller they should refetch by setting error? Instead log.
                            log::info!("Following redirect to {}", new_url);
                        } else {
                            self.error = Some("Too many redirects".to_string());
                            self.web_page = Some(WebPage::create_error_page(&self.url, "Too many redirects"));
                        }
                    } else {
                        self.error = Some("Redirect with no Location header".to_string());
                        self.web_page = Some(WebPage::create_error_page(&self.url, "Redirect without location"));
                    }
                } else if response.is_success() {
                    // Check raw body size first to avoid UI blocking
                    let is_large_raw = response.body.len() > 50_000; // 50KB threshold for raw content (Google.com is ~71KB)
                    
                    // For large raw content, get decompressed content but use simpler rendering
                    let (html_result, use_simple_rendering) = if is_large_raw {
                        println!("Large raw content detected ({} bytes), using simplified rendering", response.body.len());
                        // Get the full decompressed content but we'll render it simply
                        match response.body_as_string() {
                            Ok(full_html) => {
                                // Take a reasonable preview of the decompressed HTML
                                let preview_size = 25_000; // 25KB of decompressed HTML
                                let html_preview = if full_html.len() > preview_size {
                                    format!("{}...\n\n[Content truncated - showing {} of {} characters for performance]", 
                                           full_html.chars().take(preview_size).collect::<String>(),
                                           preview_size,
                                           full_html.len())
                                } else {
                                    full_html
                                };
                                (Ok(html_preview), true)
                            },
                            Err(e) => (Err(e), true)
                        }
                    } else if response.is_large_content() {
                        println!("Large content detected, using preview mode");
                        (response.body_as_preview(100_000), false) // 100KB preview for large sites
                    } else {
                        (response.body_as_string(), false)
                    };
                    
                    match html_result {
                        Ok(html) => {
                            // For large content, create a simpler page to avoid HTML parsing overhead
                            let mut page = if use_simple_rendering {
                                WebPage::create_simple_text_page(&html, &self.url)
                            } else {
                                WebPage::from_html(&html, None)
                            };
                            
                            // Add notification for truncated content
                            if (use_simple_rendering || response.is_large_content()) && html.contains("[Content truncated") {
                                let notification = format!(
                                    r#"<div style="background: #ff6b35; color: white; padding: 10px; margin: 10px 0; border-radius: 8px;">
                                        <strong>⚠️ Large Site Detected</strong><br>
                                        This is a large site. Content has been truncated for performance. 
                                        Full content size: {:.1} MB<br>
                                        <small>Using simplified rendering for better performance.</small>
                                    </div>"#,
                                    response.body.len() as f64 / 1_048_576.0
                                );
                                
                                if let Some(existing_html) = &page.raw_html {
                                    page.raw_html = Some(format!("{}{}", notification, existing_html));
                                } else {
                                    page.raw_html = Some(format!("{}{}", notification, html));
                                }
                            }
                            
                            self.title = page.extracted_title.clone().unwrap_or_else(|| self.url.clone());
                            self.web_page = Some(page);
                            self.error = None;
                        }
                        Err(e) => {
                            self.error = Some(format!("Failed to parse response: {}", e));
                            self.web_page = Some(WebPage::create_error_page(&self.url, &e.to_string()));
                        }
                    }
                } else {
                    let error_msg = format!("HTTP {}", response.status_code);
                    self.error = Some(error_msg.clone());
                    self.web_page = Some(WebPage::create_error_page(&self.url, &error_msg));
                }
            }
            Err(e) => {
                self.error = Some(e.clone());
                self.web_page = Some(WebPage::create_error_page(&self.url, &e));
            }
        }
    }

    /// Clean up temporary files associated with the current page
    pub fn cleanup_temp_files(&mut self) {
        if let Some(response) = &self.current_response {
            if let Err(e) = response.cleanup_temp_file() {
                eprintln!("Failed to cleanup temporary file: {}", e);
            }
        }
        self.current_response = None;
    }

    /// Get information about current temporary file usage
    pub fn temp_file_info(&self) -> Option<String> {
        if let Some(response) = &self.current_response {
            if let Some(temp_file) = response.get_temp_file() {
                return Some(format!(
                    "Using temp file: {} ({} bytes)", 
                    temp_file.path.display(), 
                    temp_file.size
                ));
            }
        }
        None
    }
}

impl Drop for BrowserTab {
    fn drop(&mut self) {
        // Ensure cleanup on drop
        self.cleanup_temp_files();
    }
}