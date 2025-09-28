use crate::engine::dom::DOMNode;
use std::collections::HashMap;

pub fn parse(html: &str) -> DOMNode {
    let mut parser = HTMLParser::new(html.to_string());
    parser.parse()
}

pub struct HTMLParser {
    input: String,
    chars: Vec<char>,
    position: usize,
}

impl HTMLParser {
    fn new(input: String) -> Self {
        let chars: Vec<char> = input.chars().collect();
        HTMLParser {
            input,
            chars,
            position: 0,
        }
    }
    
    fn parse(&mut self) -> DOMNode {
        let mut root = DOMNode::new_element("html".to_string());
        
        while !self.at_end() {
            self.skip_whitespace();
            if self.at_end() {
                break;
            }
            
            if let Some(node) = self.parse_node() {
                root.add_child(node);
            }
        }
        
        root
    }
    
    fn parse_node(&mut self) -> Option<DOMNode> {
        if self.peek() == '<' {
            self.parse_element()
        } else {
            self.parse_text()
        }
    }
    
    fn parse_element(&mut self) -> Option<DOMNode> {
        // Skip '<'
        self.consume_char();
        
        // Handle comments and DOCTYPE
        if self.input[self.position..].starts_with("!--") {
            return self.parse_comment();
        }
        
        if self.input[self.position..].to_lowercase().starts_with("!doctype") {
            return self.parse_doctype();
        }
        
        // Check for closing tag
        let is_closing = self.peek() == '/';
        if is_closing {
            self.consume_char();
        }
        
        let tag_name = self.parse_tag_name();
        
        if is_closing {
            // This is a closing tag, skip it for now
            self.skip_until('>');
            self.consume_char(); // Skip '>'
            return None;
        }
        
        let attributes = self.parse_attributes();
        
        self.skip_whitespace();
        
        // Check for self-closing tag
        let is_self_closing = self.peek() == '/' || 
                             matches!(tag_name.to_lowercase().as_str(), 
                                     "img" | "br" | "hr" | "input" | "meta" | "link");
        
        if is_self_closing && self.peek() == '/' {
            self.consume_char(); // Skip '/'
        }
        
        if self.peek() != '>' {
            return None;
        }
        self.consume_char(); // Skip '>'
        
        let mut element = DOMNode::new_element(tag_name.clone());
        
        for (name, value) in attributes {
            element.set_attribute(name, value);
        }
        
        if !is_self_closing {
            // Parse children until we find the closing tag
            while !self.at_end() {
                self.skip_whitespace();
                
                if self.peek() == '<' && self.peek_ahead(1) == '/' {
                    // Found closing tag
                    self.consume_char(); // '<'
                    self.consume_char(); // '/'
                    let closing_tag = self.parse_tag_name();
                    
                    if closing_tag.to_lowercase() == tag_name.to_lowercase() {
                        self.skip_until('>');
                        self.consume_char(); // '>'
                        break;
                    }
                }
                
                if let Some(child) = self.parse_node() {
                    element.add_child(child);
                }
            }
        }
        
        Some(element)
    }
    
    fn parse_text(&mut self) -> Option<DOMNode> {
        let mut text = String::new();
        
        while !self.at_end() && self.peek() != '<' {
            text.push(self.consume_char());
        }
        
        if text.trim().is_empty() {
            None
        } else {
            Some(DOMNode::new_text(text))
        }
    }
    
    fn parse_comment(&mut self) -> Option<DOMNode> {
        // Skip "!--"
        self.position += 3;
        
        // Find "-->"
        while !self.at_end() {
            if self.position + 2 < self.chars.len() 
                && self.chars[self.position] == '-' 
                && self.chars[self.position + 1] == '-' 
                && self.chars[self.position + 2] == '>' {
                self.position += 3;
                break;
            }
            self.consume_char();
        }
        
        None // Comments are ignored
    }
    
    fn parse_doctype(&mut self) -> Option<DOMNode> {
        self.skip_until('>');
        if !self.at_end() {
            self.consume_char(); // Skip '>'
        }
        None // DOCTYPE is ignored
    }
    
    fn parse_tag_name(&mut self) -> String {
        let mut name = String::new();
        
        while !self.at_end() {
            let ch = self.peek();
            if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                name.push(self.consume_char());
            } else {
                break;
            }
        }
        
        name
    }
    
    fn parse_attributes(&mut self) -> HashMap<String, String> {
        let mut attributes = HashMap::new();
        
        loop {
            self.skip_whitespace();
            
            if self.at_end() || self.peek() == '>' || self.peek() == '/' {
                break;
            }
            
            let name = self.parse_attribute_name();
            let mut value = String::new();
            
            self.skip_whitespace();
            
            if self.peek() == '=' {
                self.consume_char(); // Skip '='
                self.skip_whitespace();
                value = self.parse_attribute_value();
            }
            
            attributes.insert(name, value);
        }
        
        attributes
    }
    
    fn parse_attribute_name(&mut self) -> String {
        let mut name = String::new();
        
        while !self.at_end() {
            let ch = self.peek();
            if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                name.push(self.consume_char());
            } else {
                break;
            }
        }
        
        name
    }
    
    fn parse_attribute_value(&mut self) -> String {
        let quote = self.peek();
        
        if quote == '"' || quote == '\'' {
            self.consume_char(); // Skip opening quote
            let mut value = String::new();
            
            while !self.at_end() && self.peek() != quote {
                value.push(self.consume_char());
            }
            
            if !self.at_end() {
                self.consume_char(); // Skip closing quote
            }
            
            value
        } else {
            // Unquoted value
            let mut value = String::new();
            
            while !self.at_end() {
                let ch = self.peek();
                if ch.is_whitespace() || ch == '>' || ch == '/' {
                    break;
                }
                value.push(self.consume_char());
            }
            
            value
        }
    }
    
    fn peek(&self) -> char {
        self.chars.get(self.position).copied().unwrap_or('\0')
    }
    
    fn peek_ahead(&self, offset: usize) -> char {
        self.chars.get(self.position + offset).copied().unwrap_or('\0')
    }
    
    fn consume_char(&mut self) -> char {
        let ch = self.peek();
        if ch != '\0' {
            self.position += 1;
        }
        ch
    }
    
    fn skip_whitespace(&mut self) {
        while !self.at_end() && self.peek().is_whitespace() {
            self.consume_char();
        }
    }
    
    fn skip_until(&mut self, target: char) {
        while !self.at_end() && self.peek() != target {
            self.consume_char();
        }
    }
    
    fn at_end(&self) -> bool {
        self.position >= self.chars.len()
    }
}