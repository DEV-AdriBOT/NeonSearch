use eframe::egui::{Context, Ui};
use std::collections::HashMap;

/// Trait that all custom pages must implement
pub trait CustomPage {
    /// Returns the protocol and path this page handles (e.g., "neon://settings")
    fn get_url(&self) -> &str;
    
    /// Returns the page title shown in the tab
    fn get_title(&self) -> &str;
    
    /// Renders the page content
    fn render(&mut self, ui: &mut Ui, ctx: &Context);
    
    /// Called when the page is first loaded
    fn on_load(&mut self) {}
    
    /// Called when the page is being unloaded
    fn on_unload(&mut self) {}
    
    /// Returns true if the page can handle the given URL
    fn can_handle(&self, url: &str) -> bool {
        url == self.get_url() || url.starts_with(&format!("{}?", self.get_url()))
    }
}

/// Router for custom pages
pub struct PageRouter {
    pages: HashMap<String, Box<dyn CustomPage>>,
}

impl PageRouter {
    pub fn new() -> Self {
        let mut router = Self {
            pages: HashMap::new(),
        };
        
        // Register all pages
        router.register_page(Box::new(pages::AboutPage::new()));
        router.register_page(Box::new(pages::SettingsPage::new()));
        router.register_page(Box::new(pages::HistoryPage::new()));
        router.register_page(Box::new(pages::BookmarksPage::new()));
        router.register_page(Box::new(pages::DeveloperPage::new()));
        router.register_page(Box::new(pages::PerformancePage::new()));
        router.register_page(Box::new(pages::SecurityPage::new()));
        router.register_page(Box::new(pages::DownloadsPage::new()));
        router.register_page(Box::new(pages::ExtensionsPage::new()));
        router.register_page(Box::new(pages::ExperimentsPage::new()));
        
        router
    }
    
    pub fn register_page(&mut self, page: Box<dyn CustomPage>) {
        let url = page.get_url().to_string();
        self.pages.insert(url, page);
    }
    
    pub fn can_handle(&self, url: &str) -> bool {
        // Handle neon:// protocol and about: pages
        if url.starts_with("neon://") || url.starts_with("about:") {
            return self.pages.values().any(|page| page.can_handle(url));
        }
        false
    }
    
    pub fn get_page_title(&self, url: &str) -> Option<String> {
        self.pages.values()
            .find(|page| page.can_handle(url))
            .map(|page| page.get_title().to_string())
    }
    
    pub fn render_page(&mut self, url: &str, ui: &mut Ui, ctx: &Context) -> bool {
        if let Some(page) = self.pages.values_mut().find(|page| page.can_handle(url)) {
            page.render(ui, ctx);
            true
        } else {
            false
        }
    }
    
    pub fn on_page_load(&mut self, url: &str) {
        if let Some(page) = self.pages.values_mut().find(|page| page.can_handle(url)) {
            page.on_load();
        }
    }
    
    pub fn on_page_unload(&mut self, url: &str) {
        if let Some(page) = self.pages.values_mut().find(|page| page.can_handle(url)) {
            page.on_unload();
        }
    }
}

/// Base components for pages
pub mod components {
    use eframe::egui::{RichText, Ui};
    use crate::ui::theme::NeonTheme;
    use crate::ui::icons::NeonIcons;

    /// Standard page header with title and optional description
    pub fn page_header(ui: &mut Ui, title: &str, description: Option<&str>) {
        ui.add_space(20.0);
        
        // Main title
        ui.heading(RichText::new(title)
            .size(32.0)
            .color(NeonTheme::PRIMARY_TEXT));
        
        if let Some(desc) = description {
            ui.add_space(8.0);
            ui.label(RichText::new(desc)
                .size(16.0)
                .color(NeonTheme::SECONDARY_TEXT));
        }
        
        ui.add_space(20.0);
        ui.separator();
        ui.add_space(20.0);
    }
    
    /// Standard section header with icon
    pub fn section_header(ui: &mut Ui, icon: &str, title: &str) {
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(icon).size(20.0).color(NeonTheme::NEON_CYAN));
            ui.label(RichText::new(title)
                .size(20.0)
                .strong()
                .color(NeonTheme::PRIMARY_TEXT));
        });
        ui.add_space(8.0);
    }
    
    /// Card container for content sections
    pub fn card_container<R>(ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
        let card_color = NeonTheme::CARD_BG;
        
        eframe::egui::Frame::none()
            .fill(card_color)
            .rounding(12.0)
            .inner_margin(20.0)
            .show(ui, content)
            .inner
    }
    
    /// Navigation breadcrumb
    pub fn breadcrumb(ui: &mut Ui, items: &[&str]) {
        ui.horizontal(|ui| {
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    ui.label(RichText::new(NeonIcons::ARROW_RIGHT)
                        .color(NeonTheme::SECONDARY_TEXT));
                    ui.add_space(8.0);
                }
                
                let color = if i == items.len() - 1 {
                    NeonTheme::PRIMARY_TEXT
                } else {
                    NeonTheme::SECONDARY_TEXT
                };
                
                ui.label(RichText::new(*item).color(color));
                ui.add_space(8.0);
            }
        });
        ui.add_space(12.0);
    }
    
    /// Status indicator
    pub fn status_indicator(ui: &mut Ui, is_good: bool, text: &str) {
        ui.horizontal(|ui| {
            let (icon, color) = if is_good {
                (NeonIcons::CHECK_CIRCLE, NeonTheme::success_color())
            } else {
                (NeonIcons::WARNING, NeonTheme::warning_color())
            };
            
            ui.label(RichText::new(icon).color(color));
            ui.label(RichText::new(text).color(NeonTheme::PRIMARY_TEXT));
        });
    }
}

// Page modules
pub mod pages;

pub use pages::*;