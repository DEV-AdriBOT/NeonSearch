use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::engine::dom::DOMNode;
use crate::js::JSValue;

/// JavaScript DOM API implementation
/// Provides JavaScript access to DOM manipulation functions
pub struct DOMApi {
    /// Reference to the current DOM tree (from WebPage)
    pub document_root: Option<Rc<RefCell<DOMNode>>>,
}

impl DOMApi {
    pub fn new() -> Self {
        Self {
            document_root: None,
        }
    }
    
    /// Set the document root for DOM operations
    pub fn set_document(&mut self, root: Rc<RefCell<DOMNode>>) {
        self.document_root = Some(root);
    }
    
    /// JavaScript document.querySelector(selector) implementation
    pub fn query_selector(&self, selector: &str) -> JSValue {
        if let Some(ref root) = self.document_root {
            if let Some(_element) = self.find_first_element(root, selector) {
                // Create a simple object representation
                let mut element_obj = HashMap::new();
                element_obj.insert("selector".to_string(), JSValue::String(selector.to_string()));
                element_obj.insert("found".to_string(), JSValue::Boolean(true));
                JSValue::Object(element_obj)
            } else {
                JSValue::Null
            }
        } else {
            JSValue::Null
        }
    }
    
    /// JavaScript document.querySelectorAll(selector) implementation
    pub fn query_selector_all(&self, selector: &str) -> JSValue {
        if let Some(ref root) = self.document_root {
            let elements = self.find_all_elements(root, selector);
            let mut result_array = Vec::new();
            for _element in elements {
                let mut element_obj = HashMap::new();
                element_obj.insert("selector".to_string(), JSValue::String(selector.to_string()));
                element_obj.insert("found".to_string(), JSValue::Boolean(true));
                result_array.push(JSValue::Object(element_obj));
            }
            JSValue::Array(result_array)
        } else {
            JSValue::Array(Vec::new())
        }
    }
    
    /// JavaScript document.getElementById(id) implementation
    pub fn get_element_by_id(&self, id: &str) -> JSValue {
        if let Some(ref root) = self.document_root {
            if let Some(_element) = self.find_element_by_id(root, id) {
                let mut element_obj = HashMap::new();
                element_obj.insert("id".to_string(), JSValue::String(id.to_string()));
                element_obj.insert("found".to_string(), JSValue::Boolean(true));
                JSValue::Object(element_obj)
            } else {
                JSValue::Null
            }
        } else {
            JSValue::Null
        }
    }
    
    /// JavaScript document.getElementsByTagName(tagName) implementation
    pub fn get_elements_by_tag_name(&self, tag_name: &str) -> JSValue {
        if let Some(ref root) = self.document_root {
            let elements = self.find_elements_by_tag_name(root, tag_name);
            let mut result_array = Vec::new();
            for _element in elements {
                let mut element_obj = HashMap::new();
                element_obj.insert("tagName".to_string(), JSValue::String(tag_name.to_string()));
                element_obj.insert("found".to_string(), JSValue::Boolean(true));
                result_array.push(JSValue::Object(element_obj));
            }
            JSValue::Array(result_array)
        } else {
            JSValue::Array(Vec::new())
        }
    }
    
    /// JavaScript document.getElementsByClassName(className) implementation
    pub fn get_elements_by_class_name(&self, class_name: &str) -> JSValue {
        if let Some(ref root) = self.document_root {
            let elements = self.find_elements_by_class_name(root, class_name);
            let mut result_array = Vec::new();
            for _element in elements {
                let mut element_obj = HashMap::new();
                element_obj.insert("className".to_string(), JSValue::String(class_name.to_string()));
                element_obj.insert("found".to_string(), JSValue::Boolean(true));
                result_array.push(JSValue::Object(element_obj));
            }
            JSValue::Array(result_array)
        } else {
            JSValue::Array(Vec::new())
        }
    }
    
    /// JavaScript document.createElement(tagName) implementation
    pub fn create_element(&self, tag_name: &str) -> JSValue {
        let mut element_obj = HashMap::new();
        element_obj.insert("tagName".to_string(), JSValue::String(tag_name.to_string()));
        element_obj.insert("type".to_string(), JSValue::String("Element".to_string()));
        JSValue::Object(element_obj)
    }
    
    /// JavaScript document.createTextNode(text) implementation
    pub fn create_text_node(&self, text: &str) -> JSValue {
        let mut text_obj = HashMap::new();
        text_obj.insert("textContent".to_string(), JSValue::String(text.to_string()));
        text_obj.insert("type".to_string(), JSValue::String("TextNode".to_string()));
        JSValue::Object(text_obj)
    }
    
    // Private helper methods for DOM traversal
    
    fn find_first_element(&self, node: &Rc<RefCell<DOMNode>>, selector: &str) -> Option<Rc<RefCell<DOMNode>>> {
        let borrowed = node.borrow();
        
        // Simple selector matching (can be enhanced for complex selectors)
        if self.matches_selector(&borrowed, selector) {
            drop(borrowed);
            return Some(node.clone());
        }
        
        // Search children
        match &*borrowed {
            DOMNode::Element { children, .. } => {
                for child in children {
                    // Convert DOMNode to Rc<RefCell<DOMNode>> for recursion
                    let child_rc = Rc::new(RefCell::new(child.clone()));
                    if let Some(found) = self.find_first_element(&child_rc, selector) {
                        return Some(found);
                    }
                }
            }
            _ => {}
        }
        
        None
    }
    
    fn find_all_elements(&self, node: &Rc<RefCell<DOMNode>>, selector: &str) -> Vec<Rc<RefCell<DOMNode>>> {
        let mut results = Vec::new();
        self.collect_matching_elements(node, selector, &mut results);
        results
    }
    
    fn collect_matching_elements(&self, node: &Rc<RefCell<DOMNode>>, selector: &str, results: &mut Vec<Rc<RefCell<DOMNode>>>) {
        let borrowed = node.borrow();
        
        if self.matches_selector(&borrowed, selector) {
            results.push(node.clone());
        }
        
        match &*borrowed {
            DOMNode::Element { children, .. } => {
                for child in children {
                    let child_rc = Rc::new(RefCell::new(child.clone()));
                    self.collect_matching_elements(&child_rc, selector, results);
                }
            }
            _ => {}
        }
    }
    
    fn find_element_by_id(&self, node: &Rc<RefCell<DOMNode>>, id: &str) -> Option<Rc<RefCell<DOMNode>>> {
        let borrowed = node.borrow();
        
        if let DOMNode::Element { attributes, .. } = &*borrowed {
            if let Some(element_id) = attributes.get("id") {
                if element_id == id {
                    drop(borrowed);
                    return Some(node.clone());
                }
            }
        }
        
        match &*borrowed {
            DOMNode::Element { children, .. } => {
                for child in children {
                    let child_rc = Rc::new(RefCell::new(child.clone()));
                    if let Some(found) = self.find_element_by_id(&child_rc, id) {
                        return Some(found);
                    }
                }
            }
            _ => {}
        }
        
        None
    }
    
    fn find_elements_by_tag_name(&self, node: &Rc<RefCell<DOMNode>>, tag_name: &str) -> Vec<Rc<RefCell<DOMNode>>> {
        let mut results = Vec::new();
        self.collect_elements_by_tag_name(node, tag_name, &mut results);
        results
    }
    
    fn collect_elements_by_tag_name(&self, node: &Rc<RefCell<DOMNode>>, tag_name: &str, results: &mut Vec<Rc<RefCell<DOMNode>>>) {
        let borrowed = node.borrow();
        
        if let DOMNode::Element { tag_name: element_tag, .. } = &*borrowed {
            if element_tag.to_lowercase() == tag_name.to_lowercase() {
                results.push(node.clone());
            }
        }
        
        match &*borrowed {
            DOMNode::Element { children, .. } => {
                for child in children {
                    let child_rc = Rc::new(RefCell::new(child.clone()));
                    self.collect_elements_by_tag_name(&child_rc, tag_name, results);
                }
            }
            _ => {}
        }
    }
    
    fn find_elements_by_class_name(&self, node: &Rc<RefCell<DOMNode>>, class_name: &str) -> Vec<Rc<RefCell<DOMNode>>> {
        let mut results = Vec::new();
        self.collect_elements_by_class_name(node, class_name, &mut results);
        results
    }
    
    fn collect_elements_by_class_name(&self, node: &Rc<RefCell<DOMNode>>, class_name: &str, results: &mut Vec<Rc<RefCell<DOMNode>>>) {
        let borrowed = node.borrow();
        
        if let DOMNode::Element { attributes, .. } = &*borrowed {
            if let Some(classes) = attributes.get("class") {
                if classes.split_whitespace().any(|c| c == class_name) {
                    results.push(node.clone());
                }
            }
        }
        
        match &*borrowed {
            DOMNode::Element { children, .. } => {
                for child in children {
                    let child_rc = Rc::new(RefCell::new(child.clone()));
                    self.collect_elements_by_class_name(&child_rc, class_name, results);
                }
            }
            _ => {}
        }
    }
    
    /// Simple selector matching (can be enhanced for complex CSS selectors)
    fn matches_selector(&self, node: &DOMNode, selector: &str) -> bool {
        match node {
            DOMNode::Element { tag_name, attributes, .. } => {
                if selector.starts_with('#') {
                    // ID selector: #myid
                    let id = &selector[1..];
                    attributes.get("id").map_or(false, |attr_id| attr_id == id)
                } else if selector.starts_with('.') {
                    // Class selector: .myclass
                    let class = &selector[1..];
                    attributes.get("class").map_or(false, |classes| {
                        classes.split_whitespace().any(|c| c == class)
                    })
                } else {
                    // Tag selector: div
                    tag_name.to_lowercase() == selector.to_lowercase()
                }
            }
            _ => false,
        }
    }
}

impl Default for DOMApi {
    fn default() -> Self {
        Self::new()
    }
}