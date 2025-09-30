use eframe::egui::{Context, Ui};
use crate::pages::{CustomPage, components};

pub struct PerformancePage {
    url: String,
    title: String,
}

impl PerformancePage {
    pub fn new() -> Self {
        Self {
            url: "neon://performance".to_string(),
            title: "Performance".to_string(),
        }
    }
}

impl CustomPage for PerformancePage {
    fn get_url(&self) -> &str {
        &self.url
    }
    
    fn get_title(&self) -> &str {
        &self.title
    }
    
    fn render(&mut self, ui: &mut Ui, _ctx: &Context) {
        components::page_header(
            ui, 
            "Performance Metrics", 
            Some("Monitor and optimize browser performance")
        );
        
        components::card_container(ui, |ui| {
            ui.label("Performance page implementation coming soon...");
        });
    }
}