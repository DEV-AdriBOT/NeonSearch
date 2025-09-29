pub mod html_parser;
pub mod css_parser;
pub mod layout;
pub mod renderer;
pub mod dom;
pub mod streaming_parser;
pub mod virtual_scroll;
pub mod adaptive_renderer;
pub mod content_analyzer;
pub mod background_processor;

use eframe::egui;
use self::dom::DOMNode;

pub struct WebPage {
    pub dom: DOMNode,
    pub stylesheets: Vec<css_parser::Stylesheet>,
    pub layout_tree: Option<layout::LayoutBox>,
    pub raw_html: Option<String>,
    pub plain_text: Option<String>,
    pub extracted_title: Option<String>,
    pub loading_progress: Option<LoadingProgress>,
    pub content_size: usize,
    pub is_large_content: bool,
}

/// Progress tracking for large website loading
#[derive(Debug, Clone)]
pub struct LoadingProgress {
    pub phase: LoadingPhase,
    pub bytes_downloaded: usize,
    pub total_bytes: Option<usize>,
    pub nodes_parsed: usize,
    pub progress_percentage: f32,
    pub status_message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadingPhase {
    Connecting,
    Downloading,
    Parsing,
    Rendering,
    Complete,
}

impl WebPage {
    pub fn create_home_page() -> Self {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>NeonSearch Home</title>
                <style>
                    body { 
                        font-family: Arial, sans-serif; 
                        background: linear-gradient(135deg, #1e1e2e, #2a2a3e);
                        color: #ffffff;
                        text-align: center;
                        padding: 40px;
                    }
                    .logo { 
                        font-size: 3em; 
                        color: #00ffcc;
                        margin-bottom: 20px;
                    }
                    .subtitle {
                        font-size: 1.2em;
                        color: #cccccc;
                        margin-bottom: 40px;
                    }
                    .quick-links {
                        display: flex;
                        justify-content: center;
                        gap: 20px;
                        margin-top: 40px;
                    }
                    .quick-link {
                        background: rgba(0, 255, 204, 0.1);
                        border: 1px solid #00ffcc;
                        padding: 15px 25px;
                        border-radius: 8px;
                        text-decoration: none;
                        color: #00ffcc;
                        transition: all 0.3s ease;
                    }
                    .quick-link:hover {
                        background: rgba(0, 255, 204, 0.2);
                    }
                </style>
            </head>
            <body>
                <div class="logo">🚀 NeonSearch</div>
                <div class="subtitle">Fast, Secure, and Built from Scratch by NeonDev™</div>
                
                <div style="max-width: 600px; margin: 0 auto;">
                    <h2 style="color: #00ffcc;">Welcome to the Future of Browsing</h2>
                    <p>NeonSearch is a revolutionary web browser built entirely from scratch using Rust. 
                    Experience the web with unparalleled performance, security, and innovation.</p>
                    
                    <div class="quick-links">
                        <a href="https://github.com/neondev" class="quick-link">
                            🐙 GitHub
                        </a>
                        <a href="https://www.rust-lang.org" class="quick-link">
                            🦀 Rust Lang
                        </a>
                        <a href="about:blank" class="quick-link">
                            📄 New Tab
                        </a>
                    </div>
                </div>
                
                <div style="margin-top: 60px; font-size: 0.9em; color: #666;">
                    <p>Powered by NeonDev™ | Built with ❤️ and Rust 🦀</p>
                </div>
            </body>
            </html>
        "#;
        
        Self::from_html(html)
    }
    
    pub fn create_blank_page() -> Self {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Blank Page</title>
                <style>
                    body { 
                        background-color: #1e1e2e; 
                        margin: 0; 
                        padding: 0; 
                    }
                </style>
            </head>
            <body></body>
            </html>
        "#;
        
        Self::from_html(html)
    }
    
    pub fn create_loading_page(url: &str) -> Self {
        let html = format!(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Loading...</title>
                <style>
                    body {{ 
                        font-family: Arial, sans-serif; 
                        background: #1e1e2e;
                        color: #ffffff;
                        display: flex;
                        justify-content: center;
                        align-items: center;
                        height: 100vh;
                        margin: 0;
                    }}
                    .loading-container {{
                        text-align: center;
                    }}
                    .spinner {{
                        width: 50px;
                        height: 50px;
                        border: 3px solid #333;
                        border-top: 3px solid #00ffcc;
                        border-radius: 50%;
                        animation: spin 1s linear infinite;
                        margin: 0 auto 20px auto;
                    }}
                    @keyframes spin {{
                        0% {{ transform: rotate(0deg); }}
                        100% {{ transform: rotate(360deg); }}
                    }}
                </style>
            </head>
            <body>
                <div class="loading-container">
                    <div class="spinner"></div>
                    <h2>Loading {}</h2>
                    <p>Please wait while we fetch the content...</p>
                </div>
            </body>
            </html>
        "#, url);
        
        Self::from_html(&html)
    }
    
    pub fn create_loading_page_with_progress(url: &str, progress: LoadingProgress) -> Self {
        let progress_bar = if progress.progress_percentage > 0.0 {
            format!(r#"
                <div class="progress-container">
                    <div class="progress-bar">
                        <div class="progress-fill" style="width: {:.1}%"></div>
                    </div>
                    <div class="progress-text">{}</div>
                    <div class="progress-details">{:.1}% - {}</div>
                </div>
            "#, 
            progress.progress_percentage,
            progress.status_message,
            progress.progress_percentage,
            match progress.phase {
                LoadingPhase::Connecting => "Connecting to server...",
                LoadingPhase::Downloading => "Downloading content...",
                LoadingPhase::Parsing => "Parsing HTML...",
                LoadingPhase::Rendering => "Rendering page...",
                LoadingPhase::Complete => "Complete!",
            })
        } else {
            "<div class=\"progress-text\">Starting...</div>".to_string()
        };

        let html = format!(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Loading...</title>
                <style>
                    body {{ 
                        font-family: Arial, sans-serif; 
                        background: #1e1e2e;
                        color: #ffffff;
                        display: flex;
                        justify-content: center;
                        align-items: center;
                        height: 100vh;
                        margin: 0;
                    }}
                    .loading-container {{
                        text-align: center;
                        max-width: 400px;
                    }}
                    .spinner {{
                        width: 50px;
                        height: 50px;
                        border: 3px solid #333;
                        border-top: 3px solid #00ffcc;
                        border-radius: 50%;
                        animation: spin 1s linear infinite;
                        margin: 0 auto 20px auto;
                    }}
                    .progress-container {{
                        margin: 20px 0;
                        text-align: center;
                    }}
                    .progress-bar {{
                        width: 100%;
                        height: 10px;
                        background: #333;
                        border-radius: 5px;
                        overflow: hidden;
                        margin: 10px 0;
                    }}
                    .progress-fill {{
                        height: 100%;
                        background: linear-gradient(90deg, #00ffcc, #00cc99);
                        transition: width 0.3s ease;
                    }}
                    .progress-text {{
                        color: #00ffcc;
                        font-weight: bold;
                        margin: 10px 0;
                    }}
                    .progress-details {{
                        color: #cccccc;
                        font-size: 0.9em;
                    }}
                    @keyframes spin {{
                        0% {{ transform: rotate(0deg); }}
                        100% {{ transform: rotate(360deg); }}
                    }}
                </style>
            </head>
            <body>
                <div class="loading-container">
                    <div class="spinner"></div>
                    <h2>Loading {}</h2>
                    <p>Please wait while we fetch the content...</p>
                    {}
                </div>
            </body>
            </html>
        "#, url, progress_bar);
        
        let mut page = Self::from_html(&html);
        page.loading_progress = Some(progress);
        page
    }
    
    pub fn create_error_page(url: &str, error: &str) -> Self {
        let html = format!(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Error Loading Page</title>
                <style>
                    body {{ 
                        font-family: Arial, sans-serif; 
                        background: linear-gradient(135deg, #2e1e1e, #3e2a2a);
                        color: #ffffff;
                        text-align: center;
                        padding: 40px;
                        margin: 0;
                    }}
                    .error-container {{
                        max-width: 600px;
                        margin: 0 auto;
                        background: rgba(255, 0, 0, 0.1);
                        border: 1px solid #ff6b6b;
                        border-radius: 10px;
                        padding: 40px;
                    }}
                    .error-icon {{
                        font-size: 4em;
                        color: #ff6b6b;
                        margin-bottom: 20px;
                    }}
                    h1 {{
                        color: #ff6b6b;
                        margin-bottom: 20px;
                    }}
                    .error-details {{
                        background: rgba(0, 0, 0, 0.3);
                        border-radius: 5px;
                        padding: 15px;
                        margin: 20px 0;
                        font-family: monospace;
                        font-size: 0.9em;
                    }}
                    .retry-hint {{
                        color: #cccccc;
                        font-size: 0.9em;
                        margin-top: 20px;
                    }}
                </style>
            </head>
            <body>
                <div class="error-container">
                    <div class="error-icon">⚠️</div>
                    <h1>Failed to Load Page</h1>
                    <p>We couldn't load the page at:</p>
                    <div class="error-details">{}</div>
                    <p><strong>Error:</strong> {}</p>
                    <div class="retry-hint">
                        Try checking your internet connection or reloading the page.
                    </div>
                </div>
            </body>
            </html>
        "#, url, error);
        
        Self::from_html(&html)
    }
    
    pub fn create_simple_text_page(content: &str, url: &str) -> Self {
        // Extract title quickly without full HTML parsing
        let title = if let Some(start) = content.find("<title>") {
            if let Some(end) = content[start + 7..].find("</title>") {
                content[start + 7..start + 7 + end].to_string()
            } else {
                format!("Large Site: {}", url)
            }
        } else {
            format!("Large Site: {}", url)
        };
        
        // Create a simple but renderable HTML structure
        let simple_html = format!(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>{}</title>
                <style>
                    body {{ 
                        font-family: Arial, sans-serif; 
                        line-height: 1.6; 
                        margin: 20px;
                        background: #1a1a1a;
                        color: #ffffff;
                    }}
                    .preview-warning {{
                        background: #ff6b35;
                        color: white;
                        padding: 15px;
                        border-radius: 8px;
                        margin-bottom: 20px;
                        border: 1px solid #ff8c5a;
                    }}
                    .content {{
                        white-space: pre-wrap;
                        word-wrap: break-word;
                        font-family: monospace;
                        background: #2a2a2a;
                        padding: 15px;
                        border-radius: 8px;
                        max-height: 800px;
                        overflow-y: auto;
                    }}
                </style>
            </head>
            <body>
                <div class="preview-warning">
                    <strong>⚡ Large Site Preview Mode</strong><br>
                    This is a large site loaded in preview mode for better performance.<br>
                    Showing first 25KB of content from: <code>{}</code>
                </div>
                <div class="content">{}</div>
            </body>
            </html>
        "#, title, url, Self::escape_html(content));
        
        // Use the regular from_html method but with our simplified structure
        Self::from_html(&simple_html)
    }
    
    // Helper method to escape HTML content for safe display
    fn escape_html(content: &str) -> String {
        content
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
    
    pub fn from_html(html: &str) -> Self {
        let content_size = html.len();
        let is_large_content = content_size > 25 * 1024; // 25KB threshold
        
        // Limit HTML size to prevent crashes (5MB limit for large content support)
        let limited_html = if html.len() > 5 * 1024 * 1024 {
            let truncated = &html[..5 * 1024 * 1024];
            format!("{}<br><br><i>[Content truncated at 5MB to prevent crashes]</i>", truncated)
        } else {
            html.to_string()
        };
        
        let dom = html_parser::parse(&limited_html);
        let stylesheets = Vec::new(); // TODO: Parse CSS from <style> tags and external stylesheets
        let title = extract_title(&limited_html);
        let plain = strip_html(&limited_html);
        
        // Create progress indicator for large content
        let loading_progress = if is_large_content {
            Some(LoadingProgress {
                phase: LoadingPhase::Complete,
                bytes_downloaded: content_size,
                total_bytes: Some(content_size),
                nodes_parsed: 0, // TODO: Count actual nodes
                progress_percentage: 100.0,
                status_message: format!("Loaded large content: {:.1}KB", content_size as f32 / 1024.0),
            })
        } else {
            None
        };
        
        Self {
            dom,
            stylesheets,
            layout_tree: None,
            raw_html: Some(limited_html.clone()),
            plain_text: Some(plain),
            extracted_title: title,
            loading_progress,
            content_size,
            is_large_content,
        }
    }
    
    pub fn render(&self, ui: &mut egui::Ui) {
        // Show progress indicator for large content if loading
        if let Some(progress) = &self.loading_progress {
            if progress.phase != LoadingPhase::Complete {
                self.render_progress_indicator(ui, progress);
                return;
            }
        }
        
        // Show large content indicator if applicable
        if self.is_large_content {
            self.render_large_content_header(ui);
        }
        
        // For now, render a simplified version of the DOM
        self.render_dom_node(ui, &self.dom);
    }
    
    fn render_progress_indicator(&self, ui: &mut egui::Ui, progress: &LoadingProgress) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            
            // Loading spinner
            ui.ctx().request_repaint();
            let spinner_rect = ui.available_rect_before_wrap();
            let center = spinner_rect.center();
            let radius = 20.0;
            let time = ui.ctx().input(|i| i.time) as f32;
            
            ui.painter().circle(
                center,
                radius,
                egui::Color32::from_rgb(30, 30, 46),
                egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 255, 204))
            );
            
            // Progress bar
            ui.add_space(30.0);
            ui.heading(&progress.status_message);
            
            let progress_bar = egui::ProgressBar::new(progress.progress_percentage / 100.0)
                .text(format!("{:.1}%", progress.progress_percentage))
                .fill(egui::Color32::from_rgb(0, 255, 204));
            ui.add(progress_bar);
            
            // Phase indicator
            ui.add_space(10.0);
            ui.label(match progress.phase {
                LoadingPhase::Connecting => "🔗 Connecting to server...",
                LoadingPhase::Downloading => "⬇️ Downloading content...",
                LoadingPhase::Parsing => "📝 Parsing HTML...",
                LoadingPhase::Rendering => "🎨 Rendering page...",
                LoadingPhase::Complete => "✅ Complete!",
            });
            
            // Size information
            if let Some(total) = progress.total_bytes {
                ui.label(format!("Downloaded: {:.1}KB / {:.1}KB", 
                    progress.bytes_downloaded as f32 / 1024.0,
                    total as f32 / 1024.0));
            } else {
                ui.label(format!("Downloaded: {:.1}KB", 
                    progress.bytes_downloaded as f32 / 1024.0));
            }
        });
    }
    
    fn render_large_content_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("⚡");
            ui.label(format!("Large Content ({:.1}KB) - Enhanced rendering active", 
                self.content_size as f32 / 1024.0));
        });
        ui.separator();
        ui.add_space(5.0);
    }
    
    fn render_dom_node(&self, ui: &mut egui::Ui, node: &DOMNode) {
        use crate::ui::theme::NeonTheme;
        
        match node {
            DOMNode::Element { tag_name, attributes, children } => {
                match tag_name.as_str() {
                    "html" | "body" => {
                        // Render children directly
                        for child in children {
                            self.render_dom_node(ui, child);
                        }
                    }
                    "div" => {
                        // Enhanced div rendering with better styling
                        let style_attr = attributes.get("style").cloned();
                        let class_attr = attributes.get("class").cloned();
                        
                        if let Some(style) = style_attr {
                            if style.contains("text-align: center") {
                                ui.centered_and_justified(|ui| {
                                    for child in children {
                                        self.render_dom_node(ui, child);
                                    }
                                });
                            } else {
                                egui::Frame::none()
                                    .inner_margin(egui::Margin::same(8.0))
                                    .show(ui, |ui| {
                                        for child in children {
                                            self.render_dom_node(ui, child);
                                        }
                                    });
                            }
                        } else {
                            ui.vertical(|ui| {
                                for child in children {
                                    self.render_dom_node(ui, child);
                                }
                            });
                        }
                    }
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        let (size, color) = match tag_name.as_str() {
                            "h1" => (28.0, NeonTheme::NEON_CYAN),
                            "h2" => (24.0, NeonTheme::NEON_BLUE),
                            "h3" => (20.0, NeonTheme::PRIMARY_TEXT),
                            "h4" => (18.0, NeonTheme::PRIMARY_TEXT),
                            "h5" => (16.0, NeonTheme::SECONDARY_TEXT),
                            "h6" => (14.0, NeonTheme::SECONDARY_TEXT),
                            _ => (16.0, NeonTheme::PRIMARY_TEXT),
                        };
                        
                        let text = self.extract_text(node);
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new(text).size(size).strong().color(color));
                        ui.add_space(4.0);
                    }
                    "p" => {
                        let text = self.extract_text(node);
                        if !text.trim().is_empty() {
                            ui.label(
                                egui::RichText::new(text)
                                    .size(14.0)
                                    .color(NeonTheme::PRIMARY_TEXT)
                            );
                            ui.add_space(8.0);
                        }
                    }
                    "a" => {
                        let text = self.extract_text(node);
                        let href = attributes.get("href").cloned().unwrap_or_default();
                        
                        if !text.trim().is_empty() {
                            let link = ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&text)
                                        .color(NeonTheme::NEON_BLUE)
                                        .underline()
                                )
                                .sense(egui::Sense::click())
                            );
                            
                            if link.clicked() {
                                log::info!("Link clicked: {} -> {}", text, href);
                                // TODO: Implement navigation callback
                            }
                            
                            if link.hovered() {
                                link.on_hover_text(format!("Navigate to: {}", href));
                            }
                        }
                    }
                    "strong" | "b" => {
                        let text = self.extract_text(node);
                        if !text.trim().is_empty() {
                            ui.label(
                                egui::RichText::new(text)
                                    .strong()
                                    .color(NeonTheme::PRIMARY_TEXT)
                            );
                        }
                    }
                    "em" | "i" => {
                        let text = self.extract_text(node);
                        if !text.trim().is_empty() {
                            ui.label(
                                egui::RichText::new(text)
                                    .italics()
                                    .color(NeonTheme::SECONDARY_TEXT)
                            );
                        }
                    }
                    "code" => {
                        let text = self.extract_text(node);
                        if !text.trim().is_empty() {
                            ui.label(
                                egui::RichText::new(text)
                                    .monospace()
                                    .background_color(NeonTheme::ELEVATED_BG)
                                    .color(NeonTheme::NEON_GREEN)
                            );
                        }
                    }
                    "pre" => {
                        let text = self.extract_text(node);
                        if !text.trim().is_empty() {
                            egui::Frame::none()
                                .fill(NeonTheme::DARKER_BG)
                                .stroke(egui::Stroke::new(1.0, NeonTheme::BORDER_COLOR))
                                .rounding(egui::Rounding::same(4.0))
                                .inner_margin(egui::Margin::same(8.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(text)
                                            .monospace()
                                            .color(NeonTheme::PRIMARY_TEXT)
                                    );
                                });
                        }
                    }
                    "ul" | "ol" => {
                        ui.indent("list", |ui| {
                            for (i, child) in children.iter().enumerate() {
                                if let DOMNode::Element { tag_name: child_tag, .. } = child {
                                    if child_tag == "li" {
                                        ui.horizontal(|ui| {
                                            let bullet = if tag_name == "ul" { "•" } else { &format!("{}.", i + 1) };
                                            ui.label(egui::RichText::new(bullet).color(NeonTheme::NEON_CYAN));
                                            self.render_dom_node(ui, child);
                                        });
                                    }
                                } else {
                                    self.render_dom_node(ui, child);
                                }
                            }
                        });
                    }
                    "li" => {
                        // List items are handled by their parent ul/ol
                        for child in children {
                            self.render_dom_node(ui, child);
                        }
                    }
                    "br" => {
                        ui.add_space(6.0);
                    }
                    "hr" => {
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);
                    }
                    "img" => {
                        let src = attributes.get("src").cloned().unwrap_or_default();
                        let alt = attributes.get("alt").cloned().unwrap_or_else(|| "Image".to_string());
                        
                        // For now, show a placeholder
                        ui.label(
                            egui::RichText::new(format!("🖼️ {} [{}]", alt, src))
                                .color(NeonTheme::MUTED_TEXT)
                        );
                    }
                    "table" => {
                        egui::Frame::none()
                            .stroke(egui::Stroke::new(1.0, NeonTheme::BORDER_COLOR))
                            .show(ui, |ui| {
                                for child in children {
                                    self.render_dom_node(ui, child);
                                }
                            });
                    }
                    "tr" => {
                        ui.horizontal(|ui| {
                            for child in children {
                                self.render_dom_node(ui, child);
                            }
                        });
                    }
                    "td" | "th" => {
                        let text = self.extract_text(node);
                        let rich_text = if tag_name == "th" {
                            egui::RichText::new(text).strong().color(NeonTheme::NEON_CYAN)
                        } else {
                            egui::RichText::new(text).color(NeonTheme::PRIMARY_TEXT)
                        };
                        ui.label(rich_text);
                        ui.separator();
                    }
                    "blockquote" => {
                        ui.indent("blockquote", |ui| {
                            egui::Frame::none()
                                .stroke(egui::Stroke::new(3.0, NeonTheme::NEON_PURPLE))
                                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                                .show(ui, |ui| {
                                    for child in children {
                                        self.render_dom_node(ui, child);
                                    }
                                });
                        });
                    }
                    "style" | "script" | "head" | "title" | "meta" | "link" => {
                        // Skip these elements - they don't produce visible content
                    }
                    _ => {
                        // Default rendering for unknown elements
                        for child in children {
                            self.render_dom_node(ui, child);
                        }
                    }
                }
            }
            DOMNode::Text(text) => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    ui.label(
                        egui::RichText::new(trimmed)
                            .color(NeonTheme::PRIMARY_TEXT)
                            .size(14.0)
                    );
                }
            }
        }
    }
    
    fn extract_text(&self, node: &DOMNode) -> String {
        match node {
            DOMNode::Text(text) => text.clone(),
            DOMNode::Element { children, .. } => {
                children.iter()
                    .map(|child| self.extract_text(child))
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        }
    }
}

fn extract_title(html: &str) -> Option<String> {
    use regex::Regex;
    let re = Regex::new("(?is)<title>(.*?)</title>").ok()?; // (?i) case-insensitive, (?s) dot matches newline
    if let Some(caps) = re.captures(html) {
        let t = caps.get(1)?.as_str().trim();
        if t.is_empty() { return None; }
        match html_entities::decode_html_entities(t) {
            Ok(decoded) => Some(decoded),
            Err(_) => Some(t.to_string()),
        }
    } else { None }
}

fn strip_html(html: &str) -> String {
    // Limit input size to prevent regex DoS
    let limited = if html.len() > 1024 * 1024 {
        &html[..1024 * 1024] // 1MB limit for text processing
    } else {
        html
    };
    
    // Very naive plain text extraction: remove script/style blocks then tags
    let mut s = regex::Regex::new("(?is)<script.*?</script>").unwrap().replace_all(limited, " ").to_string();
    s = regex::Regex::new("(?is)<style.*?</style>").unwrap().replace_all(&s, " ").to_string();
    s = regex::Regex::new("(?is)<!--.*?-->").unwrap().replace_all(&s, " ").to_string();
    s = regex::Regex::new("(?is)<[^>]+>").unwrap().replace_all(&s, " ").to_string();
    let decoded = match html_entities::decode_html_entities(&s) { Ok(v) => v, Err(_) => s };
    let collapsed = regex::Regex::new("[ \t\n\r]+").unwrap().replace_all(&decoded.as_str(), " ");
    let result = collapsed.trim().to_string();
    
    // Limit final output length
    if result.len() > 100 * 1024 {
        format!("{}\n\n[Text truncated at 100KB to prevent crashes]", &result[..100 * 1024])
    } else {
        result
    }
}