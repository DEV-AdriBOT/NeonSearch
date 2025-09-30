use eframe::egui::{self, Color32, Rounding, Shadow, Stroke, Vec2};
use tokio::runtime::Runtime;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use uuid::Uuid;
use crate::networking::HttpResponse;
use crate::networking::cookie_manager::CookieManager;
use crate::networking::manual_client::{ManualHttpClient, FetchPhase};
use crate::networking::image_loader::ImageCache;
use crate::pages::PageRouter;

mod browser_tab;
mod address_bar;
mod navigation;
mod bookmarks;
pub mod theme;
mod error_handler;
mod dev_console;
pub mod icons;

pub use browser_tab::BrowserTab;
pub use address_bar::AddressBar;
pub use navigation::NavigationBar;
pub use bookmarks::BookmarkManager;
pub use theme::NeonTheme;
pub use error_handler::{BrowserError, ErrorType, ErrorRecovery};
pub use dev_console::DevConsole;
pub use icons::NeonIcons;

pub struct NeonSearchApp {
    tabs: HashMap<Uuid, BrowserTab>,
    active_tab: Option<Uuid>,
    address_bar: AddressBar,
    navigation_bar: NavigationBar,
    bookmark_manager: BookmarkManager,
    dev_console: DevConsole,
    page_router: PageRouter,
    show_bookmarks: bool,
    show_settings: bool,
    network_receiver: Receiver<(Uuid, Result<HttpResponse, String>)>,
    network_sender: Sender<(Uuid, Result<HttpResponse, String>)>,
    runtime: Runtime,
    cookies: CookieManager,
    loading_tabs: HashMap<Uuid, std::time::Instant>,
    manual_client: ManualHttpClient,
    tab_phases: HashMap<Uuid, Vec<FetchPhase>>,
    image_cache: ImageCache,
}

impl NeonSearchApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Apply the modern Neon theme
        NeonTheme::apply_to_context(&cc.egui_ctx);
        let (network_sender, network_receiver) = mpsc::channel();
        let runtime = Runtime::new().expect("Failed to build Tokio runtime");
        
        let mut app = Self {
            tabs: HashMap::new(),
            active_tab: None,
            address_bar: AddressBar::new(),
            navigation_bar: NavigationBar::new(),
            bookmark_manager: BookmarkManager::new(),
            dev_console: DevConsole::new(),
            page_router: PageRouter::new(),
            show_bookmarks: false,
            show_settings: false,
            network_receiver,
            network_sender,
            runtime,
            cookies: CookieManager::new(),
            loading_tabs: HashMap::new(),
            manual_client: ManualHttpClient::new().expect("manual client init"),
            tab_phases: HashMap::new(),
            image_cache: ImageCache::new(),
        };
        
        // Create initial tab
        app.create_new_tab();
        
        app
    }
    
    fn create_new_tab(&mut self) -> Uuid {
        let tab_id = Uuid::new_v4();
        let tab = BrowserTab::new("New Tab".to_string());
        self.tabs.insert(tab_id, tab);
        self.active_tab = Some(tab_id);
        tab_id
    }
    
    fn close_tab(&mut self, tab_id: Uuid) {
        if self.tabs.len() == 1 {
            // Don't close the last tab, just navigate to home
            if let Some(tab) = self.tabs.get_mut(&tab_id) {
                tab.navigate_to("about:home".to_string());
            }
            return;
        }
        
        self.tabs.remove(&tab_id);
        
        if self.active_tab == Some(tab_id) {
            // Set active tab to the first remaining tab
            self.active_tab = self.tabs.keys().next().copied();
        }
    }
    
    pub fn fetch_url(&self, tab_id: Uuid, url: String) {
        // Check if this is a custom page first
        if self.page_router.can_handle(&url) {
            // Handle custom pages immediately without network request
            let sender = self.network_sender.clone();
            let page_title = self.page_router.get_page_title(&url).unwrap_or("Custom Page".to_string());
            
            // Create a fake response for custom pages
            let headers = std::collections::HashMap::new();
            let content = format!("<html><head><title>{}</title></head><body><!-- CUSTOM_PAGE --></body></html>", page_title);
            let response = HttpResponse::new(200, "OK".to_string(), headers, content.into_bytes());
            
            // Send immediately
            let _ = sender.send((tab_id, Ok(response)));
            return;
        }
        
        let sender = self.network_sender.clone();
        let manual = self.manual_client.clone();
        let cookie_header;
        let uri = url.clone();
        // Basic cookie header assembly (domain + path split)
        if let Ok(parsed) = reqwest::Url::parse(&uri) {
            let domain = parsed.host_str().unwrap_or("");
            let path = parsed.path();
            let is_secure = parsed.scheme() == "https";
            cookie_header = self.cookies.get_cookie_header_for_request(domain, path, is_secure);
        } else {
            cookie_header = None;
        }
        let original_url = url.clone();
        self.runtime.spawn(async move {
            // Manual attempt first
            let manual_attempt = manual.fetch(&url).await;
            let result = match manual_attempt {
                Ok(res) => Ok(res.response),
                Err(e) => {
                    // Enhanced error handling with detailed categorization
                    let err_str = e.to_string();
                    
                    if let Some(redirect_url) = err_str.strip_prefix("REDIRECT:") {
                        // Handle redirects by creating a proper redirect response
                        let mut headers = std::collections::HashMap::new();
                        headers.insert("Location".to_string(), redirect_url.to_string());
                        Ok(HttpResponse::new(302, "Found".to_string(), headers, Vec::new()))
                    } else {
                        // Create detailed browser error for better user experience
                        let browser_error = BrowserError::from_anyhow(&e, Some(&url));
                        
                        // Determine if we should attempt reqwest fallback
                        let should_fallback = matches!(browser_error.error_type,
                            ErrorType::TlsHandshakeFailed | 
                            ErrorType::NetworkTimeout |
                            ErrorType::InternalError
                        ) || err_str.contains("TLS_CLOSE_NOTIFY_ISSUE");
                        
                        if should_fallback {
                            println!("🔄 Attempting reqwest fallback for {} ({})", url, browser_error.error_type);
                            
                            let mut request = crate::networking::HttpRequest::new_get(url.clone());
                            if let Some(c) = cookie_header.clone() { 
                                request.headers.insert("Cookie".to_string(), c); 
                            }
                            
                            match crate::networking::http_client::send_request(request).await {
                                Ok(response) => {
                                    println!("✅ Reqwest fallback succeeded for {}", url);
                                    Ok(response)
                                },
                                Err(fallback_err) => {
                                    // Both manual and reqwest failed
                                    println!("❌ Both manual and reqwest failed for {}", url);
                                    Err(format!(
                                        "{}. Fallback error: {}", 
                                        browser_error.message,
                                        fallback_err
                                    ))
                                }
                            }
                        } else {
                            // Don't fallback for DNS errors, 404s, etc.
                            println!("⚠️  No fallback attempted for {} ({})", url, browser_error.error_type);
                            Err(browser_error.message)
                        }
                    }
                }
            };
            if let Err(e) = &result { eprintln!("[network] Failed to fetch {original_url}: {e}"); }
            let _ = sender.send((tab_id, result));
        });
    }
    
    fn process_network_responses(&mut self) {
        while let Ok((tab_id, result)) = self.network_receiver.try_recv() {
            if let Some(tab) = self.tabs.get_mut(&tab_id) {
                if let Err(e) = &result {
                    eprintln!("[network] response error for tab {tab_id}: {e}");
                }
                if let Ok(resp) = &result {
                    // Parse cookies
                    if let Ok(parsed_url) = reqwest::Url::parse(&tab.url) {
                        let domain = parsed_url.host_str().unwrap_or("");
                        for (k, v) in &resp.headers {
                            if k.eq_ignore_ascii_case("set-cookie") {
                                self.cookies.parse_set_cookie_header(v, domain);
                            }
                        }
                    }
                }
                let was_redirect = match &result {
                    Ok(r) if r.is_redirect() => true,
                    _ => false,
                };
                // Extract HTML content before handling response (for favicon loading)
                let html_content = if let Ok(ref resp) = result {
                    if resp.is_success() {
                        resp.body_as_string().ok()
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                tab.handle_network_response(result);
                
                // Preload favicon if we successfully loaded HTML content
                if let Some(html) = html_content {
                    let image_cache = self.image_cache.clone();
                    let manual_client = self.manual_client.clone();
                    let base_url = tab.url.clone();
                    
                    self.runtime.spawn(async move {
                        image_cache.preload_favicon(&base_url, &html, &manual_client).await;
                    });
                }
                
                if was_redirect {
                    // Follow redirect automatically
                    let current_url = tab.url.clone();
                    if tab.redirects_followed > 0 && tab.redirects_followed <= 5 {
                        self.fetch_url(tab_id, current_url.clone());
                        self.loading_tabs.insert(tab_id, std::time::Instant::now());
                    }
                } else {
                    self.loading_tabs.remove(&tab_id);
                }
            }
        }
    }
}

impl eframe::App for NeonSearchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process any incoming network responses
        self.process_network_responses();
        
        // Handle keyboard shortcuts (but not when address bar has focus to avoid input interference)
        let address_bar_has_focus = ctx.memory(|mem| {
            mem.has_focus(egui::Id::new("address_bar_input"))
        });
        
        if !address_bar_has_focus {
            ctx.input(|i| {
                // Cmd+L (or Ctrl+L) to focus address bar
                if (i.modifiers.mac_cmd || i.modifiers.ctrl) && i.key_pressed(egui::Key::L) {
                    self.address_bar.focus();
                }
                
                // Cmd+T (or Ctrl+T) to create new tab
                if (i.modifiers.mac_cmd || i.modifiers.ctrl) && i.key_pressed(egui::Key::T) {
                    self.create_new_tab();
                }
                
                // Cmd+R (or Ctrl+R) to reload
                if (i.modifiers.mac_cmd || i.modifiers.ctrl) && i.key_pressed(egui::Key::R) {
                    if let Some(active_id) = self.active_tab {
                        if let Some(active_tab) = self.tabs.get_mut(&active_id) {
                            let needs_fetch = active_tab.reload();
                            let current_url = active_tab.url.clone();
                            
                            if needs_fetch {
                                self.fetch_url(active_id, current_url);
                            }
                        }
                    }
                }
                
                // F12 to toggle developer console
                if i.key_pressed(egui::Key::F12) {
                    self.dev_console.toggle_visibility();
                }
            });
        } else {
            // Still handle Cmd+L even when focused to allow re-focusing
            ctx.input(|i| {
                if (i.modifiers.mac_cmd || i.modifiers.ctrl) && i.key_pressed(egui::Key::L) {
                    self.address_bar.focus();
                }
            });
        }
        
        // Modern top panel with sophisticated design
        egui::TopBottomPanel::top("top_panel")
            .exact_height(80.0)
            .frame(egui::Frame::none().fill(NeonTheme::DARK_BG))
            .show(ctx, |ui| {
                let rect = ui.available_rect_before_wrap();
                
                // Create modern background with subtle gradient effect
                NeonTheme::create_modern_background(ui, rect);
                
                // Add bottom accent line
                ui.painter().hline(
                    rect.left()..=rect.right(),
                    rect.bottom(),
                    egui::Stroke::new(2.0, NeonTheme::NEON_CYAN.gamma_multiply(0.6))
                );
                
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(ui.available_width(), 80.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.add_space(20.0);
                        
                        // Enhanced NeonSearch logo with glow
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(200.0, ui.available_height()),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                // Logo with modern styling
                                let logo_rect = ui.available_rect_before_wrap();
                                
                                ui.horizontal(|ui| {
                                    ui.add_space(4.0);
                                    
                                    // Lightning bolt with glow effect
                                    let lightning_response = ui.label(
                                        egui::RichText::new("⚡")
                                            .size(32.0)
                                            .color(NeonTheme::NEON_CYAN)
                                    );
                                    
                                    // Add glow around the lightning
                                    NeonTheme::add_glow_effect(ui, lightning_response.rect, NeonTheme::NEON_CYAN, 0.8);
                                    
                                    ui.add_space(8.0);
                                    
                                    // Brand name
                                    ui.label(
                                        egui::RichText::new("NeonSearch")
                                            .size(28.0)
                                            .color(NeonTheme::PRIMARY_TEXT)
                                            .strong()
                                    );
                                });
                            },
                        );
                        
                        ui.add_space(24.0);
                        
                        // Navigation controls with modern styling
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(200.0, ui.available_height()),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                // Use existing navigation bar
                                let nav_action = self.navigation_bar.show(ui, self.active_tab.and_then(|id| self.tabs.get(&id)));
                                
                                // Handle navigation actions (simplified)
                                if let Some(active_id) = self.active_tab {
                                    if let Some(active_tab) = self.tabs.get_mut(&active_id) {
                                        let needs_fetch = match nav_action {
                                            crate::ui::navigation::NavigationAction::Back => active_tab.go_back(),
                                            crate::ui::navigation::NavigationAction::Forward => active_tab.go_forward(),
                                            crate::ui::navigation::NavigationAction::Reload => active_tab.reload(),
                                            crate::ui::navigation::NavigationAction::Home => active_tab.navigate_to("about:home".to_string()),
                                            crate::ui::navigation::NavigationAction::None => false,
                                        };
                                        
                                        if needs_fetch {
                                            let current_url = active_tab.url.clone();
                                            self.fetch_url(active_id, current_url);
                                        }
                                    }
                                }
                            },
                        );
                        
                        ui.add_space(16.0);
                        
                        // Address bar with enhanced styling
                        let remaining_width = ui.available_width().max(400.0) - 180.0;
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(remaining_width, ui.available_height()),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                // Use existing address bar
                                if let Some(navigate_url) = self.address_bar.show(ui) {
                                    if let Some(active_id) = self.active_tab {
                                        if let Some(active_tab) = self.tabs.get_mut(&active_id) {
                                            // Normalize URL - add https:// if no protocol is specified
                                            let normalized_url = if !navigate_url.starts_with("http://") && 
                                                                    !navigate_url.starts_with("https://") && 
                                                                    !navigate_url.starts_with("about:") {
                                                format!("https://{}", navigate_url)
                                            } else {
                                                navigate_url
                                            };
                                            
                                            let needs_fetch = active_tab.navigate_to(normalized_url);
                                            let current_url = active_tab.url.clone();
                                            self.address_bar.set_url(current_url.clone());
                                            
                                            if needs_fetch {
                                                self.fetch_url(active_id, current_url);
                                                self.loading_tabs.insert(active_id, std::time::Instant::now());
                                            }
                                        }
                                    }
                                }
                            },
                        );
                        
                        ui.add_space(16.0);
                        
                        // Modern action buttons
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 12.0;
                            
                            // Bookmarks button with modern styling
                            let bookmarks_btn = egui::Button::new(
                                egui::RichText::new(format!("{} Bookmarks", icons::NeonIcons::BOOKMARKS))
                                    .size(13.0)
                                    .color(if self.show_bookmarks { NeonTheme::PRIMARY_TEXT } else { NeonTheme::SECONDARY_TEXT })
                            )
                            .fill(if self.show_bookmarks { NeonTheme::BUTTON_PRIMARY } else { NeonTheme::BUTTON_IDLE })
                            .stroke(Stroke::new(1.0, if self.show_bookmarks { NeonTheme::NEON_CYAN } else { NeonTheme::BORDER_COLOR }))
                            .rounding(Rounding::same(12.0));
                            
                            if ui.add(bookmarks_btn).on_hover_text("Manage bookmarks").clicked() {
                                self.show_bookmarks = !self.show_bookmarks;
                            }
                            
                            // Settings button
                            let settings_btn = egui::Button::new(
                                egui::RichText::new(format!("{} Settings", icons::NeonIcons::GEAR))
                                    .size(13.0)
                                    .color(if self.show_settings { NeonTheme::PRIMARY_TEXT } else { NeonTheme::SECONDARY_TEXT })
                            )
                            .fill(if self.show_settings { NeonTheme::BUTTON_PRIMARY } else { NeonTheme::BUTTON_IDLE })
                            .stroke(Stroke::new(1.0, if self.show_settings { NeonTheme::NEON_CYAN } else { NeonTheme::BORDER_COLOR }))
                            .rounding(Rounding::same(12.0));
                            
                            if ui.add(settings_btn).on_hover_text("Browser settings").clicked() {
                                self.show_settings = !self.show_settings;
                            }
                            
                            // New tab button with primary styling
                            let new_tab_btn = egui::Button::new(
                                egui::RichText::new(format!("{} New", icons::NeonIcons::PLUS))
                                    .size(13.0)
                                    .color(NeonTheme::PRIMARY_TEXT)
                            )
                            .fill(NeonTheme::SUCCESS_COLOR)
                            .stroke(Stroke::new(0.0, Color32::TRANSPARENT))
                            .rounding(Rounding::same(12.0));
                            
                            if ui.add(new_tab_btn).on_hover_text("Open new tab (Cmd+T)").clicked() {
                                self.create_new_tab();
                            }
                        });
                        
                        ui.add_space(20.0);
                    },
                );
            });
        
        // Handle navigation and address bar logic (simplified for now)
        // TODO: Re-implement navigation action handling
        
        // Modern tab bar with enhanced styling
        egui::TopBottomPanel::top("tab_panel")
            .exact_height(50.0)
            .frame(egui::Frame::none().fill(NeonTheme::PANEL_BG))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    
                    egui::ScrollArea::horizontal()
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;
                                let mut tabs_to_close = Vec::new();
                                
                                for (&tab_id, tab) in &self.tabs {
                                    let is_active = self.active_tab == Some(tab_id);
                                    let is_loading = self.loading_tabs.contains_key(&tab_id);
                                    
                                    // Create modern tab styling
                                    let tab_bg = if is_active {
                                        NeonTheme::NEON_PURPLE
                                    } else {
                                        NeonTheme::ELEVATED_BG
                                    };
                                    
                                    let tab_text_color = if is_active {
                                        egui::Color32::WHITE
                                    } else {
                                        NeonTheme::PRIMARY_TEXT
                                    };
                                    
                                    egui::Frame::none()
                                        .fill(tab_bg)
                                        .rounding(egui::Rounding::same(8.0))
                                        .stroke(egui::Stroke::new(1.0, if is_active { NeonTheme::NEON_CYAN } else { NeonTheme::BORDER_COLOR }))
                                        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 8.0;
                                                
                                                // Favicon placeholder
                                                let favicon_text = if is_loading {
                                                    let phase_anim = (ctx.input(|i| i.time) * 6.0) as i32 % 4;
                                                    match phase_anim { 0 => "⠋", 1 => "⠙", 2 => "⠹", _ => "⠸" }
                                                } else {
                                                    icons::NeonIcons::GLOBE_SIMPLE
                                                };
                                                ui.label(egui::RichText::new(favicon_text).color(tab_text_color));
                                                
                                                // Tab title with loading indicator
                                                let mut title = tab.title.clone();
                                                if title.len() > 25 {
                                                    title = format!("{}...", &title[..22]);
                                                }
                                                
                                                if is_loading {
                                                    let phase_label = self.tab_phases.get(&tab_id)
                                                        .and_then(|v| v.last())
                                                        .map(|p| match p {
                                                            FetchPhase::Resolving => "Resolving DNS",
                                                            FetchPhase::Connecting => "Connecting",
                                                            FetchPhase::TlsHandshake => "Securing connection",
                                                            FetchPhase::SendingRequest => "Sending request",
                                                            FetchPhase::ReadingHeaders => "Reading headers",
                                                            FetchPhase::ReadingBody => "Loading content",
                                                            FetchPhase::Redirecting => "Following redirect",
                                                            FetchPhase::Completed => "Complete"
                                                        }).unwrap_or("Loading");
                                                    title = format!("{} • {}", title, phase_label);
                                                }
                                                
                                                let tab_response = ui.selectable_label(
                                                    false, // We handle selection with background color
                                                    egui::RichText::new(title)
                                                        .color(tab_text_color)
                                                        .size(13.0)
                                                );
                                                
                                                if tab_response.clicked() {
                                                    self.active_tab = Some(tab_id);
                                                }
                                                
                                                // Close button with hover effect
                                                let close_btn = egui::Button::new(
                                                    egui::RichText::new(icons::NeonIcons::X)
                                                        .size(12.0)
                                                        .color(if is_active { egui::Color32::WHITE } else { NeonTheme::MUTED_TEXT })
                                                )
                                                .fill(egui::Color32::TRANSPARENT)
                                                .stroke(egui::Stroke::NONE);
                                                
                                                if ui.add(close_btn)
                                                    .on_hover_text("Close tab")
                                                    .clicked() 
                                                {
                                                    tabs_to_close.push(tab_id);
                                                }
                                            });
                                        });
                                    
                                    ui.add_space(2.0);
                                }
                                
                                // Close tabs after iteration to avoid borrow checker issues
                                for tab_id in tabs_to_close {
                                    self.close_tab(tab_id);
                                }
                            });
                        });
                });
            });
        
        // Main content area with enhanced styling
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(NeonTheme::DARK_BG)
                    .inner_margin(egui::Margin::symmetric(16.0, 12.0))
            )
            .show(ctx, |ui| {
                if let Some(active_id) = self.active_tab {
                    let mut needs_fetch = false;
                    let mut current_url = String::new();
                    
                    if let Some(active_tab) = self.tabs.get_mut(&active_id) {
                        current_url = active_tab.url.clone();
                        
                        // Check if this is a custom page
                        if self.page_router.can_handle(&current_url) {
                            // Render custom page directly
                            egui::Frame::none()
                                .fill(NeonTheme::CARD_BG)
                                .rounding(egui::Rounding::same(12.0))
                                .stroke(egui::Stroke::new(1.0, NeonTheme::BORDER_COLOR))
                                .inner_margin(egui::Margin::same(16.0))
                                .show(ui, |ui| {
                                    egui::ScrollArea::vertical()
                                        .auto_shrink([false; 2])
                                        .show(ui, |ui| {
                                            self.page_router.render_page(&current_url, ui, ctx);
                                        });
                                });
                            
                            // Update tab title if needed
                            if let Some(page_title) = self.page_router.get_page_title(&current_url) {
                                active_tab.title = page_title;
                            }
                        } else {
                            // Render normal web page
                            egui::Frame::none()
                                .fill(NeonTheme::CARD_BG)
                                .rounding(egui::Rounding::same(12.0))
                                .stroke(egui::Stroke::new(1.0, NeonTheme::BORDER_COLOR))
                                .inner_margin(egui::Margin::same(16.0))
                                .show(ui, |ui| {
                                    needs_fetch = active_tab.show(ui);
                                });
                        }
                    }
                    
                    if needs_fetch {
                        self.fetch_url(active_id, current_url);
                        self.loading_tabs.insert(active_id, std::time::Instant::now());
                    }
                } else {
                    // No tab selected - show welcome screen
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("⚡ Welcome to NeonSearch")
                                    .size(32.0)
                                    .color(NeonTheme::NEON_CYAN)
                                    .strong()
                            );
                            ui.add_space(16.0);
                            ui.label(
                                egui::RichText::new("Press Cmd+T to open a new tab")
                                    .size(16.0)
                                    .color(NeonTheme::SECONDARY_TEXT)
                            );
                        });
                    });
                }
            });
        
        // Developer Console (F12 to toggle)
        if self.dev_console.is_visible() {
            egui::TopBottomPanel::bottom("dev_console_panel")
                .resizable(true)
                .default_height(300.0)
                .frame(
                    egui::Frame::none()
                        .fill(NeonTheme::DARKER_BG)
                        .stroke(egui::Stroke::new(1.0, NeonTheme::BORDER_COLOR))
                        .inner_margin(egui::Margin::same(8.0))
                )
                .show(ctx, |ui| {
                    // Get reference to active tab's JS engine
                    if let Some(active_id) = self.active_tab {
                        if let Some(active_tab) = self.tabs.get_mut(&active_id) {
                            if let Some(ref mut web_page) = active_tab.web_page {
                                self.dev_console.render(ui, &mut web_page.js_engine);
                            } else {
                                // No webpage loaded, render with None
                                let mut no_engine: Option<crate::js::JSEngine> = None;
                                self.dev_console.render(ui, &mut no_engine);
                            }
                        } else {
                            // No active tab, render with None
                            let mut no_engine: Option<crate::js::JSEngine> = None;
                            self.dev_console.render(ui, &mut no_engine);
                        }
                    } else {
                        // No active tab, render with None
                        let mut no_engine: Option<crate::js::JSEngine> = None;
                        self.dev_console.render(ui, &mut no_engine);
                    }
                });
        }
        
        // Side panels with enhanced styling
        if self.show_bookmarks {
            egui::SidePanel::right("bookmarks_panel")
                .resizable(true)
                .default_width(280.0)
                .frame(
                    egui::Frame::none()
                        .fill(NeonTheme::DARKER_BG)
                        .stroke(egui::Stroke::new(1.0, NeonTheme::BORDER_COLOR))
                        .inner_margin(egui::Margin::same(12.0))
                )
                .show(ctx, |ui| {
                    // Panel header
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{} Bookmarks", icons::NeonIcons::BOOKMARKS))
                                .size(18.0)
                                .color(NeonTheme::NEON_CYAN)
                                .strong()
                        );
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let close_btn = egui::Button::new(
                                egui::RichText::new(icons::NeonIcons::X).color(NeonTheme::MUTED_TEXT)
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::NONE);
                            
                            if ui.add(close_btn).on_hover_text("Close bookmarks").clicked() {
                                self.show_bookmarks = false;
                            }
                        });
                    });
                    
                    ui.separator();
                    ui.add_space(8.0);
                    
                    if let Some(navigate_url) = self.bookmark_manager.show(ui) {
                        if let Some(active_id) = self.active_tab {
                            if let Some(active_tab) = self.tabs.get_mut(&active_id) {
                                let needs_fetch = active_tab.navigate_to(navigate_url);
                                let current_url = active_tab.url.clone();
                                self.address_bar.set_url(current_url.clone());
                                
                                if needs_fetch {
                                    self.fetch_url(active_id, current_url);
                                }
                            }
                        }
                    }
                });
        }
        
        // Settings window
        if self.show_settings {
            egui::Window::new("Settings")
                .open(&mut self.show_settings)
                .default_size([400.0, 300.0])
                .show(ctx, |ui| {
                    ui.heading("NeonSearch Settings");
                    
                    ui.separator();
                    
                    ui.label(
                        egui::RichText::new(format!("{} Settings panel coming soon!", NeonIcons::WRENCH))
                            .color(NeonTheme::WARNING_COLOR)
                    );
                    ui.label("This will include:");
                    ui.label("• Theme customization");
                    ui.label("• Privacy settings");
                    ui.label("• Search engine preferences");
                    ui.label("• Extensions management");
                });
        }
    }
}