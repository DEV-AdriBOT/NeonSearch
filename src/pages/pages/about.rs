use eframe::egui::{Context, RichText, Ui};
use crate::pages::{CustomPage, components};
use crate::ui::theme::NeonTheme;
use crate::ui::icons::NeonIcons;

pub struct AboutPage {
    url: String,
    title: String,
}

impl AboutPage {
    pub fn new() -> Self {
        Self {
            url: "neon://about".to_string(),
            title: "About NeonSearch".to_string(),
        }
    }
}

impl CustomPage for AboutPage {
    fn get_url(&self) -> &str {
        &self.url
    }
    
    fn get_title(&self) -> &str {
        &self.title
    }
    
    fn render(&mut self, ui: &mut Ui, _ctx: &Context) {
        components::page_header(
            ui, 
            "About NeonSearch", 
            Some("Built entirely from scratch in Rust for speed, security, and innovation")
        );
        
        ui.vertical_centered(|ui| {
            // Browser logo/icon placeholder
            ui.add_space(20.0);
            ui.label(RichText::new("⚡")
                .size(80.0)
                .color(NeonTheme::NEON_CYAN));
            
            ui.add_space(20.0);
            
            // Version information
            components::card_container(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("NeonSearch Browser")
                        .size(28.0)
                        .strong()
                        .color(NeonTheme::PRIMARY_TEXT));
                    
                    ui.add_space(8.0);
                    ui.label(RichText::new("Version 0.2.0")
                        .size(18.0)
                        .color(NeonTheme::SECONDARY_TEXT));
                    
                    ui.add_space(16.0);
                    ui.label(RichText::new("Built with ❤️ by NeonDev™")
                        .size(16.0)
                        .color(NeonTheme::SECONDARY_TEXT));
                });
            });
            
            ui.add_space(30.0);
        });
        
        // Features section
        components::section_header(ui, "⭐", "Key Features");
        
        components::card_container(ui, |ui| {
            let features = [
                ("🚀", "Built from Scratch", "No Chromium or Gecko - pure Rust implementation"),
                ("⚡", "Lightning Fast", "Optimized for speed with advanced rendering pipeline"),
                ("🔒", "Privacy First", "Your data stays private with built-in security"),
                ("🎨", "Modern Design", "Beautiful neon-themed interface with dark mode"),
                ("🧠", "Smart Rendering", "Adaptive rendering for websites of any size"),
                ("🌐", "Web Standards", "HTML5, CSS3, and JavaScript support"),
            ];
            
            ui.columns(2, |columns| {
                for (i, (icon, title, description)) in features.iter().enumerate() {
                    let col = i % 2;
                    columns[col].horizontal(|ui| {
                        ui.label(RichText::new(*icon).size(24.0));
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(*title)
                                .size(16.0)
                                .strong()
                                .color(NeonTheme::PRIMARY_TEXT));
                            ui.label(RichText::new(*description)
                                .size(14.0)
                                .color(NeonTheme::SECONDARY_TEXT));
                        });
                    });
                    columns[col].add_space(16.0);
                }
            });
        });
        
        ui.add_space(20.0);
        
        // System information
        components::section_header(ui, "💻", "System Information");
        
        components::card_container(ui, |ui| {
            ui.columns(2, |columns| {
                // Left column - Browser info
                columns[0].vertical(|ui| {
                    ui.label(RichText::new("Browser Engine")
                        .strong()
                        .color(NeonTheme::PRIMARY_TEXT));
                    ui.label("NeonEngine v0.2.0");
                    
                    ui.add_space(12.0);
                    ui.label(RichText::new("JavaScript Engine")
                        .strong()
                        .color(NeonTheme::PRIMARY_TEXT));
                    ui.label("NeonJS (Custom Implementation)");
                    
                    ui.add_space(12.0);
                    ui.label(RichText::new("User Agent")
                        .strong()
                        .color(NeonTheme::PRIMARY_TEXT));
                    ui.label("NeonSearch/0.2.0 (Rust/Native)");
                });
                
                // Right column - System info
                columns[1].vertical(|ui| {
                    ui.label(RichText::new("Operating System")
                        .strong()
                        .color(NeonTheme::PRIMARY_TEXT));
                    let os = std::env::consts::OS;
                    let arch = std::env::consts::ARCH;
                    ui.label(format!("{} ({})", os, arch));
                    
                    ui.add_space(12.0);
                    ui.label(RichText::new("Rust Version")
                        .strong()
                        .color(NeonTheme::PRIMARY_TEXT));
                    ui.label("rustc 1.75+ (Rust 2021 Edition)");
                    
                    ui.add_space(12.0);
                    ui.label(RichText::new("Build Type")
                        .strong()
                        .color(NeonTheme::PRIMARY_TEXT));
                    if cfg!(debug_assertions) {
                        ui.label("Debug");
                    } else {
                        ui.label("Release");
                    }
                });
            });
        });
        
        ui.add_space(20.0);
        
        // Credits and legal
        components::section_header(ui, "❤️", "Credits & Legal");
        
        components::card_container(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Open Source Libraries")
                    .size(16.0)
                    .strong()
                    .color(NeonTheme::PRIMARY_TEXT));
                
                ui.add_space(12.0);
                
                let libraries = [
                    "egui - Immediate mode GUI framework",
                    "tokio - Asynchronous runtime",
                    "rustls - TLS implementation", 
                    "reqwest - HTTP client library",
                    "serde - Serialization framework",
                ];
                
                for lib in libraries.iter() {
                    ui.label(RichText::new(*lib)
                        .size(14.0)
                        .color(NeonTheme::SECONDARY_TEXT));
                }
                
                ui.add_space(20.0);
                
                ui.label(RichText::new("© 2025 NeonDev™. All rights reserved.")
                    .size(14.0)
                    .color(NeonTheme::SECONDARY_TEXT));
                
                ui.add_space(8.0);
                
                ui.horizontal_centered(|ui| {
                    if ui.link(RichText::new("License")
                        .color(NeonTheme::accent_color())).clicked() {
                        // TODO: Open license page
                    }
                    ui.label(" • ");
                    if ui.link(RichText::new("Privacy Policy")
                        .color(NeonTheme::accent_color())).clicked() {
                        // TODO: Open privacy policy
                    }
                    ui.label(" • ");
                    if ui.link(RichText::new("GitHub")
                        .color(NeonTheme::accent_color())).clicked() {
                        // TODO: Open GitHub repository
                    }
                });
            });
        });
        
        ui.add_space(40.0);
    }
}