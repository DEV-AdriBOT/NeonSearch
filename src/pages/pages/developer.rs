use eframe::egui::{Context, Ui};
use crate::pages::{CustomPage, components};

pub struct DeveloperPage {
    url: String,
    title: String,
}

impl DeveloperPage {
    pub fn new() -> Self {
        Self {
            url: "neon://developer".to_string(),
            title: "Developer Tools".to_string(),
        }
    }
}

impl CustomPage for DeveloperPage {
    fn get_url(&self) -> &str {
        &self.url
    }
    
    fn get_title(&self) -> &str {
        &self.title
    }
    
    fn render(&mut self, ui: &mut Ui, _ctx: &Context) {
        components::page_header(
            ui, 
            "Developer Tools", 
            Some("Debugging tools and browser internals")
        );
        
        components::card_container(ui, |ui| {
            ui.label("Developer tools page implementation coming soon...");
        });
    }
}