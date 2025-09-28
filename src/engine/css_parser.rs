// CSS Parser - Basic implementation for styling

// use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone)]
pub struct Selector {
    pub simple: Vec<SimpleSelector>,
}

#[derive(Debug, Clone)]
pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
}

#[derive(Debug, Clone)]
pub enum Unit {
    Px,
    Em,
    Rem,
    Percent,
}

#[derive(Debug, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub fn parse(css: &str) -> Stylesheet {
    let mut parser = CSSParser::new(css);
    parser.parse_stylesheet()
}

struct CSSParser {
    input: String,
    position: usize,
}

impl CSSParser {
    fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            position: 0,
        }
    }
    
    fn parse_stylesheet(&mut self) -> Stylesheet {
        let mut rules = Vec::new();
        
        while !self.at_end() {
            self.skip_whitespace();
            if let Some(rule) = self.parse_rule() {
                rules.push(rule);
            }
        }
        
        Stylesheet { rules }
    }
    
    fn parse_rule(&mut self) -> Option<Rule> {
        let selectors = self.parse_selectors();
        
        self.skip_whitespace();
        if self.peek() != '{' {
            return None;
        }
        self.consume_char();
        
        let declarations = self.parse_declarations();
        
        self.skip_whitespace();
        if self.peek() == '}' {
            self.consume_char();
        }
        
        Some(Rule {
            selectors,
            declarations,
        })
    }
    
    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        
        loop {
            self.skip_whitespace();
            if let Some(selector) = self.parse_selector() {
                selectors.push(selector);
            }
            
            self.skip_whitespace();
            if self.peek() == ',' {
                self.consume_char();
            } else {
                break;
            }
        }
        
        selectors
    }
    
    fn parse_selector(&mut self) -> Option<Selector> {
        let mut simple = Vec::new();
        
        loop {
            self.skip_whitespace();
            
            if let Some(simple_selector) = self.parse_simple_selector() {
                simple.push(simple_selector);
            } else {
                break;
            }
        }
        
        if simple.is_empty() {
            None
        } else {
            Some(Selector { simple })
        }
    }
    
    fn parse_simple_selector(&mut self) -> Option<SimpleSelector> {
        let mut selector = SimpleSelector {
            tag_name: None,
            id: None,
            class: Vec::new(),
        };
        
        loop {
            match self.peek() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                c if c.is_alphabetic() => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break,
            }
        }
        
        if selector.tag_name.is_some() || selector.id.is_some() || !selector.class.is_empty() {
            Some(selector)
        } else {
            None
        }
    }
    
    fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();
        
        while !self.at_end() && self.peek() != '}' {
            self.skip_whitespace();
            
            if let Some(declaration) = self.parse_declaration() {
                declarations.push(declaration);
            }
            
            self.skip_whitespace();
            if self.peek() == ';' {
                self.consume_char();
            }
        }
        
        declarations
    }
    
    fn parse_declaration(&mut self) -> Option<Declaration> {
        let name = self.parse_identifier();
        
        self.skip_whitespace();
        if self.peek() != ':' {
            return None;
        }
        self.consume_char();
        
        self.skip_whitespace();
        let value = self.parse_value();
        
        Some(Declaration { name, value })
    }
    
    fn parse_value(&mut self) -> Value {
        self.skip_whitespace();
        
        if self.peek().is_numeric() {
            self.parse_length()
        } else if self.peek() == '#' {
            self.parse_color()
        } else {
            Value::Keyword(self.parse_identifier())
        }
    }
    
    fn parse_length(&mut self) -> Value {
        let mut num_str = String::new();
        
        while !self.at_end() && (self.peek().is_numeric() || self.peek() == '.') {
            num_str.push(self.consume_char());
        }
        
        let number: f32 = num_str.parse().unwrap_or(0.0);
        let unit = self.parse_unit();
        
        Value::Length(number, unit)
    }
    
    fn parse_unit(&mut self) -> Unit {
        let unit_str = self.parse_identifier();
        
        match unit_str.to_lowercase().as_str() {
            "px" => Unit::Px,
            "em" => Unit::Em,
            "rem" => Unit::Rem,
            "%" => Unit::Percent,
            _ => Unit::Px,
        }
    }
    
    fn parse_color(&mut self) -> Value {
        self.consume_char(); // Skip '#'
        
        let mut hex = String::new();
        while !self.at_end() && hex.len() < 6 && self.peek().is_ascii_hexdigit() {
            hex.push(self.consume_char());
        }
        
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        
        Value::ColorValue(Color { r, g, b, a: 255 })
    }
    
    fn parse_identifier(&mut self) -> String {
        let start = self.position;
        
        while !self.at_end() {
            let ch = self.peek();
            if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                self.consume_char();
            } else {
                break;
            }
        }
        
        self.input[start..self.position].to_string()
    }
    
    fn peek(&self) -> char {
        self.input.chars().nth(self.position).unwrap_or('\0')
    }
    
    fn consume_char(&mut self) -> char {
        let ch = self.peek();
        if ch != '\0' {
            self.position += ch.len_utf8();
        }
        ch
    }
    
    fn skip_whitespace(&mut self) {
        while !self.at_end() && self.peek().is_whitespace() {
            self.consume_char();
        }
    }
    
    fn at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}