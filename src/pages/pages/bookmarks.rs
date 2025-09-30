use eframe::egui::{Context, Ui};
use crate::pages::{CustomPage, components};

pub struct BookmarksPage {
    url: String,
    title: String,
}

impl BookmarksPage {
    pub fn new() -> Self {
        Self {
            url: "neon://bookmarks".to_string(),
            title: "Bookmarks".to_string(),
        }
    }
}

impl CustomPage for BookmarksPage {
    fn get_url(&self) -> &str {
        &self.url
    }
    
    fn get_title(&self) -> &str {
        &self.title
    }
    
    fn render(&mut self, ui: &mut Ui, _ctx: &Context) {
        components::page_header(
            ui, 
            "Bookmarks", 
            Some("Organize and manage your saved bookmarks")
        );
        
        components::card_container(ui, |ui| {
            ui.label("Bookmarks page implementation coming soon...");
        });
    }
}