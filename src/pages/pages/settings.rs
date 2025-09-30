use eframe::egui::{Context, RichText, Ui, Slider};
use crate::pages::{CustomPage, components};
use crate::ui::theme::NeonTheme;
use crate::ui::icons::NeonIcons;

pub struct SettingsPage {
    url: String,
    title: String,
    // Settings state
    current_tab: SettingsTab,
    // General settings
    startup_page: StartupOption,
    search_engine: SearchEngine,
    downloads_path: String,
    // Privacy settings
    cookies_enabled: bool,
    javascript_enabled: bool,
    images_enabled: bool,
    tracking_protection: bool,
    // Appearance settings
    theme: ThemeOption,
    font_size: f32,
    show_bookmarks_bar: bool,
    // Performance settings
    cache_size: f32,
    max_connections: i32,
    enable_hardware_acceleration: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum SettingsTab {
    General,
    Privacy,
    Appearance,
    Performance,
    Advanced,
}

#[derive(Debug, Clone, PartialEq)]
enum StartupOption {
    HomePage,
    NewTab,
    LastSession,
}

#[derive(Debug, Clone, PartialEq)]
enum SearchEngine {
    DuckDuckGo,
    Google,
    Bing,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
enum ThemeOption {
    Dark,
    Light,
    Auto,
}

impl SettingsPage {
    pub fn new() -> Self {
        Self {
            url: "neon://settings".to_string(),
            title: "Settings".to_string(),
            current_tab: SettingsTab::General,
            // Default settings
            startup_page: StartupOption::HomePage,
            search_engine: SearchEngine::DuckDuckGo,
            downloads_path: "~/Downloads".to_string(),
            cookies_enabled: true,
            javascript_enabled: true,
            images_enabled: true,
            tracking_protection: true,
            theme: ThemeOption::Dark,
            font_size: 14.0,
            show_bookmarks_bar: true,
            cache_size: 100.0,
            max_connections: 10,
            enable_hardware_acceleration: true,
        }
    }
}

impl CustomPage for SettingsPage {
    fn get_url(&self) -> &str {
        &self.url
    }
    
    fn get_title(&self) -> &str {
        &self.title
    }
    
    fn render(&mut self, ui: &mut Ui, _ctx: &Context) {
        components::page_header(
            ui, 
            "Settings", 
            Some("Customize your NeonSearch browsing experience")
        );
        
        ui.horizontal(|ui| {
            // Left sidebar with settings categories
            ui.vertical(|ui| {
                ui.set_min_width(200.0);
                
                components::card_container(ui, |ui| {
                    ui.vertical(|ui| {
                        self.settings_nav_button(ui, SettingsTab::General, NeonIcons::GEAR, "General");
                        self.settings_nav_button(ui, SettingsTab::Privacy, NeonIcons::SHIELD_CHECK, "Privacy & Security");
                        self.settings_nav_button(ui, SettingsTab::Appearance, "🎨", "Appearance");
                        self.settings_nav_button(ui, SettingsTab::Performance, "⚡", "Performance");
                        self.settings_nav_button(ui, SettingsTab::Advanced, NeonIcons::WRENCH, "Advanced");
                    });
                });
            });
            
            ui.add_space(20.0);
            
            // Main content area
            ui.vertical(|ui| {
                ui.set_min_width(600.0);
                
                match self.current_tab {
                    SettingsTab::General => self.render_general_settings(ui),
                    SettingsTab::Privacy => self.render_privacy_settings(ui),
                    SettingsTab::Appearance => self.render_appearance_settings(ui),
                    SettingsTab::Performance => self.render_performance_settings(ui),
                    SettingsTab::Advanced => self.render_advanced_settings(ui),
                }
            });
        });
    }
}

impl SettingsPage {
    fn settings_nav_button(&mut self, ui: &mut Ui, tab: SettingsTab, icon: &str, label: &str) {
        let is_selected = self.current_tab == tab;
        
        let (bg_color, text_color) = if is_selected {
            (NeonTheme::NEON_CYAN.linear_multiply(0.2), NeonTheme::NEON_CYAN)
        } else {
            (egui::Color32::TRANSPARENT, NeonTheme::SECONDARY_TEXT)
        };
        
        let response = egui::Frame::none()
            .fill(bg_color)
            .rounding(8.0)
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icon).size(16.0).color(text_color));
                    ui.add_space(8.0);
                    ui.label(RichText::new(label).color(text_color));
                })
            })
            .response;
        
        if response.clicked() {
            self.current_tab = tab;
        }
        
        // Hover effect
        if response.hovered() && !is_selected {
            ui.painter().rect_filled(
                response.rect,
                8.0,
                NeonTheme::SECONDARY_TEXT.linear_multiply(0.1)
            );
        }
        
        ui.add_space(4.0);
    }
    
    fn render_general_settings(&mut self, ui: &mut Ui) {
        components::section_header(ui, NeonIcons::GEAR, "General Settings");
        
        components::card_container(ui, |ui| {
            // Startup options
            ui.label(RichText::new("When NeonSearch starts")
                .strong()
                .color(NeonTheme::PRIMARY_TEXT));
            
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.startup_page, StartupOption::HomePage, "Open the home page");
            });
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.startup_page, StartupOption::NewTab, "Open a new tab");
            });
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.startup_page, StartupOption::LastSession, "Continue where you left off");
            });
            
            ui.add_space(20.0);
            
            // Search engine
            ui.label(RichText::new("Default search engine")
                .strong()
                .color(NeonTheme::PRIMARY_TEXT));
            
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.search_engine, SearchEngine::DuckDuckGo, "DuckDuckGo (Privacy-focused)");
            });
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.search_engine, SearchEngine::Google, "Google");
            });
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.search_engine, SearchEngine::Bing, "Bing");
            });
            
            ui.add_space(20.0);
            
            // Downloads
            ui.label(RichText::new("Downloads")
                .strong()
                .color(NeonTheme::PRIMARY_TEXT));
            
            ui.horizontal(|ui| {
                ui.label("Save files to:");
                ui.add(egui::TextEdit::singleline(&mut self.downloads_path)
                    .desired_width(300.0));
                    
                if ui.button(RichText::new(format!("{} Browse...", NeonIcons::FOLDER))
                    .color(NeonTheme::NEON_CYAN)).clicked() {
                    // TODO: Open file dialog
                    // For now, show a placeholder path
                    self.downloads_path = "~/Downloads".to_string();
                }
            });
            
            ui.add_space(12.0);
            
            // Quick folder shortcuts
            ui.horizontal(|ui| {
                ui.label("Quick access:");
                if ui.small_button("Desktop").clicked() {
                    self.downloads_path = "~/Desktop".to_string();
                }
                if ui.small_button("Downloads").clicked() {
                    self.downloads_path = "~/Downloads".to_string();
                }
                if ui.small_button("Documents").clicked() {
                    self.downloads_path = "~/Documents".to_string();
                }
            });
        });
    }
    
    fn render_privacy_settings(&mut self, ui: &mut Ui) {
        components::section_header(ui, NeonIcons::SHIELD_CHECK, "Privacy & Security");
        
        components::card_container(ui, |ui| {
            ui.label(RichText::new("Web Content")
                .size(18.0)
                .strong()
                .color(NeonTheme::PRIMARY_TEXT));
            
            ui.add_space(12.0);
            
            ui.checkbox(&mut self.javascript_enabled, "Enable JavaScript");
            ui.checkbox(&mut self.images_enabled, "Load images");
            ui.checkbox(&mut self.cookies_enabled, "Accept cookies");
            
            ui.add_space(20.0);
            
            ui.label(RichText::new("Privacy Protection")
                .size(18.0)
                .strong()
                .color(NeonTheme::PRIMARY_TEXT));
            
            ui.add_space(12.0);
            
            ui.checkbox(&mut self.tracking_protection, "Block tracking requests");
            
            ui.add_space(20.0);
            
            // Privacy actions
            ui.horizontal(|ui| {
                if ui.button(RichText::new(format!("{} Clear Browsing Data...", NeonIcons::TRASH))
                    .color(NeonTheme::error_color())).clicked() {
                    // TODO: Open clear data dialog
                    // For now, show confirmation
                    println!("Clear browsing data clicked");
                }
                
                ui.add_space(8.0);
                
                if ui.button(RichText::new(format!("{} Manage Cookies...", NeonIcons::GEAR))
                    .color(NeonTheme::NEON_CYAN)).clicked() {
                    // TODO: Open cookie manager
                    println!("Manage cookies clicked");
                }
            });
            
            ui.add_space(12.0);
            
            // Privacy status indicators
            ui.horizontal(|ui| {
                components::status_indicator(ui, self.tracking_protection, "Tracking Protection");
                ui.add_space(16.0);
                components::status_indicator(ui, self.javascript_enabled, "JavaScript Enabled");
            });
        });
    }
    
    fn render_appearance_settings(&mut self, ui: &mut Ui) {
        components::section_header(ui, "🎨", "Appearance");
        
        components::card_container(ui, |ui| {
            // Theme selection
            ui.label(RichText::new("Theme")
                .strong()
                .color(NeonTheme::PRIMARY_TEXT));
            
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.theme, ThemeOption::Dark, "Dark");
                ui.radio_value(&mut self.theme, ThemeOption::Light, "Light");
                ui.radio_value(&mut self.theme, ThemeOption::Auto, "Auto (System)");
            });
            
            ui.add_space(20.0);
            
            // Font size
            ui.label(RichText::new("Font Size")
                .strong()
                .color(NeonTheme::PRIMARY_TEXT));
            
            ui.horizontal(|ui| {
                ui.add(Slider::new(&mut self.font_size, 8.0..=24.0)
                    .text("px")
                    .show_value(true));
                if ui.button("Reset").clicked() {
                    self.font_size = 14.0;
                }
            });
            
            ui.add_space(20.0);
            
            // UI options
            ui.label(RichText::new("Interface")
                .strong()
                .color(NeonTheme::PRIMARY_TEXT));
            
            ui.checkbox(&mut self.show_bookmarks_bar, "Show bookmarks bar");
        });
    }
    
    fn render_performance_settings(&mut self, ui: &mut Ui) {
        components::section_header(ui, "⚡", "Performance");
        
        components::card_container(ui, |ui| {
            // Cache settings
            ui.label(RichText::new("Cache")
                .strong()
                .color(NeonTheme::PRIMARY_TEXT));
            
            ui.horizontal(|ui| {
                ui.label("Cache size:");
                ui.add(Slider::new(&mut self.cache_size, 10.0..=1000.0)
                    .text("MB")
                    .show_value(true));
            });
            
            ui.horizontal(|ui| {
                if ui.button(RichText::new(format!("{} Clear Cache", NeonIcons::TRASH))
                    .color(NeonTheme::warning_color())).clicked() {
                    // TODO: Clear cache
                    println!("Cache cleared");
                }
                
                ui.add_space(8.0);
                
                ui.label(RichText::new(format!("Current cache size: {:.1} MB", self.cache_size * 0.7))
                    .color(NeonTheme::SECONDARY_TEXT));
            });
            
            ui.add_space(20.0);
            
            // Network settings
            ui.label(RichText::new("Network")
                .strong()
                .color(NeonTheme::PRIMARY_TEXT));
            
            ui.horizontal(|ui| {
                ui.label("Max connections:");
                ui.add(Slider::new(&mut self.max_connections, 1..=20)
                    .show_value(true));
            });
            
            ui.add_space(20.0);
            
            // Hardware acceleration
            ui.label(RichText::new("Graphics")
                .strong()
                .color(NeonTheme::PRIMARY_TEXT));
            
            ui.checkbox(&mut self.enable_hardware_acceleration, 
                "Use hardware acceleration when available");
        });
    }
    
    fn render_advanced_settings(&mut self, ui: &mut Ui) {
        components::section_header(ui, NeonIcons::WRENCH, "Advanced Settings");
        
        components::card_container(ui, |ui| {
            ui.label(RichText::new("⚠️ Warning")
                .strong()
                .color(NeonTheme::warning_color()));
            ui.label("These settings are for advanced users only. Changing them may affect browser stability.");
            
            ui.add_space(20.0);
            
            // Developer options
            ui.label(RichText::new("Developer")
                .strong()
                .color(NeonTheme::PRIMARY_TEXT));
            
            ui.horizontal(|ui| {
                if ui.button(RichText::new(format!("{} Open Developer Console", NeonIcons::CODE))
                    .color(NeonTheme::NEON_CYAN)).clicked() {
                    // TODO: Open dev console
                    println!("Developer console opened");
                }
                
                ui.add_space(8.0);
                
                if ui.button(RichText::new(format!("{} Reset All Settings", NeonIcons::REFRESH))
                    .color(NeonTheme::error_color())).clicked() {
                    // Reset to defaults
                    *self = Self::new();
                    println!("Settings reset to defaults");
                }
            });
            
            ui.add_space(20.0);
            
            // Data management
            ui.label(RichText::new("Data")
                .strong()
                .color(NeonTheme::PRIMARY_TEXT));
            
            ui.horizontal(|ui| {
                if ui.button(RichText::new(format!("{} Export Settings...", NeonIcons::DOWNLOAD))
                    .color(NeonTheme::NEON_CYAN)).clicked() {
                    // TODO: Export settings
                    println!("Settings exported");
                }
                
                ui.add_space(8.0);
                
                if ui.button(RichText::new(format!("{} Import Settings...", NeonIcons::UPLOAD))
                    .color(NeonTheme::NEON_CYAN)).clicked() {
                    // TODO: Import settings
                    println!("Settings import dialog opened");
                }
            });
            
            ui.add_space(20.0);
            
            // Settings info
            ui.separator();
            ui.add_space(12.0);
            
            ui.horizontal(|ui| {
                ui.label(RichText::new("Settings Version:").color(NeonTheme::SECONDARY_TEXT));
                ui.label(RichText::new("1.0.0").color(NeonTheme::PRIMARY_TEXT));
                
                ui.add_space(20.0);
                
                ui.label(RichText::new("Last Modified:").color(NeonTheme::SECONDARY_TEXT));
                ui.label(RichText::new("Today").color(NeonTheme::PRIMARY_TEXT));
            });
        });
    }
}