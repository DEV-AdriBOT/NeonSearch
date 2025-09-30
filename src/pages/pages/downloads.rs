use eframe::egui::{Context, Ui};
use crate::pages::{CustomPage, components};

pub struct DownloadsPage {
    url: String,
    title: String,
}

impl DownloadsPage {
    pub fn new() -> Self {
        Self {
            url: "neon://downloads".to_string(),
            title: "Downloads".to_string(),
        }
    }
}

impl CustomPage for DownloadsPage {
    fn get_url(&self) -> &str {
        &self.url
    }
    
    fn get_title(&self) -> &str {
        &self.title
    }
    
    fn render(&mut self, ui: &mut Ui, _ctx: &Context) {
        components::page_header(
            ui, 
            "Downloads", 
            Some("View and manage your downloads")
        );
        
        components::card_container(ui, |ui| {
            ui.label("Downloads page implementation coming soon...");
        });
    }
}