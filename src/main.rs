#![allow(dead_code)] // Allow dead code for future engine components
pub mod networking;
pub mod ui;
pub mod engine;
pub mod js;
pub mod security;

use eframe::egui;
use log::info;

fn main() -> Result<(), eframe::Error> {
    // Initialize logger
    env_logger::init();
    info!("Starting NeonSearch Browser by NeonDev™");

    // Test JavaScript engine on startup
    println!("\n🚀 NeonSearch v0.2.0 - JavaScript Engine Test");
    js::test::test_js_engine();
    js::test::test_html_with_js();
    println!(""); // Empty line for readability

    // macOS beta compatibility workarounds
    #[cfg(target_os = "macos")]
    {
        // Set environment variables to help with macOS beta compatibility
        std::env::set_var("OBJC_DISABLE_INITIALIZE_FORK_SAFETY", "YES");
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("NeonSearch - Web Browser by NeonDev™")
            .with_min_inner_size([800.0, 600.0])
            .with_resizable(true)
            .with_decorations(true),
        hardware_acceleration: eframe::HardwareAcceleration::Required,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "NeonSearch",
        options,
        Box::new(|cc| {
            // Configure egui style for NeonSearch theme
            setup_custom_style(&cc.egui_ctx);
            Ok(Box::new(ui::NeonSearchApp::new(cc)))
        }),
    )
}

fn setup_custom_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // NeonSearch custom theme - Dark mode with neon accents
    style.visuals.dark_mode = true;
    style.visuals.override_text_color = Some(egui::Color32::from_rgb(220, 220, 220));
    style.visuals.window_fill = egui::Color32::from_rgb(30, 30, 35);
    style.visuals.panel_fill = egui::Color32::from_rgb(25, 25, 30);
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(0, 200, 255); // Neon blue
    style.visuals.hyperlink_color = egui::Color32::from_rgb(0, 255, 200); // Neon cyan
    
    ctx.set_style(style);
}
