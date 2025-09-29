use eframe::egui;
use crate::ui::{BrowserTab, NeonIcons, NeonTheme};

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
            let back_button = ui.add_enabled(
                can_go_back, 
                egui::Button::new(
                    egui::RichText::new(NeonIcons::ARROW_LEFT)
                        .size(16.0)
                        .color(if can_go_back { NeonTheme::PRIMARY_TEXT } else { NeonTheme::MUTED_TEXT })
                )
            );
            if back_button.clicked() {
                action = NavigationAction::Back;
            }
            back_button.on_hover_text("Go back");
            
            // Forward button
            let can_go_forward = current_tab.map_or(false, |tab| tab.can_go_forward());
            let forward_button = ui.add_enabled(
                can_go_forward, 
                egui::Button::new(
                    egui::RichText::new(NeonIcons::ARROW_RIGHT)
                        .size(16.0)
                        .color(if can_go_forward { NeonTheme::PRIMARY_TEXT } else { NeonTheme::MUTED_TEXT })
                )
            );
            if forward_button.clicked() {
                action = NavigationAction::Forward;
            }
            forward_button.on_hover_text("Go forward");
            
            // Refresh button
            let refresh_button = ui.button(
                egui::RichText::new(NeonIcons::ARROW_CLOCKWISE)
                    .size(16.0)
                    .color(NeonTheme::PRIMARY_TEXT)
            );
            if refresh_button.clicked() {
                action = NavigationAction::Reload;
            }
            refresh_button.on_hover_text("Reload");
            
            // Home button
            let home_button = ui.button(
                egui::RichText::new(NeonIcons::HOUSE)
                    .size(16.0)
                    .color(NeonTheme::NEON_BLUE)
            );
            if home_button.clicked() {
                action = NavigationAction::Home;
            }
            home_button.on_hover_text("Home");
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