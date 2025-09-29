// Event system for JavaScript integration
use anyhow::Result;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

pub type EventCallback = Box<dyn Fn(&EventData) -> Result<()>>;

#[derive(Debug, Clone)]
pub struct EventData {
    pub event_type: String,
    pub target: String, // Element ID or selector
    pub data: HashMap<String, String>,
}

pub struct EventSystem {
    listeners: Rc<RefCell<HashMap<String, Vec<String>>>>, // event_type -> callback names
}

impl EventSystem {
    pub fn new() -> Self {
        Self {
            listeners: Rc::new(RefCell::new(HashMap::new())),
        }
    }
    
    pub fn add_event_listener(&self, event_type: &str, callback_name: &str) {
        self.listeners.borrow_mut()
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(callback_name.to_string());
    }
    
    pub fn remove_event_listener(&self, event_type: &str, callback_name: &str) {
        if let Some(callbacks) = self.listeners.borrow_mut().get_mut(event_type) {
            callbacks.retain(|name| name != callback_name);
        }
    }
    
    pub fn trigger_event(&self, event: EventData) -> Result<()> {
        // This would be called from the UI layer when events occur
        // For now, just log the event
        println!("Event triggered: {:?}", event);
        
        // In a full implementation, we'd execute the JavaScript callbacks here
        if let Some(callbacks) = self.listeners.borrow().get(&event.event_type) {
            for callback in callbacks {
                println!("Would execute callback: {}", callback);
            }
        }
        
        Ok(())
    }
    
    pub fn has_listeners(&self, event_type: &str) -> bool {
        self.listeners.borrow()
            .get(event_type)
            .map(|listeners| !listeners.is_empty())
            .unwrap_or(false)
    }
}