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
        let color = if is_selected {
            NeonTheme::NEON_CYAN
        } else {
            NeonTheme::SECONDARY_TEXT
        };
        
        let response = ui.horizontal(|ui| {
            ui.label(RichText::new(icon).color(color));
            ui.label(RichText::new(label).color(color));
        }).response;
        
        if response.clicked() {
            self.current_tab = tab;
        }
        
        if is_selected {
            ui.add_space(2.0);
        } else {
            ui.add_space(8.0);
        }
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
                ui.text_edit_singleline(&mut self.downloads_path);
                if ui.button("Browse...").clicked() {
                    // TODO: Open file dialog
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
                if ui.button("Clear Browsing Data...").clicked() {
                    // TODO: Open clear data dialog
                }
                if ui.button("Manage Cookies...").clicked() {
                    // TODO: Open cookie manager
                }
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
                if ui.button("Clear Cache").clicked() {
                    // TODO: Clear cache
                }
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
                if ui.button("Open Developer Console").clicked() {
                    // TODO: Open dev console
                }
                if ui.button("Reset All Settings").clicked() {
                    // TODO: Reset settings dialog
                }
            });
            
            ui.add_space(20.0);
            
            // Data management
            ui.label(RichText::new("Data")
                .strong()
                .color(NeonTheme::PRIMARY_TEXT));
            
            ui.horizontal(|ui| {
                if ui.button("Export Settings...").clicked() {
                    // TODO: Export settings
                }
                if ui.button("Import Settings...").clicked() {
                    // TODO: Import settings
                }
            });
        });
    }
}