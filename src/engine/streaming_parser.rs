use anyhow::Result;
use crate::engine::dom::DOMNode;

/// Streaming HTML parser that can build DOM incrementally
#[derive(Debug)]
pub struct StreamingHtmlParser {
    buffer: String,
    parse_position: usize,
    partial_dom: Vec<DOMNode>,
    tag_stack: Vec<String>,
    current_text: String,
    chunk_size: usize,
    processed_bytes: usize,
    total_bytes: Option<usize>,
}

/// Progress information for streaming parsing
#[derive(Debug, Clone)]
pub struct ParseProgress {
    pub processed_bytes: usize,
    pub total_bytes: Option<usize>,
    pub nodes_created: usize,
    pub current_tag: Option<String>,
    pub is_complete: bool,
}

/// Chunk of parsed content that can be rendered immediately
#[derive(Debug, Clone)]
pub struct ParsedChunk {
    pub nodes: Vec<DOMNode>,
    pub progress: ParseProgress,
    pub text_content: String, // For fallback text rendering
}

impl StreamingHtmlParser {
    /// Create a new streaming parser
    pub fn new(chunk_size: usize) -> Self {
        Self {
            buffer: String::new(),
            parse_position: 0,
            partial_dom: Vec::new(),
            tag_stack: Vec::new(),
            current_text: String::new(),
            chunk_size,
            processed_bytes: 0,
            total_bytes: None,
        }
    }

    /// Set the expected total content size for progress tracking
    pub fn set_total_size(&mut self, total_size: usize) {
        self.total_bytes = Some(total_size);
    }

    /// Add a chunk of HTML content and return any completed nodes
    pub fn add_chunk(&mut self, chunk: &str) -> Result<Option<ParsedChunk>> {
        self.buffer.push_str(chunk);
        self.processed_bytes += chunk.len();
        
        // Try to parse complete elements from the buffer
        self.parse_incremental()
    }

    /// Signal that no more content will be added and parse remaining buffer
    pub fn finalize(&mut self) -> Result<ParsedChunk> {
        // Parse any remaining content
        let final_chunk = self.parse_remaining()?;
        Ok(final_chunk)
    }

    /// Parse incrementally while we have enough content
    fn parse_incremental(&mut self) -> Result<Option<ParsedChunk>> {
        let mut new_nodes = Vec::new();
        let mut text_content = String::new();
        let start_position = self.parse_position;

        while self.parse_position < self.buffer.len() {
            if let Some(node) = self.try_parse_next_element()? {
                new_nodes.push(node.clone());
                
                if let Some(text) = self.extract_text_content(&node) {
                    text_content.push_str(&text);
                    text_content.push(' ');
                }
            }

            // Stop if we've processed enough for this chunk
            if self.parse_position - start_position >= self.chunk_size {
                break;
            }
        }

        if !new_nodes.is_empty() {
            self.partial_dom.extend(new_nodes.iter().cloned());
            
            Ok(Some(ParsedChunk {
                nodes: new_nodes,
                progress: self.get_progress(),
                text_content,
            }))
        } else {
            Ok(None)
        }
    }

    /// Try to parse the next complete element
    fn try_parse_next_element(&mut self) -> Result<Option<DOMNode>> {
        // Skip whitespace
        while self.parse_position < self.buffer.len() && 
              self.buffer.chars().nth(self.parse_position).unwrap_or(' ').is_whitespace() {
            self.parse_position += 1;
        }

        if self.parse_position >= self.buffer.len() {
            return Ok(None);
        }

        let remaining = &self.buffer[self.parse_position..];
        
        if remaining.starts_with('<') {
            self.parse_tag()
        } else {
            self.parse_text()
        }
    }

    /// Parse an HTML tag
    fn parse_tag(&mut self) -> Result<Option<DOMNode>> {
        let start = self.parse_position;
        let remaining = &self.buffer[start..];
        
        // Find the end of the tag
        if let Some(end_pos) = remaining.find('>') {
            let tag_content = &remaining[1..end_pos]; // Skip '<' and '>'
            self.parse_position = start + end_pos + 1;
            
            // Handle different tag types
            if tag_content.starts_with('/') {
                // Closing tag
                let tag_name = tag_content[1..].trim().to_lowercase();
                if let Some(expected) = self.tag_stack.pop() {
                    if expected != tag_name {
                        // Mismatched tag - be lenient and continue
                        println!("Warning: Mismatched closing tag. Expected {}, got {}", expected, tag_name);
                    }
                }
                Ok(None) // Closing tags don't create new nodes
            } else if tag_content.ends_with('/') {
                // Self-closing tag
                let tag_name = tag_content[..tag_content.len()-1].trim();
                let (name, _attributes) = self.parse_tag_name_and_attributes(tag_name);
                Ok(Some(self.create_element_node(name)))
            } else if tag_content.starts_with('!') {
                // Comment or DOCTYPE - skip for now
                Ok(None)
            } else {
                // Opening tag
                let (name, _attributes) = self.parse_tag_name_and_attributes(tag_content);
                self.tag_stack.push(name.clone());
                Ok(Some(self.create_element_node(name)))
            }
        } else {
            // Incomplete tag - wait for more content
            Ok(None)
        }
    }

    /// Parse text content between tags
    fn parse_text(&mut self) -> Result<Option<DOMNode>> {
        let start = self.parse_position;
        let remaining = &self.buffer[start..];
        
        if let Some(next_tag) = remaining.find('<') {
            let text = &remaining[..next_tag];
            self.parse_position = start + next_tag;
            
            let trimmed_text = text.trim();
            if !trimmed_text.is_empty() {
                Ok(Some(self.create_text_node(trimmed_text.to_string())))
            } else {
                Ok(None)
            }
        } else {
            // No more tags - this might be incomplete text
            // Only process if we're finalizing
            Ok(None)
        }
    }

    /// Parse tag name and attributes from tag content
    fn parse_tag_name_and_attributes(&self, tag_content: &str) -> (String, Vec<(String, String)>) {
        let mut parts = tag_content.split_whitespace();
        let tag_name = parts.next().unwrap_or("").to_lowercase();
        
        let mut attributes = Vec::new();
        for part in parts {
            if let Some(eq_pos) = part.find('=') {
                let name = part[..eq_pos].to_lowercase();
                let value = part[eq_pos+1..].trim_matches('"').trim_matches('\'').to_string();
                attributes.push((name, value));
            } else {
                // Boolean attribute
                attributes.push((part.to_lowercase(), String::new()));
            }
        }
        
        (tag_name, attributes)
    }

    /// Create an element DOM node
    fn create_element_node(&self, tag_name: String) -> DOMNode {
        DOMNode::new_element(tag_name)
    }

    /// Create a text DOM node
    fn create_text_node(&self, text: String) -> DOMNode {
        DOMNode::new_text(text)
    }

    /// Parse any remaining content when finalizing
    fn parse_remaining(&mut self) -> Result<ParsedChunk> {
        let mut nodes = Vec::new();
        let mut text_content = String::new();
        
        // Parse any remaining complete elements
        while self.parse_position < self.buffer.len() {
            if let Some(node) = self.try_parse_next_element()? {
                nodes.push(node.clone());
                
                if let Some(text) = self.extract_text_content(&node) {
                    text_content.push_str(&text);
                    text_content.push(' ');
                }
            } else {
                break;
            }
        }
        
        // Handle any remaining text
        if self.parse_position < self.buffer.len() {
            let remaining_text = &self.buffer[self.parse_position..];
            let trimmed = remaining_text.trim();
            if !trimmed.is_empty() {
                nodes.push(self.create_text_node(trimmed.to_string()));
                text_content.push_str(trimmed);
            }
        }
        
        self.partial_dom.extend(nodes.iter().cloned());
        
        Ok(ParsedChunk {
            nodes,
            progress: ParseProgress {
                processed_bytes: self.processed_bytes,
                total_bytes: self.total_bytes,
                nodes_created: self.partial_dom.len(),
                current_tag: self.tag_stack.last().cloned(),
                is_complete: true,
            },
            text_content,
        })
    }

    /// Extract text content from a DOM node
    fn extract_text_content(&self, node: &DOMNode) -> Option<String> {
        match node {
            DOMNode::Text(text) => Some(text.clone()),
            DOMNode::Element { tag_name, .. } => {
                // For certain tags, extract meaningful text
                match tag_name.as_str() {
                    "title" | "h1" | "h2" | "h3" | 
                    "h4" | "h5" | "h6" | "p" | 
                    "a" | "span" | "div" => {
                        Some(format!("[{}]", tag_name))
                    },
                    _ => None
                }
            },
            DOMNode::Comment(_) => None, // Comments don't contribute to text content
        }
    }

    /// Get current parsing progress
    fn get_progress(&self) -> ParseProgress {
        ParseProgress {
            processed_bytes: self.processed_bytes,
            total_bytes: self.total_bytes,
            nodes_created: self.partial_dom.len(),
            current_tag: self.tag_stack.last().cloned(),
            is_complete: false,
        }
    }

    /// Get all parsed nodes so far
    pub fn get_partial_dom(&self) -> &[DOMNode] {
        &self.partial_dom
    }

    /// Get current parsing progress without parsing more
    pub fn current_progress(&self) -> ParseProgress {
        self.get_progress()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_parser_basic() {
        let mut parser = StreamingHtmlParser::new(1024);
        
        let chunk1 = "<html><head><title>Test";
        let chunk2 = " Page</title></head><body>";
        let chunk3 = "<h1>Hello World</h1></body></html>";
        
        parser.set_total_size(chunk1.len() + chunk2.len() + chunk3.len());
        
        let result1 = parser.add_chunk(chunk1).unwrap();
        let result2 = parser.add_chunk(chunk2).unwrap();
        let result3 = parser.add_chunk(chunk3).unwrap();
        
        let final_result = parser.finalize().unwrap();
        assert!(final_result.progress.is_complete);
        assert!(!final_result.nodes.is_empty());
    }

    #[test]
    fn test_text_extraction() {
        let mut parser = StreamingHtmlParser::new(1024);
        let html = "<p>Hello</p><div>World</div>";
        
        let _result = parser.add_chunk(html).unwrap();
        let final_result = parser.finalize().unwrap();
        
        assert!(final_result.text_content.contains("Hello"));
        assert!(final_result.text_content.contains("World"));
    }
}