use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum DOMNode {
    Element {
        tag_name: String,
        attributes: HashMap<String, String>,
        children: Vec<DOMNode>,
    },
    Text(String),
}

impl DOMNode {
    pub fn new_element(tag_name: String) -> Self {
        DOMNode::Element {
            tag_name,
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }
    
    pub fn new_text(content: String) -> Self {
        DOMNode::Text(content)
    }
    
    pub fn add_child(&mut self, child: DOMNode) {
        if let DOMNode::Element { children, .. } = self {
            children.push(child);
        }
    }
    
    pub fn set_attribute(&mut self, name: String, value: String) {
        if let DOMNode::Element { attributes, .. } = self {
            attributes.insert(name, value);
        }
    }
    
    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        if let DOMNode::Element { attributes, .. } = self {
            attributes.get(name)
        } else {
            None
        }
    }
    
    pub fn tag_name(&self) -> Option<&String> {
        if let DOMNode::Element { tag_name, .. } = self {
            Some(tag_name)
        } else {
            None
        }
    }
    
    pub fn is_element(&self) -> bool {
        matches!(self, DOMNode::Element { .. })
    }
    
    pub fn is_text(&self) -> bool {
        matches!(self, DOMNode::Text(_))
    }
}