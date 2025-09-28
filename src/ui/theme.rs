use eframe::egui::{self, Color32, Rounding, Shadow, Stroke, Style, Visuals, Vec2};

pub struct NeonTheme;

impl NeonTheme {
    // Neon color palette
    pub const NEON_CYAN: Color32 = Color32::from_rgb(0, 255, 255);
    pub const NEON_PURPLE: Color32 = Color32::from_rgb(138, 43, 226);
    pub const NEON_PINK: Color32 = Color32::from_rgb(255, 20, 147);
    pub const NEON_GREEN: Color32 = Color32::from_rgb(57, 255, 20);
    pub const NEON_BLUE: Color32 = Color32::from_rgb(30, 144, 255);
    
    // Dark theme base colors
    pub const DARK_BG: Color32 = Color32::from_rgb(18, 18, 20);
    pub const DARKER_BG: Color32 = Color32::from_rgb(12, 12, 14);
    pub const CARD_BG: Color32 = Color32::from_rgb(28, 28, 32);
    pub const ELEVATED_BG: Color32 = Color32::from_rgb(38, 38, 42);
    
    // Text colors
    pub const PRIMARY_TEXT: Color32 = Color32::from_rgb(240, 240, 245);
    pub const SECONDARY_TEXT: Color32 = Color32::from_rgb(180, 180, 190);
    pub const MUTED_TEXT: Color32 = Color32::from_rgb(120, 120, 130);
    
    // Border and outline colors
    pub const BORDER_COLOR: Color32 = Color32::from_rgb(60, 60, 68);
    pub const FOCUS_BORDER: Color32 = Color32::from_rgb(0, 200, 255);
    pub const ERROR_COLOR: Color32 = Color32::from_rgb(255, 69, 58);
    pub const SUCCESS_COLOR: Color32 = Color32::from_rgb(48, 209, 88);
    pub const WARNING_COLOR: Color32 = Color32::from_rgb(255, 204, 0);

    pub fn apply_to_context(ctx: &egui::Context) {
        ctx.set_style(Self::create_style());
        ctx.set_visuals(Self::create_visuals());
    }

    pub fn create_style() -> Style {
        let mut style = Style::default();
        
        // Spacing and layout
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        style.spacing.menu_margin = egui::Margin::same(8.0);
        style.spacing.indent = 20.0;
        style.spacing.icon_width = 16.0;
        style.spacing.icon_spacing = 6.0;
        style.spacing.tooltip_width = 600.0;
        style.spacing.combo_height = 200.0;
        
        // Window styling
        style.visuals.window_rounding = Rounding::same(12.0);
        style.visuals.window_shadow = Shadow {
            offset: Vec2::new(2.0, 4.0),
            blur: 20.0,
            spread: 0.0,
            color: Color32::from_black_alpha(80),
        };
        style.visuals.popup_shadow = Shadow {
            offset: Vec2::new(1.0, 2.0),
            blur: 12.0,
            spread: 0.0,
            color: Color32::from_black_alpha(60),
        };
        
        // Widget styling
        style.visuals.widgets.noninteractive.bg_fill = Self::CARD_BG;
        style.visuals.widgets.noninteractive.weak_bg_fill = Self::DARKER_BG;
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Self::BORDER_COLOR);
        style.visuals.widgets.noninteractive.rounding = Rounding::same(8.0);
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Self::PRIMARY_TEXT);
        
        // Interactive widgets (buttons, etc.)
        style.visuals.widgets.inactive.bg_fill = Self::ELEVATED_BG;
        style.visuals.widgets.inactive.weak_bg_fill = Self::CARD_BG;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Self::BORDER_COLOR);
        style.visuals.widgets.inactive.rounding = Rounding::same(8.0);
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Self::PRIMARY_TEXT);
        
        // Hovered widgets
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(48, 48, 55);
        style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(40, 40, 45);
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Self::NEON_CYAN);
        style.visuals.widgets.hovered.rounding = Rounding::same(8.0);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Self::NEON_CYAN);
        
        // Active widgets (pressed)
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(55, 55, 65);
        style.visuals.widgets.active.weak_bg_fill = Color32::from_rgb(45, 45, 50);
        style.visuals.widgets.active.bg_stroke = Stroke::new(2.0, Self::NEON_PURPLE);
        style.visuals.widgets.active.rounding = Rounding::same(8.0);
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, Self::NEON_PURPLE);
        
        // Open widgets (dropdowns, etc.)
        style.visuals.widgets.open.bg_fill = Color32::from_rgb(35, 35, 40);
        style.visuals.widgets.open.weak_bg_fill = Self::CARD_BG;
        style.visuals.widgets.open.bg_stroke = Stroke::new(1.0, Self::FOCUS_BORDER);
        style.visuals.widgets.open.rounding = Rounding::same(8.0);
        style.visuals.widgets.open.fg_stroke = Stroke::new(1.0, Self::FOCUS_BORDER);
        
        style
    }

    pub fn create_visuals() -> Visuals {
        let mut visuals = Visuals::dark();
        
        // Main background and panel colors
        visuals.panel_fill = Self::DARK_BG;
        visuals.window_fill = Self::CARD_BG;
        visuals.extreme_bg_color = Self::DARKER_BG;
        visuals.faint_bg_color = Color32::from_rgb(25, 25, 28);
        
        // Text colors
        visuals.override_text_color = Some(Self::PRIMARY_TEXT);
        // Note: weak_text_color and strong_text_color are methods in egui 0.29
        
        // Selection and highlighting
        visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(0, 200, 255, 40);
        visuals.selection.stroke = Stroke::new(1.0, Self::NEON_CYAN);
        
        // Hyperlink colors
        visuals.hyperlink_color = Self::NEON_BLUE;
        
        // Warning and error colors
        visuals.warn_fg_color = Self::WARNING_COLOR;
        visuals.error_fg_color = Self::ERROR_COLOR;
        
        // Window and shadow effects
        visuals.window_rounding = Rounding::same(12.0);
        visuals.menu_rounding = Rounding::same(8.0);
        
        visuals.window_shadow = Shadow {
            offset: Vec2::new(2.0, 4.0),
            blur: 20.0,
            spread: 0.0,
            color: Color32::from_black_alpha(100),
        };
        
        visuals.popup_shadow = Shadow {
            offset: Vec2::new(1.0, 2.0),
            blur: 12.0,
            spread: 0.0,
            color: Color32::from_black_alpha(80),
        };
        
        // Widget colors (using widgets.active instead of deprecated fields)
        visuals.widgets.active.bg_fill = Self::NEON_PURPLE;
        visuals.widgets.hovered.bg_fill = Self::NEON_CYAN;
        
        // Separator colors
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Self::BORDER_COLOR);
        
        visuals
    }

    // Helper methods for custom widget styling
    pub fn button_primary() -> egui::Button<'static> {
        egui::Button::new("").fill(Self::NEON_CYAN).stroke(Stroke::new(0.0, Color32::TRANSPARENT))
    }

    pub fn button_secondary() -> egui::Button<'static> {
        egui::Button::new("").fill(Self::ELEVATED_BG).stroke(Stroke::new(1.0, Self::BORDER_COLOR))
    }

    pub fn button_danger() -> egui::Button<'static> {
        egui::Button::new("").fill(Self::ERROR_COLOR).stroke(Stroke::new(0.0, Color32::TRANSPARENT))
    }

    pub fn gradient_background() -> egui::Color32 {
        // For now, return a solid color. In future versions we could implement gradients
        Self::DARK_BG
    }

    pub fn add_glow_effect(ui: &mut egui::Ui, rect: egui::Rect, color: Color32) {
        // Add a subtle glow effect around elements
        let painter = ui.painter();
        let glow_rect = rect.expand(2.0);
        painter.rect_filled(glow_rect, Rounding::same(10.0), Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 30));
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
}
