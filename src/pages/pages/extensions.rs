use eframe::egui::{Context, Ui};
use crate::pages::{CustomPage, components};

pub struct ExtensionsPage {
    url: String,
    title: String,
}

impl ExtensionsPage {
    pub fn new() -> Self {
        Self {
            url: "neon://extensions".to_string(),
            title: "Extensions".to_string(),
        }
    }
}

impl CustomPage for ExtensionsPage {
    fn get_url(&self) -> &str {
        &self.url
    }
    
    fn get_title(&self) -> &str {
        &self.title
    }
    
    fn render(&mut self, ui: &mut Ui, _ctx: &Context) {
        components::page_header(
            ui, 
            "Extensions", 
            Some("Manage browser extensions and add-ons")
        );
        
        components::card_container(ui, |ui| {
            ui.label("Extensions page implementation coming soon...");
        });
    }
}