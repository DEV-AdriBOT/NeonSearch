// Layout engine for positioning elements

use crate::engine::dom::DOMNode;
use crate::engine::css_parser::{Value, Unit};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub box_type: BoxType,
    pub children: Vec<LayoutBox>,
    pub content: Rect,
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes,
}

#[derive(Debug, Clone)]
pub enum BoxType {
    BlockNode(DOMNode),
    InlineNode(DOMNode),
    AnonymousBlock,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeSizes {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Debug, Clone)]
pub struct StyledNode {
    pub node: DOMNode,
    pub specified_values: HashMap<String, Value>,
    pub children: Vec<StyledNode>,
}

impl LayoutBox {
    pub fn new(box_type: BoxType) -> LayoutBox {
        LayoutBox {
            box_type,
            children: Vec::new(),
            content: Rect::default(),
            padding: EdgeSizes::default(),
            border: EdgeSizes::default(),
            margin: EdgeSizes::default(),
        }
    }
    
    pub fn get_style_node(&self) -> &DOMNode {
        match &self.box_type {
            BoxType::BlockNode(node) | BoxType::InlineNode(node) => node,
            BoxType::AnonymousBlock => panic!("Anonymous block has no style node"),
        }
    }
    
    pub fn layout(&mut self, containing_block: Rect) {
        match &self.box_type {
            BoxType::BlockNode(_) => self.layout_block(containing_block),
            BoxType::InlineNode(_) => self.layout_inline(containing_block),
            BoxType::AnonymousBlock => self.layout_block(containing_block),
        }
    }
    
    fn layout_block(&mut self, containing_block: Rect) {
        // Calculate the box's width
        self.calculate_block_width(containing_block);
        
        // Determine where the box is located within its container
        self.calculate_block_position(containing_block);
        
        // Recursively lay out the children of this box
        self.layout_block_children();
        
        // Parent height can depend on child height, so calculate_height must be called after the children are laid out
        self.calculate_block_height();
    }
    
    fn calculate_block_width(&mut self, containing_block: Rect) {
        // Default values
        let auto = Value::Keyword("auto".to_string());
        let zero = Value::Length(0.0, Unit::Px);
        
        // TODO: Get values from computed styles
        let width = auto.clone();
        let margin_left = zero.clone();
        let margin_right = zero.clone();
        let border_left = zero.clone();
        let border_right = zero.clone();
        let padding_left = zero.clone();
        let padding_right = zero.clone();
        
        let _total = [&margin_left, &border_left, &padding_left,
                     &width, &padding_right, &border_right, &margin_right];
        
        // Convert lengths to pixels
        self.margin.left = to_px(&margin_left, containing_block.width);
        self.margin.right = to_px(&margin_right, containing_block.width);
        self.border.left = to_px(&border_left, containing_block.width);
        self.border.right = to_px(&border_right, containing_block.width);
        self.padding.left = to_px(&padding_left, containing_block.width);
        self.padding.right = to_px(&padding_right, containing_block.width);
        
        let underflow = containing_block.width - self.margin.left - self.margin.right
            - self.border.left - self.border.right - self.padding.left - self.padding.right;
        
        // Set the content width
        self.content.width = underflow;
    }
    
    fn calculate_block_position(&mut self, containing_block: Rect) {
        // CSS values handled by layout engine
        let zero = Value::Length(0.0, Unit::Px);
        
        self.margin.top = to_px(&zero, containing_block.height);
        self.margin.bottom = to_px(&zero, containing_block.height);
        self.border.top = to_px(&zero, containing_block.height);
        self.border.bottom = to_px(&zero, containing_block.height);
        self.padding.top = to_px(&zero, containing_block.height);
        self.padding.bottom = to_px(&zero, containing_block.height);
        
        self.content.x = containing_block.x + self.margin.left + self.border.left + self.padding.left;
        self.content.y = containing_block.y + self.margin.top + self.border.top + self.padding.top;
    }
    
    fn layout_block_children(&mut self) {
        let d = &mut self.content;
        for child in &mut self.children {
            child.layout(*d);
            d.height += child.margin_box().height;
        }
    }
    
    fn calculate_block_height(&mut self) {
        // If the height is set to an explicit length, use that exact length.
        // Otherwise, just keep the value set by `layout_block_children`.
        
        // Height computed from CSS via layout engine
        // For now, height is determined by the content
    }
    
    fn layout_inline(&mut self, containing_block: Rect) {
        // Simplified inline layout - treat as block for now
        self.layout_block(containing_block);
    }
    
    fn margin_box(&self) -> Rect {
        self.border_box().expanded_by(self.margin)
    }
    
    fn border_box(&self) -> Rect {
        self.padding_box().expanded_by(self.border)
    }
    
    fn padding_box(&self) -> Rect {
        self.content.expanded_by(self.padding)
    }
}

impl Rect {
    fn expanded_by(self, edge: EdgeSizes) -> Rect {
        Rect {
            x: self.x - edge.left,
            y: self.y - edge.top,
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top + edge.bottom,
        }
    }
}

fn to_px(value: &Value, containing_block_size: f32) -> f32 {
    match value {
        Value::Length(length, Unit::Px) => *length,
        Value::Length(length, Unit::Percent) => containing_block_size * length / 100.0,
        Value::Length(length, Unit::Em) => length * 16.0, // Assume 16px base font size
        Value::Length(length, Unit::Rem) => length * 16.0, // Assume 16px base font size
        _ => 0.0,
    }
}

pub fn build_layout_tree(root: &StyledNode) -> LayoutBox {
    let mut root_box = match &root.node {
        DOMNode::Element { .. } => {
            // Determine display type from styles
            let display = root.specified_values
                .get("display")
                .and_then(|v| match v {
                    Value::Keyword(s) => Some(s.as_str()),
                    _ => None,
                })
                .unwrap_or("block");
            
            match display {
                "block" => LayoutBox::new(BoxType::BlockNode(root.node.clone())),
                "inline" => LayoutBox::new(BoxType::InlineNode(root.node.clone())),
                _ => LayoutBox::new(BoxType::BlockNode(root.node.clone())),
            }
        }
        DOMNode::Text(_) => LayoutBox::new(BoxType::InlineNode(root.node.clone())),
        DOMNode::Comment(_) => LayoutBox::new(BoxType::InlineNode(root.node.clone())), // Comments don't affect layout
    };
    
    for child in &root.children {
        root_box.children.push(build_layout_tree(child));
    }
    
    root_box
}

/// Layout engine for processing DOM trees and computing layout
#[derive(Debug)]
pub struct LayoutEngine {
    viewport_width: f32,
    viewport_height: f32,
}

impl LayoutEngine {
    /// Create a new layout engine
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            viewport_width,
            viewport_height,
        }
    }

    /// Compute layout for a styled node tree
    pub fn layout(&self, styled_root: &StyledNode) -> LayoutBox {
        let mut layout_root = build_layout_tree(styled_root);
        layout_root.layout(Rect {
            x: 0.0,
            y: 0.0,
            width: self.viewport_width,
            height: self.viewport_height,
        });
        layout_root
    }

    /// Update viewport dimensions
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    /// Get current viewport dimensions
    pub fn get_viewport(&self) -> (f32, f32) {
        (self.viewport_width, self.viewport_height)
    }
}