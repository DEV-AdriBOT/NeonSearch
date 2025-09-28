// Rendering engine for painting to screen

use crate::engine::layout::{LayoutBox, BoxType, Rect};
use crate::engine::css_parser::{Color, Value};
use eframe::egui;

pub struct DisplayList {
    pub items: Vec<DisplayItem>,
}

#[derive(Debug, Clone)]
pub enum DisplayItem {
    SolidColor {
        color: Color,
        rect: Rect,
    },
    Text {
        text: String,
        rect: Rect,
        color: Color,
    },
    Border {
        color: Color,
        rect: Rect,
        width: f32,
    },
}

impl DisplayList {
    pub fn new() -> Self {
        DisplayList { items: Vec::new() }
    }
    
    pub fn render(&self, ui: &mut egui::Ui) {
        for item in &self.items {
            match item {
                DisplayItem::SolidColor { color, rect } => {
                    self.render_solid_color(ui, color, rect);
                }
                DisplayItem::Text { text, rect, color } => {
                    self.render_text(ui, text, rect, color);
                }
                DisplayItem::Border { color, rect, width } => {
                    self.render_border(ui, color, rect, *width);
                }
            }
        }
    }
    
    fn render_solid_color(&self, ui: &mut egui::Ui, color: &Color, rect: &Rect) {
        let egui_color = egui::Color32::from_rgba_premultiplied(color.r, color.g, color.b, color.a);
        let egui_rect = egui::Rect::from_min_size(
            egui::pos2(rect.x, rect.y),
            egui::vec2(rect.width, rect.height),
        );
        
        ui.painter().rect_filled(egui_rect, 0.0, egui_color);
    }
    
    fn render_text(&self, ui: &mut egui::Ui, text: &str, rect: &Rect, color: &Color) {
        let egui_color = egui::Color32::from_rgba_premultiplied(color.r, color.g, color.b, color.a);
        let pos = egui::pos2(rect.x, rect.y);
        
        ui.painter().text(
            pos,
            egui::Align2::LEFT_TOP,
            text,
            egui::FontId::default(),
            egui_color,
        );
    }
    
    fn render_border(&self, ui: &mut egui::Ui, color: &Color, rect: &Rect, width: f32) {
        let egui_color = egui::Color32::from_rgba_premultiplied(color.r, color.g, color.b, color.a);
        let egui_rect = egui::Rect::from_min_size(
            egui::pos2(rect.x, rect.y),
            egui::vec2(rect.width, rect.height),
        );
        
        ui.painter().rect_stroke(egui_rect, 0.0, egui::Stroke::new(width, egui_color));
    }
}

pub fn build_display_list(layout_root: &LayoutBox) -> DisplayList {
    let mut list = DisplayList::new();
    render_layout_box(&mut list, layout_root);
    list
}

fn render_layout_box(list: &mut DisplayList, layout_box: &LayoutBox) {
    render_background(list, layout_box);
    render_borders(list, layout_box);
    
    match &layout_box.box_type {
        BoxType::BlockNode(node) | BoxType::InlineNode(node) => {
            if let crate::engine::dom::DOMNode::Text(text) = node {
                render_text(list, layout_box, text);
            }
        }
        BoxType::AnonymousBlock => {}
    }
    
    for child in &layout_box.children {
        render_layout_box(list, child);
    }
}

fn render_background(_list: &mut DisplayList, _layout_box: &LayoutBox) {
    // TODO: Get background color from computed styles
    // For now, use transparent background
}

fn render_borders(_list: &mut DisplayList, _layout_box: &LayoutBox) {
    // TODO: Get border properties from computed styles
    // For now, no borders
}

fn render_text(list: &mut DisplayList, layout_box: &LayoutBox, text: &str) {
    let color = Color { r: 255, g: 255, b: 255, a: 255 }; // White text for now
    
    list.items.push(DisplayItem::Text {
        text: text.to_string(),
        rect: layout_box.content,
        color,
    });
}

pub fn get_color(value: &Value) -> Option<Color> {
    match value {
        Value::ColorValue(color) => Some(color.clone()),
        Value::Keyword(name) => {
            match name.to_lowercase().as_str() {
                "black" => Some(Color { r: 0, g: 0, b: 0, a: 255 }),
                "white" => Some(Color { r: 255, g: 255, b: 255, a: 255 }),
                "red" => Some(Color { r: 255, g: 0, b: 0, a: 255 }),
                "green" => Some(Color { r: 0, g: 255, b: 0, a: 255 }),
                "blue" => Some(Color { r: 0, g: 0, b: 255, a: 255 }),
                "transparent" => Some(Color { r: 0, g: 0, b: 0, a: 0 }),
                _ => None,
            }
        }
        _ => None,
    }
}