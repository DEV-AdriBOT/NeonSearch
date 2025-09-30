use eframe::egui;
use crate::engine::WebPage;
use crate::networking::HttpResponse;
use crate::ui::{NeonTheme, NeonIcons};
use crate::pages::{PageRouter, CustomPage};

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
            url if url.starts_with("neon://") => {
                // Handle neon:// protocol custom pages
                let router = PageRouter::new();
                if router.can_handle(&self.url) {
                    let html_content = self.generate_custom_page_html(&self.url);
                    self.title = router.get_page_title(&self.url).unwrap_or_else(|| "Custom Page".to_string());
                    self.web_page = Some(WebPage::create_simple_text_page(&html_content, &self.url));
                    self.loading = false;
                    false // No network request needed
                } else {
                    // Unknown neon:// URL
                    self.title = "Page Not Found".to_string();
                    self.web_page = Some(WebPage::create_error_page(&self.url, "Unknown neon:// page"));
                    self.loading = false;
                    false
                }
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
                ui.label(egui::RichText::new("X Error").size(20.0).color(egui::Color32::RED));
                ui.label(format!("URL: {}", self.url));
                ui.separator();
                ui.label(egui::RichText::new(error).color(egui::Color32::LIGHT_RED));
                ui.separator();
                ui.label("Try:");
                ui.label("- Check your internet connection");
                ui.label("- Make sure the URL is correct");
                ui.label("- Try adding https:// prefix");
                if ui.button(
                    egui::RichText::new(format!("{} Retry", NeonIcons::ARROW_CLOCKWISE))
                        .color(NeonTheme::NEON_BLUE)
                ).clicked() {
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

    /// Generate HTML content for custom neon:// pages
    fn generate_custom_page_html(&self, url: &str) -> String {
        let router = PageRouter::new();
        
        // Create a mock egui UI context for rendering
        // Since we need to convert egui output to HTML, we'll generate static HTML
        match url {
            "neon://about" => {
                format!(r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>About NeonSearch</title>
                    <style>
                        body {{ 
                            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                            background: linear-gradient(135deg, #1a1a2e, #16213e);
                            color: #ffffff;
                            margin: 0;
                            padding: 20px;
                            line-height: 1.6;
                        }}
                        .container {{
                            max-width: 800px;
                            margin: 0 auto;
                            background: rgba(0, 0, 0, 0.3);
                            border-radius: 12px;
                            padding: 30px;
                            border: 1px solid #00ffcc;
                        }}
                        .header {{
                            text-align: center;
                            margin-bottom: 30px;
                        }}
                        .logo {{
                            font-size: 2.5em;
                            color: #00ffcc;
                            margin-bottom: 10px;
                        }}
                        .version {{
                            color: #cccccc;
                            font-size: 1.2em;
                        }}
                        .section {{
                            margin: 20px 0;
                            padding: 20px;
                            background: rgba(255, 255, 255, 0.05);
                            border-radius: 8px;
                        }}
                        .section h3 {{
                            color: #00ffcc;
                            margin-top: 0;
                        }}
                        .feature-list {{
                            list-style: none;
                            padding: 0;
                        }}
                        .feature-list li {{
                            padding: 5px 0;
                            color: #ffffff;
                        }}
                        .feature-list li:before {{
                            content: "✓ ";
                            color: #00ffcc;
                            font-weight: bold;
                        }}
                    </style>
                </head>
                <body>
                    <div class="container">
                        <div class="header">
                            <div class="logo">🌐 NeonSearch</div>
                            <div class="version">Version 0.2.0</div>
                        </div>
                        
                        <div class="section">
                            <h3>About This Browser</h3>
                            <p>NeonSearch is a custom web browser built entirely from scratch in Rust. Unlike other browsers that use existing engines like Chromium or Gecko, every component from HTML parsing to GPU rendering is implemented natively.</p>
                        </div>
                        
                        <div class="section">
                            <h3>Key Features</h3>
                            <ul class="feature-list">
                                <li>Custom HTML/CSS parsing engine</li>
                                <li>Native Rust implementation</li>
                                <li>Advanced large website handling</li>
                                <li>5-tier adaptive rendering system</li>
                                <li>Virtual scrolling for massive pages</li>
                                <li>Streaming content processing</li>
                                <li>Custom neon:// protocol pages</li>
                                <li>Built-in JavaScript engine</li>
                            </ul>
                        </div>
                        
                        <div class="section">
                            <h3>System Information</h3>
                            <p><strong>Platform:</strong> {}</p>
                            <p><strong>Build Date:</strong> September 2025</p>
                            <p><strong>Engine:</strong> Custom Rust Engine</p>
                            <p><strong>UI Framework:</strong> egui 0.29</p>
                        </div>
                        
                        <div class="section">
                            <h3>Credits</h3>
                            <p>Developed by <strong>NeonDev™</strong></p>
                            <p>A from-scratch browser implementation showcasing the power and safety of Rust for systems programming.</p>
                        </div>
                    </div>
                </body>
                </html>
                "#, std::env::consts::OS)
            }
            "neon://settings" => {
                r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>NeonSearch Settings</title>
                    <style>
                        body { 
                            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                            background: linear-gradient(135deg, #1a1a2e, #16213e);
                            color: #ffffff;
                            margin: 0;
                            padding: 20px;
                            line-height: 1.6;
                        }
                        .container {
                            max-width: 900px;
                            margin: 0 auto;
                            background: rgba(0, 0, 0, 0.3);
                            border-radius: 12px;
                            padding: 30px;
                            border: 1px solid #00ffcc;
                        }
                        .header {
                            text-align: center;
                            margin-bottom: 30px;
                        }
                        .title {
                            font-size: 2.2em;
                            color: #00ffcc;
                            margin-bottom: 10px;
                        }
                        .tabs {
                            display: flex;
                            margin-bottom: 30px;
                            border-bottom: 2px solid #00ffcc;
                        }
                        .tab {
                            padding: 12px 24px;
                            background: rgba(255, 255, 255, 0.1);
                            margin-right: 5px;
                            border-radius: 8px 8px 0 0;
                            cursor: pointer;
                            color: #cccccc;
                        }
                        .tab.active {
                            background: #00ffcc;
                            color: #1a1a2e;
                        }
                        .section {
                            margin: 20px 0;
                            padding: 20px;
                            background: rgba(255, 255, 255, 0.05);
                            border-radius: 8px;
                        }
                        .section h3 {
                            color: #00ffcc;
                            margin-top: 0;
                        }
                        .setting-item {
                            display: flex;
                            justify-content: space-between;
                            align-items: center;
                            padding: 10px 0;
                            border-bottom: 1px solid rgba(255, 255, 255, 0.1);
                        }
                        .setting-item:last-child {
                            border-bottom: none;
                        }
                        input[type="checkbox"] {
                            width: 20px;
                            height: 20px;
                        }
                    </style>
                </head>
                <body>
                    <div class="container">
                        <div class="header">
                            <div class="title">⚙️ Settings</div>
                        </div>
                        
                        <div class="tabs">
                            <div class="tab active">General</div>
                            <div class="tab">Privacy</div>
                            <div class="tab">Performance</div>
                            <div class="tab">Appearance</div>
                            <div class="tab">Advanced</div>
                        </div>
                        
                        <div class="section">
                            <h3>General Settings</h3>
                            <div class="setting-item">
                                <span>Default Search Engine</span>
                                <select style="background: #2a2a3e; color: white; border: 1px solid #00ffcc; padding: 5px;">
                                    <option>DuckDuckGo</option>
                                    <option>Google</option>
                                    <option>Bing</option>
                                </select>
                            </div>
                            <div class="setting-item">
                                <span>Homepage</span>
                                <input type="text" value="neon://home" style="background: #2a2a3e; color: white; border: 1px solid #00ffcc; padding: 5px;">
                            </div>
                        </div>
                        
                        <div class="section">
                            <h3>Privacy & Security</h3>
                            <div class="setting-item">
                                <span>Enable JavaScript</span>
                                <input type="checkbox" checked>
                            </div>
                            <div class="setting-item">
                                <span>Block Third-Party Cookies</span>
                                <input type="checkbox">
                            </div>
                            <div class="setting-item">
                                <span>Enable HTTPS-Only Mode</span>
                                <input type="checkbox" checked>
                            </div>
                        </div>
                        
                        <div class="section">
                            <h3>Performance</h3>
                            <div class="setting-item">
                                <span>Enable Hardware Acceleration</span>
                                <input type="checkbox" checked>
                            </div>
                            <div class="setting-item">
                                <span>Maximum Content Size</span>
                                <select style="background: #2a2a3e; color: white; border: 1px solid #00ffcc; padding: 5px;">
                                    <option>50 MB</option>
                                    <option>100 MB</option>
                                    <option>200 MB</option>
                                </select>
                            </div>
                        </div>
                    </div>
                </body>
                </html>
                "#.to_string()
            }
            _ => {
                // For other neon:// pages, create a basic "coming soon" page
                let page_name = url.strip_prefix("neon://").unwrap_or("unknown");
                format!(r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>{} - NeonSearch</title>
                    <style>
                        body {{ 
                            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                            background: linear-gradient(135deg, #1a1a2e, #16213e);
                            color: #ffffff;
                            margin: 0;
                            padding: 40px;
                            text-align: center;
                        }}
                        .container {{
                            max-width: 600px;
                            margin: 0 auto;
                            background: rgba(0, 0, 0, 0.3);
                            border-radius: 12px;
                            padding: 40px;
                            border: 1px solid #00ffcc;
                        }}
                        .icon {{
                            font-size: 4em;
                            margin-bottom: 20px;
                        }}
                        .title {{
                            font-size: 2em;
                            color: #00ffcc;
                            margin-bottom: 20px;
                            text-transform: capitalize;
                        }}
                        .message {{
                            font-size: 1.2em;
                            color: #cccccc;
                            margin-bottom: 30px;
                        }}
                        .link {{
                            color: #00ffcc;
                            text-decoration: none;
                            margin: 0 10px;
                        }}
                        .link:hover {{
                            text-decoration: underline;
                        }}
                    </style>
                </head>
                <body>
                    <div class="container">
                        <div class="icon">🚧</div>
                        <div class="title">{} Page</div>
                        <div class="message">This page is under development and will be available in a future version of NeonSearch.</div>
                        <div>
                            <a href="neon://about" class="link">About</a>
                            <a href="neon://settings" class="link">Settings</a>
                            <a href="about:home" class="link">Home</a>
                        </div>
                    </div>
                </body>
                </html>
                "#, page_name, page_name)
            }
        }
    }
}

impl Drop for BrowserTab {
    fn drop(&mut self) {
        // Ensure cleanup on drop
        self.cleanup_temp_files();
    }
}