// JavaScript engine integration - Basic interpreter
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use regex::Regex;

use crate::engine::dom::DOMNode;

pub mod console;
pub mod dom_api;

use dom_api::DOMApi;
pub mod event_system;
pub mod test;

use console::ConsoleAPI;
use event_system::EventSystem;

#[derive(Debug, Clone)]
pub enum JSValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Undefined,
    Object(HashMap<String, JSValue>),
    Array(Vec<JSValue>),
}

impl JSValue {
    pub fn to_string(&self) -> String {
        match self {
            JSValue::String(s) => s.clone(),
            JSValue::Number(n) => n.to_string(),
            JSValue::Boolean(b) => b.to_string(),
            JSValue::Null => "null".to_string(),
            JSValue::Undefined => "undefined".to_string(),
            JSValue::Object(_) => "[object Object]".to_string(),
            JSValue::Array(arr) => {
                format!("[{}]", arr.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", "))
            }
        }
    }
    
    pub fn is_truthy(&self) -> bool {
        match self {
            JSValue::Boolean(b) => *b,
            JSValue::Number(n) => *n != 0.0 && !n.is_nan(),
            JSValue::String(s) => !s.is_empty(),
            JSValue::Null | JSValue::Undefined => false,
            JSValue::Object(_) | JSValue::Array(_) => true,
        }
    }
}

pub struct JSEngine {
    variables: HashMap<String, JSValue>,
    console_api: ConsoleAPI,
    event_system: EventSystem,
    dom_root: Option<Rc<RefCell<DOMNode>>>,
    dom_api: DOMApi,
}

impl JSEngine {
    pub fn new() -> Result<Self> {
        let console_output = Rc::new(RefCell::new(Vec::new()));
        let console_api = ConsoleAPI::new(console_output);
        let event_system = EventSystem::new();
        let dom_api = DOMApi::new();
        
        let mut engine = Self {
            variables: HashMap::new(),
            console_api,
            event_system,
            dom_root: None,
            dom_api,
        };
        
        // Set up global objects
        engine.setup_globals()?;
        
        Ok(engine)
    }
    
    pub fn execute(&mut self, code: &str) -> Result<String> {
        // Simple JavaScript interpreter
        // This handles basic statements like:
        // - console.log("message")
        // - var x = 5
        // - x = "hello"
        
        let code = code.trim();
        
        // Handle console.log statements
        if let Some(result) = self.handle_console_log(code)? {
            return Ok(result);
        }
        
        // Handle variable declarations
        if let Some(result) = self.handle_variable_declaration(code)? {
            return Ok(result);
        }
        
        // Handle variable assignments
        if let Some(result) = self.handle_variable_assignment(code)? {
            return Ok(result);
        }
        
        // Handle variable access
        if let Some(result) = self.handle_variable_access(code)? {
            return Ok(result);
        }
        
        // Handle function calls (basic)
        if let Some(result) = self.handle_function_call(code)? {
            return Ok(result);
        }
        
        // Handle DOM API calls
        if let Some(result) = self.handle_dom_api_call(code)? {
            return Ok(result);
        }
        
        // Return the code as-is if not recognized
        Ok(format!("Unrecognized JavaScript: {}", code))
    }
    
    fn handle_console_log(&mut self, code: &str) -> Result<Option<String>> {
        let console_regex = Regex::new(r#"console\.log\s*\(\s*["']([^"']*)["']\s*\)"#)?;
        
        if let Some(captures) = console_regex.captures(code) {
            let message = captures.get(1).map_or("", |m| m.as_str());
            self.console_api.log(message);
            return Ok(Some("undefined".to_string()));
        }
        
        // Handle console.log with variables
        let var_regex = Regex::new(r#"console\.log\s*\(\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)"#)?;
        if let Some(captures) = var_regex.captures(code) {
            let var_name = captures.get(1).map_or("", |m| m.as_str());
            if let Some(value) = self.variables.get(var_name) {
                self.console_api.log(&value.to_string());
            } else {
                self.console_api.log(&format!("ReferenceError: {} is not defined", var_name));
            }
            return Ok(Some("undefined".to_string()));
        }
        
        Ok(None)
    }
    
    fn handle_variable_declaration(&mut self, code: &str) -> Result<Option<String>> {
        // Handle: var x = "value" or let x = 5
        let var_regex = Regex::new(r#"(?:var|let|const)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.+)"#)?;
        
        if let Some(captures) = var_regex.captures(code) {
            let var_name = captures.get(1).map_or("", |m| m.as_str()).to_string();
            let value_str = captures.get(2).map_or("", |m| m.as_str()).trim();
            
            let value = self.parse_value(value_str)?;
            self.variables.insert(var_name, value);
            
            return Ok(Some("undefined".to_string()));
        }
        
        Ok(None)
    }
    
    fn handle_variable_assignment(&mut self, code: &str) -> Result<Option<String>> {
        // Handle: x = "value"
        let assign_regex = Regex::new(r#"([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.+)"#)?;
        
        if let Some(captures) = assign_regex.captures(code) {
            let var_name = captures.get(1).map_or("", |m| m.as_str()).to_string();
            let value_str = captures.get(2).map_or("", |m| m.as_str()).trim();
            
            let value = self.parse_value(value_str)?;
            let result = value.to_string();
            self.variables.insert(var_name, value);
            
            return Ok(Some(result));
        }
        
        Ok(None)
    }
    
    fn handle_variable_access(&mut self, code: &str) -> Result<Option<String>> {
        // Handle: x (variable access)
        let var_regex = Regex::new(r#"^[a-zA-Z_][a-zA-Z0-9_]*$"#)?;
        
        if var_regex.is_match(code) {
            if let Some(value) = self.variables.get(code) {
                return Ok(Some(value.to_string()));
            } else {
                return Ok(Some(format!("ReferenceError: {} is not defined", code)));
            }
        }
        
        Ok(None)
    }
    
    fn handle_function_call(&mut self, code: &str) -> Result<Option<String>> {
        // Handle basic function calls like alert("message")
        let func_regex = Regex::new(r#"([a-zA-Z_][a-zA-Z0-9_]*)\s*\(\s*["']([^"']*)["']\s*\)"#)?;
        
        if let Some(captures) = func_regex.captures(code) {
            let func_name = captures.get(1).map_or("", |m| m.as_str());
            let arg = captures.get(2).map_or("", |m| m.as_str());
            
            match func_name {
                "alert" => {
                    // For now, just log the alert message
                    self.console_api.log(&format!("ALERT: {}", arg));
                    return Ok(Some("undefined".to_string()));
                },
                _ => {
                    return Ok(Some(format!("TypeError: {} is not a function", func_name)));
                }
            }
        }
        
        Ok(None)
    }
    
    fn parse_value(&self, value_str: &str) -> Result<JSValue> {
        let value_str = value_str.trim();
        
        // String literals
        if (value_str.starts_with('"') && value_str.ends_with('"')) ||
           (value_str.starts_with('\'') && value_str.ends_with('\'')) {
            let content = &value_str[1..value_str.len()-1];
            return Ok(JSValue::String(content.to_string()));
        }
        
        // Numbers
        if let Ok(num) = value_str.parse::<f64>() {
            return Ok(JSValue::Number(num));
        }
        
        // Booleans
        match value_str {
            "true" => return Ok(JSValue::Boolean(true)),
            "false" => return Ok(JSValue::Boolean(false)),
            "null" => return Ok(JSValue::Null),
            "undefined" => return Ok(JSValue::Undefined),
            _ => {}
        }
        
        // Variable reference
        if let Some(value) = self.variables.get(value_str) {
            return Ok(value.clone());
        }
        
        // Default to string if nothing else matches
        Ok(JSValue::String(value_str.to_string()))
    }
    
    fn setup_globals(&mut self) -> Result<()> {
        // Set up basic global variables
        self.variables.insert("undefined".to_string(), JSValue::Undefined);
        self.variables.insert("null".to_string(), JSValue::Null);
        
        // Set up window object (basic)
        let mut window = HashMap::new();
        window.insert("location".to_string(), JSValue::String("about:blank".to_string()));
        self.variables.insert("window".to_string(), JSValue::Object(window));
        
        Ok(())
    }
    
    pub fn set_dom_root(&mut self, dom_root: Rc<RefCell<DOMNode>>) -> Result<()> {
        self.dom_root = Some(dom_root.clone());
        
        // Configure DOM API with the new root
        self.dom_api.set_document(dom_root);
        
        // Set up basic document object
        let mut document = HashMap::new();
        document.insert("title".to_string(), JSValue::String("NeonSearch Document".to_string()));
        document.insert("URL".to_string(), JSValue::String("about:blank".to_string()));
        self.variables.insert("document".to_string(), JSValue::Object(document));
        
        Ok(())
    }
    
    pub fn get_console_output(&self) -> Vec<String> {
        self.console_api.get_output()
    }
    
    pub fn clear_console(&self) {
        self.console_api.clear_output();
    }
    
    pub fn trigger_event(&self, event: event_system::EventData) -> Result<()> {
        self.event_system.trigger_event(event)
    }
    
    pub fn has_event_listeners(&self, event_type: &str) -> bool {
        self.event_system.has_listeners(event_type)
    }
    
    fn handle_dom_api_call(&mut self, code: &str) -> Result<Option<String>> {
        // Handle document.querySelector() calls
        let query_selector_regex = Regex::new(r#"document\.querySelector\s*\(\s*["']([^"']*)["']\s*\)"#)?;
        if let Some(captures) = query_selector_regex.captures(code) {
            let selector = captures.get(1).map_or("", |m| m.as_str());
            let result = self.dom_api.query_selector(selector);
            return Ok(Some(result.to_string()));
        }
        
        // Handle document.querySelectorAll() calls
        let query_selector_all_regex = Regex::new(r#"document\.querySelectorAll\s*\(\s*["']([^"']*)["']\s*\)"#)?;
        if let Some(captures) = query_selector_all_regex.captures(code) {
            let selector = captures.get(1).map_or("", |m| m.as_str());
            let result = self.dom_api.query_selector_all(selector);
            return Ok(Some(result.to_string()));
        }
        
        // Handle document.getElementById() calls
        let get_by_id_regex = Regex::new(r#"document\.getElementById\s*\(\s*["']([^"']*)["']\s*\)"#)?;
        if let Some(captures) = get_by_id_regex.captures(code) {
            let id = captures.get(1).map_or("", |m| m.as_str());
            let result = self.dom_api.get_element_by_id(id);
            return Ok(Some(result.to_string()));
        }
        
        // Handle document.getElementsByTagName() calls
        let get_by_tag_regex = Regex::new(r#"document\.getElementsByTagName\s*\(\s*["']([^"']*)["']\s*\)"#)?;
        if let Some(captures) = get_by_tag_regex.captures(code) {
            let tag_name = captures.get(1).map_or("", |m| m.as_str());
            let result = self.dom_api.get_elements_by_tag_name(tag_name);
            return Ok(Some(result.to_string()));
        }
        
        // Handle document.getElementsByClassName() calls
        let get_by_class_regex = Regex::new(r#"document\.getElementsByClassName\s*\(\s*["']([^"']*)["']\s*\)"#)?;
        if let Some(captures) = get_by_class_regex.captures(code) {
            let class_name = captures.get(1).map_or("", |m| m.as_str());
            let result = self.dom_api.get_elements_by_class_name(class_name);
            return Ok(Some(result.to_string()));
        }
        
        // Handle document.createElement() calls
        let create_element_regex = Regex::new(r#"document\.createElement\s*\(\s*["']([^"']*)["']\s*\)"#)?;
        if let Some(captures) = create_element_regex.captures(code) {
            let tag_name = captures.get(1).map_or("", |m| m.as_str());
            let result = self.dom_api.create_element(tag_name);
            return Ok(Some(result.to_string()));
        }
        
        // Handle document.createTextNode() calls
        let create_text_regex = Regex::new(r#"document\.createTextNode\s*\(\s*["']([^"']*)["']\s*\)"#)?;
        if let Some(captures) = create_text_regex.captures(code) {
            let text = captures.get(1).map_or("", |m| m.as_str());
            let result = self.dom_api.create_text_node(text);
            return Ok(Some(result.to_string()));
        }
        
        Ok(None)
    }
    
    pub fn bind_dom_api(&mut self) -> Result<()> {
        // Basic DOM API will be implemented later
        Ok(())
    }
}