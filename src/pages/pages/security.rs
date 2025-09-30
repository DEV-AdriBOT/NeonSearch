use eframe::egui::{Context, Ui};
use crate::pages::{CustomPage, components};

pub struct SecurityPage {
    url: String,
    title: String,
}

impl SecurityPage {
    pub fn new() -> Self {
        Self {
            url: "neon://security".to_string(),
            title: "Security".to_string(),
        }
    }
}

impl CustomPage for SecurityPage {
    fn get_url(&self) -> &str {
        &self.url
    }
    
    fn get_title(&self) -> &str {
        &self.title
    }
    
    fn render(&mut self, ui: &mut Ui, _ctx: &Context) {
        components::page_header(
            ui, 
            "Security Status", 
            Some("Privacy controls and security information")
        );
        
        components::card_container(ui, |ui| {
            ui.label("Security page implementation coming soon...");
        });
    }
}