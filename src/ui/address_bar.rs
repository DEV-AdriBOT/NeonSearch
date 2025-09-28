use eframe::egui;
use crate::ui::theme::NeonTheme;

// Editing lifecycle states for the address bar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditState {
    Idle,          // Showing committed URL
    Editing,       // User is typing (staged text differs from committed)
    PendingCommit, // Enter pressed this frame
}

pub struct AddressBar {
    // The committed / loaded URL (what the tab actually represents)
    current_url: String,
    // The staged text the user is editing (may differ from current_url while editing)
    staged_input: String,
    // Cached value used by egui::TextEdit binding (we keep one binding string)
    edit_buffer: String,
    suggestions: Vec<String>,
    show_suggestions: bool,
    should_focus: bool,
    state: EditState,
}

impl AddressBar {
    pub fn new() -> Self {
        let initial = "about:home".to_string();
        Self {
            current_url: initial.clone(),
            staged_input: initial.clone(),
            edit_buffer: initial,
            suggestions: Vec::new(),
            show_suggestions: false,
            should_focus: false,
            state: EditState::Idle,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<String> {
        let mut navigate_to = None;
        
        // Modern address bar with enhanced styling
        egui::Frame::none()
            .fill(NeonTheme::ELEVATED_BG)
            .rounding(egui::Rounding::same(25.0)) // Pill shape
            .stroke(egui::Stroke::new(1.0, NeonTheme::BORDER_COLOR))
            .inner_margin(egui::Margin::symmetric(16.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    
                    // Enhanced security indicator
                    let (icon, tooltip, color) = if self.current_url.starts_with("https://") {
                        ("🔒", "Secure HTTPS connection", NeonTheme::SUCCESS_COLOR)
                    } else if self.current_url.starts_with("http://") {
                        ("⚠️", "Insecure HTTP connection", NeonTheme::WARNING_COLOR)
                    } else if self.current_url.starts_with("about:") {
                        ("🏠", "Browser page", NeonTheme::NEON_BLUE)
                    } else {
                        ("🌐", "Local or custom scheme", NeonTheme::MUTED_TEXT)
                    };
                    
                    ui.label(egui::RichText::new(icon).color(color).size(16.0))
                        .on_hover_text(tooltip);
            
                    // Modern URL input field
                    let available_width = ui.available_width() - 40.0; // Space for go button
                    
                    let text_edit_id = egui::Id::new("address_bar_input");
                    
                    // Handle focus request before creating the TextEdit
                    if self.should_focus {
                        // When focusing, load committed URL into buffer & select all
                        self.edit_buffer = self.current_url.clone();
                        self.staged_input = self.edit_buffer.clone();
                        self.state = EditState::Editing;
                        ui.memory_mut(|mem| mem.request_focus(text_edit_id));
                        self.should_focus = false;
                    }
                    
                    // Sync edit_buffer with staged_input if we just transitioned to Editing
                    if self.state == EditState::Idle {
                        // Ensure buffer shows committed value while idle
                        self.edit_buffer = self.current_url.clone();
                    }

                    let text_edit = egui::TextEdit::singleline(&mut self.edit_buffer)
                        .id(text_edit_id)
                        .hint_text("🔍 Search or enter URL...")
                        .margin(egui::vec2(8.0, 6.0))
                        .desired_width(available_width);
                        
                    let response = ui.add_sized([available_width, 36.0], text_edit);
            
            // Always try to focus if clicked
            if response.clicked() {
                // Enter editing mode when clicked (if not already)
                if self.state == EditState::Idle {
                    self.state = EditState::Editing;
                    self.edit_buffer = self.current_url.clone();
                }
                response.request_focus();
            }
            
            // Handle text changes and focus
            if response.changed() {
                if self.state != EditState::Editing {
                    self.state = EditState::Editing;
                }
                self.staged_input = self.edit_buffer.clone();
                self.update_suggestions();
            }
            
            // Submit on Enter key
            if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if self.state == EditState::Editing {
                    self.state = EditState::PendingCommit;
                }
            }

            // Commit navigation after losing focus with PendingCommit, or immediate
            if self.state == EditState::PendingCommit {
                // Apply staged_input, process, then leave editing mode
                self.edit_buffer = self.staged_input.clone();
                navigate_to = Some(self.process_input());
                self.show_suggestions = false;
                self.state = EditState::Idle;
            }
            
                    // Modern Go button
                    let go_btn = egui::Button::new(
                        egui::RichText::new("→")
                            .size(18.0)
                            .color(egui::Color32::WHITE)
                    )
                    .fill(NeonTheme::NEON_CYAN)
                    .rounding(egui::Rounding::same(18.0))
                    .stroke(egui::Stroke::NONE);
                    
                    if ui.add_sized([36.0, 36.0], go_btn)
                        .on_hover_text("Navigate to URL (Enter)")
                        .clicked() {
                        // Force commit current edit buffer if editing
                        self.staged_input = self.edit_buffer.clone();
                        self.state = EditState::Idle;
                        navigate_to = Some(self.process_input());
                    }
                });
            });
        
        // Show suggestions dropdown
        if self.show_suggestions && !self.suggestions.is_empty() {
            ui.group(|ui| {
                let mut selected_suggestion = None;
                for suggestion in &self.suggestions {
                    if ui.selectable_label(false, suggestion).clicked() {
                        selected_suggestion = Some(suggestion.clone());
                    }
                }
                if let Some(suggestion) = selected_suggestion {
                    self.edit_buffer = suggestion.clone();
                    self.staged_input = suggestion.clone();
                    self.state = EditState::PendingCommit; // Trigger commit below path
                    navigate_to = Some(self.process_input());
                    self.show_suggestions = false;
                    self.state = EditState::Idle;
                }
            });
        }
        
        navigate_to
    }
    
    fn process_input(&mut self) -> String {
        let input = self.staged_input.trim().to_string();
        
        // If it looks like a URL, use it directly
        if input.starts_with("http://") || 
           input.starts_with("https://") || 
           input.starts_with("about:") ||
           input.contains('.') {
            self.current_url = input.clone();
            input
        } else {
            // Otherwise, treat it as a search query
            let search_url = format!("https://duckduckgo.com/?q={}", 
                                   urlencoding::encode(&input));
            self.current_url = search_url.clone();
            search_url
        }
    }
    
    fn update_suggestions(&mut self) {
        // TODO: Implement proper suggestion system
        // For now, just show some basic suggestions
        self.suggestions.clear();
        
        if !self.staged_input.is_empty() {
            // Add some common suggestions
            if "about:home".starts_with(&self.staged_input.to_lowercase()) {
                self.suggestions.push("about:home".to_string());
            }
            if "about:blank".starts_with(&self.staged_input.to_lowercase()) {
                self.suggestions.push("about:blank".to_string());
            }
            
            // Add search suggestion
            if !self.staged_input.contains("://") {
                self.suggestions.push(format!("Search for '{}'", self.staged_input));
            }
        }
        
        self.show_suggestions = !self.suggestions.is_empty();
    }
    
    pub fn set_url(&mut self, url: String) {
        self.current_url = url.clone();
        // Only update staging/buffer if not actively editing
        if matches!(self.state, EditState::Idle) {
            self.staged_input = url.clone();
            self.edit_buffer = url;
        }
        self.show_suggestions = false;
    }
    
    pub fn focus(&mut self) {
        self.should_focus = true;
    }
}