use eframe::egui::{Context, Ui};
use crate::pages::{CustomPage, components};

pub struct ExperimentsPage {
    url: String,
    title: String,
}

impl ExperimentsPage {
    pub fn new() -> Self {
        Self {
            url: "neon://experiments".to_string(),
            title: "Experiments".to_string(),
        }
    }
}

impl CustomPage for ExperimentsPage {
    fn get_url(&self) -> &str {
        &self.url
    }
    
    fn get_title(&self) -> &str {
        &self.title
    }
    
    fn render(&mut self, ui: &mut Ui, _ctx: &Context) {
        components::page_header(
            ui, 
            "Experimental Features", 
            Some("Enable beta features and experimental settings")
        );
        
        components::card_container(ui, |ui| {
            ui.label("Experiments page implementation coming soon...");
        });
    }
}