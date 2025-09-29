// Developer Console UI for JavaScript debugging
use eframe::egui;
use crate::js::JSEngine;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum ConsoleMessage {
    Log(String),
    Error(String),
    Warn(String),
    Info(String),
    Input(String),  // User input
    Output(String), // Command output
}

impl ConsoleMessage {
    pub fn color(&self) -> egui::Color32 {
        match self {
            ConsoleMessage::Log(_) => egui::Color32::from_rgb(220, 220, 220),
            ConsoleMessage::Error(_) => egui::Color32::from_rgb(255, 100, 100),
            ConsoleMessage::Warn(_) => egui::Color32::from_rgb(255, 200, 100),
            ConsoleMessage::Info(_) => egui::Color32::from_rgb(100, 200, 255),
            ConsoleMessage::Input(_) => egui::Color32::from_rgb(100, 255, 100),
            ConsoleMessage::Output(_) => egui::Color32::from_rgb(200, 200, 255),
        }
    }
    
    pub fn prefix(&self) -> &'static str {
        match self {
            ConsoleMessage::Log(_) => "LOG",
            ConsoleMessage::Error(_) => "ERR",
            ConsoleMessage::Warn(_) => "WARN",
            ConsoleMessage::Info(_) => "INFO",
            ConsoleMessage::Input(_) => ">",
            ConsoleMessage::Output(_) => "←",
        }
    }
    
    pub fn content(&self) -> &str {
        match self {
            ConsoleMessage::Log(s) |
            ConsoleMessage::Error(s) |
            ConsoleMessage::Warn(s) |
            ConsoleMessage::Info(s) |
            ConsoleMessage::Input(s) |
            ConsoleMessage::Output(s) => s,
        }
    }
}

pub struct DevConsole {
    messages: VecDeque<ConsoleMessage>,
    input_buffer: String,
    is_visible: bool,
    max_messages: usize,
    auto_scroll: bool,
    command_history: Vec<String>,
    history_index: Option<usize>,
}

impl Default for DevConsole {
    fn default() -> Self {
        Self::new()
    }
}

impl DevConsole {
    pub fn new() -> Self {
        let mut console = Self {
            messages: VecDeque::new(),
            input_buffer: String::new(),
            is_visible: false,
            max_messages: 1000,
            auto_scroll: true,
            command_history: Vec::new(),
            history_index: None,
        };
        
        // Add welcome message
        console.add_message(ConsoleMessage::Info(
            "🚀 NeonSearch Developer Console - JavaScript Engine v0.2.0".to_string()
        ));
        console.add_message(ConsoleMessage::Log(
            "Type JavaScript commands to execute them. Press F12 to toggle console.".to_string()
        ));
        
        console
    }
    
    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
    }
    
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
    
    pub fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
    }
    
    pub fn add_message(&mut self, message: ConsoleMessage) {
        self.messages.push_back(message);
        
        // Limit message history
        while self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }
    }
    
    pub fn log(&mut self, message: String) {
        self.add_message(ConsoleMessage::Log(message));
    }
    
    pub fn error(&mut self, message: String) {
        self.add_message(ConsoleMessage::Error(message));
    }
    
    pub fn warn(&mut self, message: String) {
        self.add_message(ConsoleMessage::Warn(message));
    }
    
    pub fn info(&mut self, message: String) {
        self.add_message(ConsoleMessage::Info(message));
    }
    
    pub fn clear(&mut self) {
        self.messages.clear();
        self.add_message(ConsoleMessage::Info("Console cleared".to_string()));
    }
    
    pub fn execute_command(&mut self, command: String, js_engine: &mut Option<JSEngine>) -> String {
        // Add command to history
        if !command.trim().is_empty() && !self.command_history.contains(&command) {
            self.command_history.push(command.clone());
            if self.command_history.len() > 100 {
                self.command_history.remove(0);
            }
        }
        self.history_index = None;
        
        // Log the input command
        self.add_message(ConsoleMessage::Input(command.clone()));
        
        // Handle special console commands
        let cmd = command.trim();
        match cmd {
            "clear" => {
                self.clear();
                return "Console cleared".to_string();
            }
            "help" => {
                let help_text = r#"Available commands:
• clear - Clear the console
• help - Show this help message
• JavaScript expressions - Execute JavaScript code
• var x = 5 - Declare variables
• console.log("text") - Log messages"#;
                self.add_message(ConsoleMessage::Info(help_text.to_string()));
                return "Help displayed".to_string();
            }
            _ => {}
        }
        
        // Execute JavaScript command
        if let Some(ref mut engine) = js_engine {
            match engine.execute(&command) {
                Ok(result) => {
                    if !result.trim().is_empty() && result != "undefined" {
                        self.add_message(ConsoleMessage::Output(result.clone()));
                    }
                    
                    // Also show any console output from the engine
                    let console_output = engine.get_console_output();
                    for line in console_output {
                        if line.starts_with("[LOG]") {
                            self.add_message(ConsoleMessage::Log(line[5..].trim().to_string()));
                        } else if line.starts_with("[ERROR]") {
                            self.add_message(ConsoleMessage::Error(line[7..].trim().to_string()));
                        } else if line.starts_with("[WARN]") {
                            self.add_message(ConsoleMessage::Warn(line[6..].trim().to_string()));
                        } else if line.starts_with("[INFO]") {
                            self.add_message(ConsoleMessage::Info(line[6..].trim().to_string()));
                        } else {
                            self.add_message(ConsoleMessage::Log(line));
                        }
                    }
                    
                    result
                }
                Err(e) => {
                    let error_msg = format!("JavaScript Error: {}", e);
                    self.add_message(ConsoleMessage::Error(error_msg.clone()));
                    error_msg
                }
            }
        } else {
            let error_msg = "No JavaScript engine available".to_string();
            self.add_message(ConsoleMessage::Error(error_msg.clone()));
            error_msg
        }
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, js_engine: &mut Option<JSEngine>) {
        if !self.is_visible {
            return;
        }
        
        // Console window
        egui::Window::new("🧑‍💻 Developer Console")
            .default_width(800.0)
            .default_height(400.0)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Clear").clicked() {
                        self.clear();
                    }
                    
                    ui.separator();
                    
                    ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                    
                    ui.separator();
                    
                    ui.label(format!("Messages: {}", self.messages.len()));
                });
                
                ui.separator();
                
                // Messages area
                let messages_height = ui.available_height() - 60.0; // Leave space for input
                egui::ScrollArea::vertical()
                    .max_height(messages_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for message in &self.messages {
                            ui.horizontal(|ui| {
                                // Prefix with color
                                ui.label(
                                    egui::RichText::new(format!("[{}]", message.prefix()))
                                        .color(message.color())
                                        .monospace()
                                        .size(12.0)
                                );
                                
                                // Message content
                                ui.label(
                                    egui::RichText::new(message.content())
                                        .color(message.color())
                                        .size(13.0)
                                );
                            });
                        }
                        
                        // Auto-scroll to bottom
                        if self.auto_scroll {
                            ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                        }
                    });
                
                ui.separator();
                
                // Input area
                ui.horizontal(|ui| {
                    ui.label(">");
                    
                    let input_response = ui.add(
                        egui::TextEdit::singleline(&mut self.input_buffer)
                            .desired_width(ui.available_width() - 80.0)
                            .hint_text("Enter JavaScript command...")
                    );
                    
                    // Handle input
                    if input_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let command = self.input_buffer.clone();
                        self.input_buffer.clear();
                        self.execute_command(command, js_engine);
                        
                        // Request focus back to input
                        input_response.request_focus();
                    }
                    
                    // Handle history navigation
                    if input_response.has_focus() {
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                            self.navigate_history_up();
                        } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                            self.navigate_history_down();
                        }
                    }
                    
                    // Execute button
                    if ui.button("Execute").clicked() && !self.input_buffer.trim().is_empty() {
                        let command = self.input_buffer.clone();
                        self.input_buffer.clear();
                        self.execute_command(command, js_engine);
                    }
                });
            });
    }
    
    fn navigate_history_up(&mut self) {
        if self.command_history.is_empty() {
            return;
        }
        
        let new_index = match self.history_index {
            None => Some(self.command_history.len() - 1),
            Some(i) if i > 0 => Some(i - 1),
            Some(_) => return, // Already at the beginning
        };
        
        if let Some(index) = new_index {
            self.history_index = Some(index);
            self.input_buffer = self.command_history[index].clone();
        }
    }
    
    fn navigate_history_down(&mut self) {
        if let Some(index) = self.history_index {
            if index + 1 < self.command_history.len() {
                self.history_index = Some(index + 1);
                self.input_buffer = self.command_history[index + 1].clone();
            } else {
                self.history_index = None;
                self.input_buffer.clear();
            }
        }
    }
    
    pub fn sync_with_js_engine(&mut self, js_engine: &JSEngine) {
        // Sync console output from JavaScript engine
        let console_output = js_engine.get_console_output();
        for line in console_output {
            if line.starts_with("[LOG]") {
                self.add_message(ConsoleMessage::Log(line[5..].trim().to_string()));
            } else if line.starts_with("[ERROR]") {
                self.add_message(ConsoleMessage::Error(line[7..].trim().to_string()));
            } else if line.starts_with("[WARN]") {
                self.add_message(ConsoleMessage::Warn(line[6..].trim().to_string()));
            } else if line.starts_with("[INFO]") {
                self.add_message(ConsoleMessage::Info(line[6..].trim().to_string()));
            } else {
                self.add_message(ConsoleMessage::Log(line));
            }
        }
    }
}