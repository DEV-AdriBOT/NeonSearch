use eframe::egui::{Context, Ui};
use crate::pages::{CustomPage, components};

pub struct HistoryPage {
    url: String,
    title: String,
}

impl HistoryPage {
    pub fn new() -> Self {
        Self {
            url: "neon://history".to_string(),
            title: "History".to_string(),
        }
    }
}

impl CustomPage for HistoryPage {
    fn get_url(&self) -> &str {
        &self.url
    }
    
    fn get_title(&self) -> &str {
        &self.title
    }
    
    fn render(&mut self, ui: &mut Ui, _ctx: &Context) {
        components::page_header(
            ui, 
            "Browsing History", 
            Some("View and manage your browsing history")
        );
        
        components::card_container(ui, |ui| {
            ui.label("History page implementation coming soon...");
        });
    }
}