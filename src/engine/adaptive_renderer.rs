use std::rc::Rc;
use std::cell::RefCell;
use anyhow::{Result, anyhow};
use crate::engine::{
    dom::DOMNode,
    streaming_parser::{StreamingHtmlParser, ParsedChunk, ParseProgress},
    virtual_scroll::{VirtualViewport, VirtualScrollConfig},
};
use crate::networking::HttpResponse;

/// Different rendering strategies based on content size and complexity
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderingMode {
    /// Full HTML parsing and layout (< 25KB)
    Full,
    /// Progressive parsing with prioritized rendering (25KB - 500KB)
    Enhanced,
    /// Virtual scrolling with lazy DOM creation (500KB - 5MB)
    Streaming,
    /// Text-focused with minimal styling (5MB - 50MB)
    Efficient,
    /// Plain text mode with link extraction (> 50MB)
    Minimal,
}

/// Configuration for adaptive rendering
#[derive(Debug, Clone)]
pub struct AdaptiveRenderConfig {
    pub full_threshold: usize,      // 25KB
    pub enhanced_threshold: usize,   // 500KB
    pub streaming_threshold: usize,  // 5MB
    pub efficient_threshold: usize,  // 50MB
    pub chunk_size: usize,          // For streaming parsing
    pub virtual_scroll_config: VirtualScrollConfig,
}

impl Default for AdaptiveRenderConfig {
    fn default() -> Self {
        Self {
            full_threshold: 25 * 1024,           // 25KB
            enhanced_threshold: 500 * 1024,      // 500KB
            streaming_threshold: 5 * 1024 * 1024, // 5MB
            efficient_threshold: 50 * 1024 * 1024, // 50MB
            chunk_size: 8192,                    // 8KB chunks
            virtual_scroll_config: VirtualScrollConfig::default(),
        }
    }
}

/// Adaptive rendering engine that chooses the best strategy
pub struct AdaptiveRenderer {
    config: AdaptiveRenderConfig,
    current_mode: Option<RenderingMode>,
    streaming_parser: Option<StreamingHtmlParser>,
    virtual_viewport: Option<VirtualViewport>,
    processed_chunks: Vec<ParsedChunk>,
    content_buffer: String,
}

/// Result of adaptive rendering
#[derive(Debug, Clone)]
pub struct AdaptiveRenderResult {
    pub mode: RenderingMode,
    pub content: RenderContent,
    pub progress: Option<ParseProgress>,
    pub is_complete: bool,
    pub performance_metrics: PerformanceMetrics,
}

/// Different types of rendered content
#[derive(Debug, Clone)]
pub enum RenderContent {
    /// Full DOM tree with all features
    FullDOM {
        dom_text: String,  // Serialized DOM representation
        extracted_text: String,
    },
    /// Progressive DOM with chunks
    ProgressiveDOM {
        completed_chunks: Vec<ParsedChunk>,
        current_text: String,
    },
    /// Virtual scrolling viewport
    VirtualScrolling {
        viewport_config: VirtualScrollConfig,
        total_nodes: usize,
    },
    /// Simplified text with basic formatting
    SimplifiedText {
        formatted_text: String,
        links: Vec<ExtractedLink>,
        headings: Vec<ExtractedHeading>,
    },
    /// Plain text with minimal processing
    PlainText {
        text: String,
        word_count: usize,
    },
}

/// Performance metrics for rendering
#[derive(Debug, Default, Clone)]
pub struct PerformanceMetrics {
    pub parse_time_ms: u64,
    pub render_time_ms: u64,
    pub memory_usage_kb: usize,
    pub nodes_created: usize,
    pub content_size_bytes: usize,
}

/// Extracted link information
#[derive(Debug, Clone)]
pub struct ExtractedLink {
    pub text: String,
    pub url: String,
    pub position: usize,
}

/// Extracted heading information
#[derive(Debug, Clone)]
pub struct ExtractedHeading {
    pub text: String,
    pub level: u8, // 1-6 for h1-h6
    pub position: usize,
}

impl AdaptiveRenderer {
    /// Create a new adaptive renderer
    pub fn new(config: AdaptiveRenderConfig) -> Self {
        Self {
            config,
            current_mode: None,
            streaming_parser: None,
            virtual_viewport: None,
            processed_chunks: Vec::new(),
            content_buffer: String::new(),
        }
    }

    /// Create with default configuration
    pub fn new_default() -> Self {
        Self::new(AdaptiveRenderConfig::default())
    }

    /// Determine the appropriate rendering mode for given content
    pub fn determine_mode(&self, content_size: usize, content_type: Option<&str>) -> RenderingMode {
        // Consider content type
        if let Some(content_type) = content_type {
            if !content_type.contains("text/html") && !content_type.contains("text/plain") {
                return RenderingMode::Minimal;
            }
        }

        // Size-based determination
        if content_size < self.config.full_threshold {
            RenderingMode::Full
        } else if content_size < self.config.enhanced_threshold {
            RenderingMode::Enhanced
        } else if content_size < self.config.streaming_threshold {
            RenderingMode::Streaming
        } else if content_size < self.config.efficient_threshold {
            RenderingMode::Efficient
        } else {
            RenderingMode::Minimal
        }
    }

    /// Start rendering with adaptive strategy
    pub fn start_render(&mut self, response: &HttpResponse) -> Result<AdaptiveRenderResult> {
        let content_size = response.body.len();
        let content_type = response.headers.get("content-type").map(|s| s.as_str());
        let mode = self.determine_mode(content_size, content_type);
        
        self.current_mode = Some(mode);
        
        let start_time = std::time::Instant::now();
        
        let result = match mode {
            RenderingMode::Full => self.render_full(response),
            RenderingMode::Enhanced => self.render_enhanced(response),
            RenderingMode::Streaming => self.render_streaming(response),
            RenderingMode::Efficient => self.render_efficient(response),
            RenderingMode::Minimal => self.render_minimal(response),
        }?;
        
        let render_time = start_time.elapsed().as_millis() as u64;
        
        Ok(AdaptiveRenderResult {
            mode,
            content: result,
            progress: self.get_current_progress(),
            is_complete: true,
            performance_metrics: PerformanceMetrics {
                render_time_ms: render_time,
                content_size_bytes: content_size,
                ..Default::default()
            },
        })
    }

    /// Add content chunk for progressive rendering
    pub fn add_content_chunk(&mut self, chunk: &str) -> Result<Option<AdaptiveRenderResult>> {
        if let Some(mode) = self.current_mode {
            match mode {
                RenderingMode::Enhanced | RenderingMode::Streaming => {
                    if let Some(ref mut parser) = self.streaming_parser {
                        if let Some(parsed_chunk) = parser.add_chunk(chunk)? {
                            self.processed_chunks.push(parsed_chunk);
                            
                            return Ok(Some(AdaptiveRenderResult {
                                mode,
                                content: self.get_progressive_content()?,
                                progress: self.get_current_progress(),
                                is_complete: false,
                                performance_metrics: PerformanceMetrics::default(),
                            }));
                        }
                    }
                }
                _ => {
                    self.content_buffer.push_str(chunk);
                }
            }
        }
        
        Ok(None)
    }

    /// Finalize rendering
    pub fn finalize(&mut self) -> Result<AdaptiveRenderResult> {
        if let Some(mode) = self.current_mode {
            let content = match mode {
                RenderingMode::Enhanced | RenderingMode::Streaming => {
                    if let Some(ref mut parser) = self.streaming_parser {
                        let final_chunk = parser.finalize()?;
                        self.processed_chunks.push(final_chunk);
                    }
                    self.get_progressive_content()?
                }
                _ => {
                    // Handle buffered content for other modes
                    self.render_buffered_content(mode)?
                }
            };
            
            Ok(AdaptiveRenderResult {
                mode,
                content,
                progress: self.get_current_progress(),
                is_complete: true,
                performance_metrics: PerformanceMetrics::default(),
            })
        } else {
            Err(anyhow!("No rendering mode set"))
        }
    }

    /// Render with full HTML parsing and layout
    fn render_full(&mut self, response: &HttpResponse) -> Result<RenderContent> {
        let html = response.body_as_string()?;
        
        // Use existing HTML parser for full rendering
        let dom_nodes = self.parse_html_simple(&html)?;
        let extracted_text = self.extract_text_from_nodes(&dom_nodes);
        
        Ok(RenderContent::FullDOM {
            dom_text: format!("{} DOM nodes created", dom_nodes.len()),
            extracted_text,
        })
    }

    /// Render with progressive parsing and prioritized content
    fn render_enhanced(&mut self, response: &HttpResponse) -> Result<RenderContent> {
        let html = response.body_as_string()?;
        
        // Initialize streaming parser
        self.streaming_parser = Some(StreamingHtmlParser::new(self.config.chunk_size));
        if let Some(ref mut parser) = self.streaming_parser {
            parser.set_total_size(html.len());
            
            // Process in chunks
            let chunk_size = self.config.chunk_size;
            for chunk in html.as_bytes().chunks(chunk_size) {
                let chunk_str = String::from_utf8_lossy(chunk);
                if let Some(parsed_chunk) = parser.add_chunk(&chunk_str)? {
                    self.processed_chunks.push(parsed_chunk);
                }
            }
            
            let final_chunk = parser.finalize()?;
            self.processed_chunks.push(final_chunk);
        }
        
        self.get_progressive_content()
    }

    /// Render with virtual scrolling for massive content
    fn render_streaming(&mut self, response: &HttpResponse) -> Result<RenderContent> {
        let html = response.body_as_string()?;
        
        // Parse content to get node count estimate
        let node_count = self.estimate_node_count(&html);
        
        // Initialize virtual viewport
        self.virtual_viewport = Some(VirtualViewport::new(
            self.config.virtual_scroll_config.clone()
        ));
        
        // Parse content for virtual scrolling
        let dom_nodes = self.parse_html_simple(&html)?;
        
        Ok(RenderContent::VirtualScrolling {
            viewport_config: self.config.virtual_scroll_config.clone(),
            total_nodes: dom_nodes.len(),
        })
    }

    /// Render with text-focused approach
    fn render_efficient(&mut self, response: &HttpResponse) -> Result<RenderContent> {
        let html = response.body_as_string()?;
        
        let formatted_text = self.extract_formatted_text(&html);
        let links = self.extract_links(&html);
        let headings = self.extract_headings(&html);
        
        Ok(RenderContent::SimplifiedText {
            formatted_text,
            links,
            headings,
        })
    }

    /// Render with minimal processing
    fn render_minimal(&mut self, response: &HttpResponse) -> Result<RenderContent> {
        let text = if response.headers.get("content-type")
            .map_or(false, |ct| ct.contains("text/html")) {
            self.strip_html_tags(&response.body_as_string()?)
        } else {
            response.body_as_string()?
        };
        
        let word_count = text.split_whitespace().count();
        
        Ok(RenderContent::PlainText {
            text,
            word_count,
        })
    }

    /// Get progressive content from processed chunks
    fn get_progressive_content(&self) -> Result<RenderContent> {
        let current_text = self.processed_chunks
            .iter()
            .map(|chunk| chunk.text_content.clone())
            .collect::<Vec<_>>()
            .join(" ");
        
        Ok(RenderContent::ProgressiveDOM {
            completed_chunks: self.processed_chunks.clone(),
            current_text,
        })
    }

    /// Render buffered content for non-streaming modes
    fn render_buffered_content(&self, mode: RenderingMode) -> Result<RenderContent> {
        match mode {
            RenderingMode::Full => {
                let dom_nodes = self.parse_html_simple(&self.content_buffer)?;
                let extracted_text = self.extract_text_from_nodes(&dom_nodes);
                Ok(RenderContent::FullDOM { dom_text: format!("{} DOM nodes", dom_nodes.len()), extracted_text })
            }
            RenderingMode::Efficient => {
                let formatted_text = self.extract_formatted_text(&self.content_buffer);
                let links = self.extract_links(&self.content_buffer);
                let headings = self.extract_headings(&self.content_buffer);
                Ok(RenderContent::SimplifiedText { formatted_text, links, headings })
            }
            RenderingMode::Minimal => {
                let text = self.strip_html_tags(&self.content_buffer);
                let word_count = text.split_whitespace().count();
                Ok(RenderContent::PlainText { text, word_count })
            }
            _ => Err(anyhow!("Unsupported buffered mode: {:?}", mode))
        }
    }

    /// Simple HTML parsing (placeholder for now)
    fn parse_html_simple(&self, html: &str) -> Result<Vec<Rc<RefCell<DOMNode>>>> {
        // This is a simplified parser - in a real implementation,
        // you'd use the existing HTML parser or create a more sophisticated one
        Ok(Vec::new())
    }

    /// Extract text content from DOM nodes
    fn extract_text_from_nodes(&self, _nodes: &[Rc<RefCell<DOMNode>>]) -> String {
        // Placeholder implementation
        String::new()
    }

    /// Estimate number of DOM nodes from HTML
    fn estimate_node_count(&self, html: &str) -> usize {
        // Simple estimation based on tag count
        html.matches('<').count().max(100)
    }

    /// Extract formatted text with basic structure
    fn extract_formatted_text(&self, html: &str) -> String {
        // Simple implementation - strip tags but preserve structure
        self.strip_html_tags(html)
    }

    /// Extract links from HTML
    fn extract_links(&self, html: &str) -> Vec<ExtractedLink> {
        let mut links = Vec::new();
        let mut position = 0;
        
        // Simple regex-like extraction (simplified for demo)
        for line in html.lines() {
            if line.contains("<a ") && line.contains("href=") {
                // Extract link (simplified)
                if let Some(start) = line.find("href=\"") {
                    if let Some(end) = line[start + 6..].find("\"") {
                        let url = line[start + 6..start + 6 + end].to_string();
                        let text = line.split(">").nth(1)
                            .and_then(|s| s.split("<").next())
                            .unwrap_or("Link")
                            .to_string();
                        
                        links.push(ExtractedLink {
                            text,
                            url,
                            position,
                        });
                    }
                }
            }
            position += line.len();
        }
        
        links
    }

    /// Extract headings from HTML
    fn extract_headings(&self, html: &str) -> Vec<ExtractedHeading> {
        let mut headings = Vec::new();
        let mut position = 0;
        
        for line in html.lines() {
            for level in 1..=6 {
                let open_tag = format!("<h{}", level);
                let close_tag = format!("</h{}>", level);
                
                if line.contains(&open_tag) {
                    if let Some(start) = line.find(">") {
                        if let Some(end) = line.find(&close_tag) {
                            let text = line[start + 1..end].to_string();
                            headings.push(ExtractedHeading {
                                text,
                                level: level as u8,
                                position,
                            });
                        }
                    }
                }
            }
            position += line.len();
        }
        
        headings
    }

    /// Strip HTML tags to get plain text
    fn strip_html_tags(&self, html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;
        
        for ch in html.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => result.push(ch),
                _ => {}
            }
        }
        
        // Clean up whitespace
        result.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get current parsing progress
    fn get_current_progress(&self) -> Option<ParseProgress> {
        self.streaming_parser.as_ref()
            .map(|parser| parser.current_progress())
    }

    /// Get current rendering mode
    pub fn current_mode(&self) -> Option<RenderingMode> {
        self.current_mode
    }

    /// Reset renderer state
    pub fn reset(&mut self) {
        self.current_mode = None;
        self.streaming_parser = None;
        self.virtual_viewport = None;
        self.processed_chunks.clear();
        self.content_buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::networking::HttpResponse;
    use std::collections::HashMap;

    #[test]
    fn test_mode_determination() {
        let renderer = AdaptiveRenderer::new_default();
        
        assert_eq!(renderer.determine_mode(10 * 1024, Some("text/html")), RenderingMode::Full);
        assert_eq!(renderer.determine_mode(100 * 1024, Some("text/html")), RenderingMode::Enhanced);
        assert_eq!(renderer.determine_mode(1024 * 1024, Some("text/html")), RenderingMode::Streaming);
        assert_eq!(renderer.determine_mode(10 * 1024 * 1024, Some("text/html")), RenderingMode::Efficient);
        assert_eq!(renderer.determine_mode(100 * 1024 * 1024, Some("text/html")), RenderingMode::Minimal);
    }

    #[test]
    fn test_html_tag_stripping() {
        let renderer = AdaptiveRenderer::new_default();
        let html = "<html><body><h1>Title</h1><p>Content</p></body></html>";
        let stripped = renderer.strip_html_tags(html);
        
        assert_eq!(stripped, "Title Content");
    }
}