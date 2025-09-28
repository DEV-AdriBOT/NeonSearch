use eframe::egui;
use crate::ui::BrowserTab;

pub struct NavigationBar;

impl NavigationBar {
    pub fn new() -> Self {
        Self
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, current_tab: Option<&BrowserTab>) -> NavigationAction {
        let mut action = NavigationAction::None;
        
        ui.horizontal(|ui| {
            // Back button
            let can_go_back = current_tab.map_or(false, |tab| tab.can_go_back());
            let back_button = ui.add_enabled(can_go_back, egui::Button::new("←"));
            if back_button.clicked() {
                action = NavigationAction::Back;
            }
            back_button.on_hover_text("Go back");
            
            // Forward button
            let can_go_forward = current_tab.map_or(false, |tab| tab.can_go_forward());
            let forward_button = ui.add_enabled(can_go_forward, egui::Button::new("→"));
            if forward_button.clicked() {
                action = NavigationAction::Forward;
            }
            forward_button.on_hover_text("Go forward");
            
            // Refresh button
            if ui.button("↻").on_hover_text("Reload").clicked() {
                action = NavigationAction::Reload;
            }
            
            // Home button
            if ui.button("🏠").on_hover_text("Home").clicked() {
                action = NavigationAction::Home;
            }
        });
        
        action
    }
}

pub enum NavigationAction {
    None,
    Back,
    Forward,
    Reload,
    Home,
}