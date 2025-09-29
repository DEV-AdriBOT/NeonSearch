use eframe::egui::{self, Color32, Rounding, Shadow, Stroke, Style, Visuals, Vec2};

pub struct NeonTheme;

impl NeonTheme {
    // Modern Neon color palette - more sophisticated and vibrant
    pub const NEON_CYAN: Color32 = Color32::from_rgb(0, 255, 255);       // Electric cyan
    pub const NEON_PURPLE: Color32 = Color32::from_rgb(138, 43, 226);     // Electric purple  
    pub const NEON_PINK: Color32 = Color32::from_rgb(255, 20, 147);       // Hot pink
    pub const NEON_GREEN: Color32 = Color32::from_rgb(57, 255, 20);       // Electric lime
    pub const NEON_BLUE: Color32 = Color32::from_rgb(30, 144, 255);       // Dodger blue
    pub const NEON_ORANGE: Color32 = Color32::from_rgb(255, 140, 0);      // Dark orange
    pub const NEON_GOLD: Color32 = Color32::from_rgb(255, 215, 0);        // Gold accent
    
    // Modern dark theme with deeper contrasts and warmer tones
    pub const DARK_BG: Color32 = Color32::from_rgb(13, 17, 23);           // GitHub dark
    pub const DARKER_BG: Color32 = Color32::from_rgb(8, 12, 16);          // Deeper dark
    pub const CARD_BG: Color32 = Color32::from_rgb(21, 32, 43);           // Card background
    pub const ELEVATED_BG: Color32 = Color32::from_rgb(33, 41, 54);       // Elevated surfaces
    pub const PANEL_BG: Color32 = Color32::from_rgb(22, 27, 34);          // Panel background
    pub const SURFACE_BG: Color32 = Color32::from_rgb(30, 35, 42);        // Surface background
    
    // Enhanced text colors with better hierarchy
    pub const PRIMARY_TEXT: Color32 = Color32::from_rgb(248, 250, 252);   // Almost white
    pub const SECONDARY_TEXT: Color32 = Color32::from_rgb(201, 209, 217); // Cool gray
    pub const MUTED_TEXT: Color32 = Color32::from_rgb(139, 148, 158);     // Muted gray
    pub const HINT_TEXT: Color32 = Color32::from_rgb(87, 96, 106);        // Hint text
    pub const ACCENT_TEXT: Color32 = Color32::from_rgb(88, 166, 255);     // Accent blue
    
    // Enhanced border and outline colors with more sophistication
    pub const BORDER_COLOR: Color32 = Color32::from_rgb(48, 54, 61);      // Subtle border
    pub const FOCUS_BORDER: Color32 = Color32::from_rgb(0, 122, 255);     // Focus indicator
    pub const HOVER_BORDER: Color32 = Color32::from_rgb(88, 166, 255);    // Hover state
    pub const ACTIVE_BORDER: Color32 = Color32::from_rgb(0, 255, 255);    // Active state
    
    // Status colors with modern design system approach
    pub const ERROR_COLOR: Color32 = Color32::from_rgb(255, 69, 58);      // iOS red
    pub const SUCCESS_COLOR: Color32 = Color32::from_rgb(52, 199, 89);    // iOS green
    pub const WARNING_COLOR: Color32 = Color32::from_rgb(255, 204, 0);    // iOS yellow
    pub const INFO_COLOR: Color32 = Color32::from_rgb(90, 200, 250);      // Info blue
    
    // Interactive state colors with smooth transitions
    pub const BUTTON_IDLE: Color32 = Color32::from_rgb(48, 54, 61);       // Button at rest
    pub const BUTTON_HOVER: Color32 = Color32::from_rgb(65, 72, 80);      // Button hovered
    pub const BUTTON_ACTIVE: Color32 = Color32::from_rgb(82, 90, 99);     // Button pressed
    pub const BUTTON_PRIMARY: Color32 = Color32::from_rgb(0, 122, 255);   // Primary button
    pub const BUTTON_PRIMARY_HOVER: Color32 = Color32::from_rgb(10, 132, 255); // Primary hover

    pub fn apply_to_context(ctx: &egui::Context) {
        ctx.set_style(Self::create_style());
        ctx.set_visuals(Self::create_visuals());
    }

    pub fn create_style() -> Style {
        let mut style = Style::default();
        
        // Modern spacing and layout with better proportions
        style.spacing.item_spacing = egui::vec2(12.0, 10.0);
        style.spacing.button_padding = egui::vec2(20.0, 12.0);
        style.spacing.menu_margin = egui::Margin::same(12.0);
        style.spacing.indent = 28.0;
        style.spacing.icon_width = 20.0;
        style.spacing.icon_spacing = 10.0;
        style.spacing.tooltip_width = 800.0;
        style.spacing.combo_height = 300.0;
        style.spacing.slider_width = 180.0;
        style.spacing.text_edit_width = 280.0;
        
        // Modern window styling with sophisticated shadows
        style.visuals.window_rounding = Rounding::same(20.0);
        style.visuals.window_shadow = Shadow {
            offset: Vec2::new(0.0, 12.0),
            blur: 40.0,
            spread: 0.0,
            color: Color32::from_black_alpha(160),
        };
        style.visuals.popup_shadow = Shadow {
            offset: Vec2::new(0.0, 6.0),
            blur: 24.0,
            spread: 0.0,
            color: Color32::from_black_alpha(120),
        };
        
        // Enhanced widget styling with modern design principles
        
        // Non-interactive widgets (labels, separators, etc.)
        style.visuals.widgets.noninteractive.bg_fill = Self::SURFACE_BG;
        style.visuals.widgets.noninteractive.weak_bg_fill = Self::DARK_BG;
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Self::BORDER_COLOR);
        style.visuals.widgets.noninteractive.rounding = Rounding::same(12.0);
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Self::PRIMARY_TEXT);
        
        // Inactive widgets (buttons, inputs at rest)
        style.visuals.widgets.inactive.bg_fill = Self::BUTTON_IDLE;
        style.visuals.widgets.inactive.weak_bg_fill = Self::CARD_BG;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Self::BORDER_COLOR);
        style.visuals.widgets.inactive.rounding = Rounding::same(12.0);
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Self::SECONDARY_TEXT);
        
        // Hovered widgets
        style.visuals.widgets.hovered.bg_fill = Self::BUTTON_HOVER;
        style.visuals.widgets.hovered.weak_bg_fill = Self::ELEVATED_BG;
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(2.0, Self::HOVER_BORDER);
        style.visuals.widgets.hovered.rounding = Rounding::same(12.0);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Self::PRIMARY_TEXT);
        
        // Active widgets (pressed/clicked)
        style.visuals.widgets.active.bg_fill = Self::BUTTON_ACTIVE;
        style.visuals.widgets.active.weak_bg_fill = Self::ELEVATED_BG;
        style.visuals.widgets.active.bg_stroke = Stroke::new(2.0, Self::ACTIVE_BORDER);
        style.visuals.widgets.active.rounding = Rounding::same(12.0);
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, Self::NEON_CYAN);
        
        // Open widgets (dropdowns, menus, etc.)
        style.visuals.widgets.open.bg_fill = Self::ELEVATED_BG;
        style.visuals.widgets.open.weak_bg_fill = Self::CARD_BG;
        style.visuals.widgets.open.bg_stroke = Stroke::new(2.0, Self::FOCUS_BORDER);
        style.visuals.widgets.open.rounding = Rounding::same(12.0);
        style.visuals.widgets.open.fg_stroke = Stroke::new(1.0, Self::FOCUS_BORDER);
        
        style
    }

    pub fn create_visuals() -> Visuals {
        let mut visuals = Visuals::dark();
        
        // Main background and panel colors with sophisticated hierarchy
        visuals.panel_fill = Self::DARK_BG;
        visuals.window_fill = Self::CARD_BG;
        visuals.extreme_bg_color = Self::DARKER_BG;
        visuals.faint_bg_color = Self::SURFACE_BG;
        
        // Text colors with enhanced contrast
        visuals.override_text_color = Some(Self::PRIMARY_TEXT);
        
        // Selection and highlighting with modern blue
        visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(0, 122, 255, 50);
        visuals.selection.stroke = Stroke::new(1.5, Self::FOCUS_BORDER);
        
        // Hyperlink colors using accent blue
        visuals.hyperlink_color = Self::ACCENT_TEXT;
        
        // Warning and error colors
        visuals.warn_fg_color = Self::WARNING_COLOR;
        visuals.error_fg_color = Self::ERROR_COLOR;
        
        // Window and shadow effects with modern aesthetics
        visuals.window_rounding = Rounding::same(16.0);
        visuals.menu_rounding = Rounding::same(12.0);
        
        visuals.window_shadow = Shadow {
            offset: Vec2::new(0.0, 8.0),
            blur: 32.0,
            spread: 0.0,
            color: Color32::from_black_alpha(140),
        };
        
        visuals.popup_shadow = Shadow {
            offset: Vec2::new(0.0, 4.0),
            blur: 16.0,
            spread: 0.0,
            color: Color32::from_black_alpha(100),
        };
        
        // Widget colors using our modern color scheme
        visuals.widgets.active.bg_fill = Self::BUTTON_PRIMARY;
        visuals.widgets.hovered.bg_fill = Self::BUTTON_HOVER;
        
        // Separator colors
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Self::BORDER_COLOR);
        
        visuals
    }

    // Helper methods for modern widget styling
    pub fn button_primary() -> egui::Button<'static> {
        egui::Button::new("")
            .fill(Self::BUTTON_PRIMARY)
            .stroke(Stroke::new(0.0, Color32::TRANSPARENT))
            .rounding(Rounding::same(12.0))
    }

    pub fn button_secondary() -> egui::Button<'static> {
        egui::Button::new("")
            .fill(Self::ELEVATED_BG)
            .stroke(Stroke::new(1.5, Self::BORDER_COLOR))
            .rounding(Rounding::same(12.0))
    }

    pub fn button_danger() -> egui::Button<'static> {
        egui::Button::new("")
            .fill(Self::ERROR_COLOR)
            .stroke(Stroke::new(0.0, Color32::TRANSPARENT))
            .rounding(Rounding::same(12.0))
    }

    pub fn button_success() -> egui::Button<'static> {
        egui::Button::new("")
            .fill(Self::SUCCESS_COLOR)
            .stroke(Stroke::new(0.0, Color32::TRANSPARENT))
            .rounding(Rounding::same(12.0))
    }

    // Modern gradient backgrounds
    pub fn create_modern_background(ui: &mut egui::Ui, rect: egui::Rect) {
        let painter = ui.painter();
        
        // Create a subtle gradient effect using layered rectangles
        painter.rect_filled(rect, Rounding::same(16.0), Self::DARK_BG);
        
        // Add a subtle overlay for depth
        let overlay_rect = egui::Rect::from_min_size(
            rect.min + egui::Vec2::new(0.0, 0.0),
            rect.size()
        );
        painter.rect_filled(
            overlay_rect,
            Rounding::same(16.0),
            Color32::from_rgba_unmultiplied(0, 122, 255, 8)
        );
    }

    // Add sophisticated glow effects
    pub fn add_glow_effect(ui: &mut egui::Ui, rect: egui::Rect, color: Color32, intensity: f32) {
        let painter = ui.painter();
        let blur_size = 8.0 * intensity;
        
        // Create multiple layers for a smooth glow effect
        for i in 1..=3 {
            let layer_alpha = (30.0 * intensity / i as f32) as u8;
            let glow_rect = rect.expand(blur_size * i as f32);
            painter.rect_filled(
                glow_rect,
                Rounding::same(16.0 + blur_size),
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), layer_alpha)
            );
        }
    }

    // Modern card-style containers
    pub fn create_card(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
        let available_rect = ui.available_rect_before_wrap();
        
        egui::Frame::none()
            .fill(Self::CARD_BG)
            .stroke(Stroke::new(1.0, Self::BORDER_COLOR))
            .rounding(Rounding::same(16.0))
            .shadow(Shadow {
                offset: Vec2::new(0.0, 4.0),
                blur: 12.0,
                spread: 0.0,
                color: Color32::from_black_alpha(40),
            })
            .inner_margin(egui::Margin::same(16.0))
            .show(ui, add_contents);
    }

    // Modern input field styling
    pub fn style_text_edit(response: &egui::Response, ui: &mut egui::Ui) {
        if response.hovered() {
            // Add subtle glow on hover
            Self::add_glow_effect(ui, response.rect, Self::FOCUS_BORDER, 0.3);
        }
        if response.has_focus() {
            // Add stronger glow on focus
            Self::add_glow_effect(ui, response.rect, Self::NEON_CYAN, 0.5);
        }
    }

    pub fn gradient_background() -> egui::Color32 {
        Self::DARK_BG
    }

    pub fn neon_text_color() -> Color32 {
        Self::NEON_CYAN
    }

    pub fn success_color() -> Color32 {
        Self::SUCCESS_COLOR
    }

    pub fn error_color() -> Color32 {
        Self::ERROR_COLOR
    }

    pub fn warning_color() -> Color32 {
        Self::WARNING_COLOR
    }

    pub fn accent_color() -> Color32 {
        Self::ACCENT_TEXT
    }
}
