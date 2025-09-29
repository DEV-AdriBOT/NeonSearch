use std::rc::Rc;
use std::cell::RefCell;
use crate::engine::dom::DOMNode;

/// Configuration for virtual scrolling behavior
#[derive(Debug, Clone)]
pub struct VirtualScrollConfig {
    pub viewport_height: f32,
    pub item_height: f32,
    pub buffer_size: usize,
    pub scroll_offset: f32,
}

impl Default for VirtualScrollConfig {
    fn default() -> Self {
        Self {
            viewport_height: 600.0,
            item_height: 20.0,
            buffer_size: 10,
            scroll_offset: 0.0,
        }
    }
}

/// Virtual viewport for managing large DOM trees efficiently
#[derive(Debug)]
pub struct VirtualViewport {
    config: VirtualScrollConfig,
    total_items: usize,
    visible_range: (usize, usize),
    rendered_nodes: Vec<Rc<RefCell<DOMNode>>>,
}

impl VirtualViewport {
    /// Create a new virtual viewport
    pub fn new(config: VirtualScrollConfig) -> Self {
        Self {
            config,
            total_items: 0,
            visible_range: (0, 0),
            rendered_nodes: Vec::new(),
        }
    }

    /// Update the scroll position and recalculate visible range
    pub fn update_scroll(&mut self, scroll_offset: f32) {
        self.config.scroll_offset = scroll_offset;
        self.calculate_visible_range();
    }

    /// Set the total number of items in the virtual list
    pub fn set_total_items(&mut self, total: usize) {
        self.total_items = total;
        self.calculate_visible_range();
    }

    /// Calculate which items should be visible based on scroll position
    fn calculate_visible_range(&mut self) {
        let items_per_viewport = (self.config.viewport_height / self.config.item_height) as usize;
        let start_index = (self.config.scroll_offset / self.config.item_height) as usize;
        
        let start_with_buffer = start_index.saturating_sub(self.config.buffer_size);
        let end_with_buffer = (start_index + items_per_viewport + self.config.buffer_size).min(self.total_items);
        
        self.visible_range = (start_with_buffer, end_with_buffer);
    }

    /// Get the currently visible range of items
    pub fn get_visible_range(&self) -> (usize, usize) {
        self.visible_range
    }

    /// Update the rendered nodes for the current visible range
    pub fn update_rendered_nodes(&mut self, nodes: Vec<Rc<RefCell<DOMNode>>>) {
        self.rendered_nodes = nodes;
    }

    /// Get the currently rendered nodes
    pub fn get_rendered_nodes(&self) -> &[Rc<RefCell<DOMNode>>] {
        &self.rendered_nodes
    }

    /// Calculate the total virtual height needed for scrolling
    pub fn get_virtual_height(&self) -> f32 {
        self.total_items as f32 * self.config.item_height
    }
}
