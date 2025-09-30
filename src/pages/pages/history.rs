use eframe::egui::{Context, Ui, RichText, Layout, Align};
use crate::pages::{CustomPage, components};
use crate::ui::theme::NeonTheme;
use crate::ui::icons::NeonIcons;
use std::time::{SystemTime, Duration};

#[derive(Debug, Clone)]
pub struct HistoryItem {
    pub id: String,
    pub title: String,
    pub url: String,
    pub visit_time: SystemTime,
    pub visit_count: u32,
    pub favicon_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeFilter {
    Today,
    Yesterday,
    LastWeek,
    LastMonth,
    AllTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortBy {
    VisitTime,
    Title,
    Url,
    VisitCount,
}

pub struct HistoryPage {
    url: String,
    title: String,
    history_items: Vec<HistoryItem>,
    search_query: String,
    time_filter: TimeFilter,
    sort_by: SortBy,
    ascending: bool,
    selected_items: Vec<String>,
    show_details: bool,
}

impl HistoryPage {
    pub fn new() -> Self {
        // Create sample history items for demonstration
        let mut history_items = Vec::new();
        let now = SystemTime::now();
        
        // Recent items
        history_items.push(HistoryItem {
            id: "hist_1".to_string(),
            title: "GitHub - The complete developer platform".to_string(),
            url: "https://github.com".to_string(),
            visit_time: now,
            visit_count: 15,
            favicon_url: Some("https://github.com/favicon.ico".to_string()),
        });
        
        history_items.push(HistoryItem {
            id: "hist_2".to_string(),
            title: "Rust Programming Language".to_string(),
            url: "https://www.rust-lang.org/".to_string(),
            visit_time: now - Duration::from_secs(3600), // 1 hour ago
            visit_count: 8,
            favicon_url: Some("https://www.rust-lang.org/favicon.ico".to_string()),
        });
        
        history_items.push(HistoryItem {
            id: "hist_3".to_string(),
            title: "MDN Web Docs".to_string(),
            url: "https://developer.mozilla.org/".to_string(),
            visit_time: now - Duration::from_secs(7200), // 2 hours ago
            visit_count: 23,
            favicon_url: Some("https://developer.mozilla.org/favicon.ico".to_string()),
        });
        
        history_items.push(HistoryItem {
            id: "hist_4".to_string(),
            title: "Stack Overflow - Where Developers Learn".to_string(),
            url: "https://stackoverflow.com".to_string(),
            visit_time: now - Duration::from_secs(86400), // Yesterday
            visit_count: 42,
            favicon_url: Some("https://stackoverflow.com/favicon.ico".to_string()),
        });
        
        history_items.push(HistoryItem {
            id: "hist_5".to_string(),
            title: "Crates.io: Rust Package Registry".to_string(),
            url: "https://crates.io".to_string(),
            visit_time: now - Duration::from_secs(172800), // 2 days ago
            visit_count: 12,
            favicon_url: Some("https://crates.io/favicon.ico".to_string()),
        });
        
        history_items.push(HistoryItem {
            id: "hist_6".to_string(),
            title: "The egui Book".to_string(),
            url: "https://docs.rs/egui/latest/egui/".to_string(),
            visit_time: now - Duration::from_secs(259200), // 3 days ago
            visit_count: 6,
            favicon_url: None,
        });
        
        Self {
            url: "neon://history".to_string(),
            title: "History".to_string(),
            history_items,
            search_query: String::new(),
            time_filter: TimeFilter::AllTime,
            sort_by: SortBy::VisitTime,
            ascending: false,
            selected_items: Vec::new(),
            show_details: false,
        }
    }
    
    fn format_time_ago(time: SystemTime) -> String {
        if let Ok(duration) = SystemTime::now().duration_since(time) {
            let seconds = duration.as_secs();
            
            if seconds < 60 {
                "Just now".to_string()
            } else if seconds < 3600 {
                let minutes = seconds / 60;
                format!("{} minute{} ago", minutes, if minutes == 1 { "" } else { "s" })
            } else if seconds < 86400 {
                let hours = seconds / 3600;
                format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
            } else if seconds < 604800 {
                let days = seconds / 86400;
                format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
            } else if seconds < 2592000 {
                let weeks = seconds / 604800;
                format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" })
            } else {
                let months = seconds / 2592000;
                format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
            }
        } else {
            "Unknown".to_string()
        }
    }
    
    fn get_filtered_and_sorted_items(&self) -> Vec<&HistoryItem> {
        let now = SystemTime::now();
        
        let mut filtered: Vec<&HistoryItem> = self.history_items.iter()
            .filter(|item| {
                // Search filter
                let matches_search = self.search_query.is_empty() || 
                    item.title.to_lowercase().contains(&self.search_query.to_lowercase()) ||
                    item.url.to_lowercase().contains(&self.search_query.to_lowercase());
                
                // Time filter
                let matches_time = match self.time_filter {
                    TimeFilter::AllTime => true,
                    TimeFilter::Today => {
                        if let Ok(duration) = now.duration_since(item.visit_time) {
                            duration.as_secs() < 86400 // Last 24 hours
                        } else {
                            false
                        }
                    },
                    TimeFilter::Yesterday => {
                        if let Ok(duration) = now.duration_since(item.visit_time) {
                            let seconds = duration.as_secs();
                            seconds >= 86400 && seconds < 172800 // 24-48 hours ago
                        } else {
                            false
                        }
                    },
                    TimeFilter::LastWeek => {
                        if let Ok(duration) = now.duration_since(item.visit_time) {
                            duration.as_secs() < 604800 // Last 7 days
                        } else {
                            false
                        }
                    },
                    TimeFilter::LastMonth => {
                        if let Ok(duration) = now.duration_since(item.visit_time) {
                            duration.as_secs() < 2592000 // Last 30 days
                        } else {
                            false
                        }
                    },
                };
                
                matches_search && matches_time
            })
            .collect();
        
        // Sort items
        filtered.sort_by(|a, b| {
            let ordering = match self.sort_by {
                SortBy::VisitTime => a.visit_time.cmp(&b.visit_time),
                SortBy::Title => a.title.cmp(&b.title),
                SortBy::Url => a.url.cmp(&b.url),
                SortBy::VisitCount => a.visit_count.cmp(&b.visit_count),
            };
            
            if self.ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });
        
        filtered
    }
}

impl CustomPage for HistoryPage {
    fn get_url(&self) -> &str {
        &self.url
    }
    
    fn get_title(&self) -> &str {
        &self.title
    }
    
    fn render(&mut self, ui: &mut Ui, _ctx: &Context) {
        components::page_header(
            ui, 
            "Browsing History", 
            Some("View and manage your browsing history")
        );
        
        // Top toolbar with search, filters, and actions
        components::card_container(ui, |ui| {
            ui.vertical(|ui| {
                // First row: Search and time filter
                ui.horizontal(|ui| {
                    // Search box
                    ui.label(RichText::new(NeonIcons::SEARCH).color(NeonTheme::SECONDARY_TEXT));
                    ui.add(egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Search history...")
                        .desired_width(300.0));
                    
                    ui.add_space(20.0);
                    
                    // Time filter
                    ui.label("Time:");
                    egui::ComboBox::from_id_salt("time_filter")
                        .selected_text(format!("{:?}", self.time_filter))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.time_filter, TimeFilter::Today, "Today");
                            ui.selectable_value(&mut self.time_filter, TimeFilter::Yesterday, "Yesterday");
                            ui.selectable_value(&mut self.time_filter, TimeFilter::LastWeek, "Last Week");
                            ui.selectable_value(&mut self.time_filter, TimeFilter::LastMonth, "Last Month");
                            ui.selectable_value(&mut self.time_filter, TimeFilter::AllTime, "All Time");
                        });
                    
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // Clear history button
                        if ui.button(RichText::new("Clear History...")
                            .color(NeonTheme::error_color())).clicked() {
                            // TODO: Show confirmation dialog
                        }
                    });
                });
                
                ui.add_space(12.0);
                
                // Second row: Sort options and view toggle
                ui.horizontal(|ui| {
                    ui.label("Sort by:");
                    egui::ComboBox::from_id_salt("sort_by")
                        .selected_text(format!("{:?}", self.sort_by))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.sort_by, SortBy::VisitTime, "Visit Time");
                            ui.selectable_value(&mut self.sort_by, SortBy::Title, "Title");
                            ui.selectable_value(&mut self.sort_by, SortBy::Url, "URL");
                            ui.selectable_value(&mut self.sort_by, SortBy::VisitCount, "Visit Count");
                        });
                    
                    // Sort direction toggle
                    let sort_icon = if self.ascending { "↑" } else { "↓" };
                    if ui.button(sort_icon).clicked() {
                        self.ascending = !self.ascending;
                    }
                    
                    ui.add_space(20.0);
                    
                    ui.checkbox(&mut self.show_details, "Show details");
                    
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if !self.selected_items.is_empty() {
                            if ui.button(format!("Delete Selected ({})", self.selected_items.len())).clicked() {
                                // TODO: Delete selected items
                                self.selected_items.clear();
                            }
                            
                            ui.add_space(10.0);
                        }
                        
                        let filtered_count = self.get_filtered_and_sorted_items().len();
                        ui.label(RichText::new(format!("{} items", filtered_count))
                            .color(NeonTheme::SECONDARY_TEXT));
                    });
                });
            });
        });
        
        ui.add_space(16.0);
        
        // History list
        components::section_header(ui, NeonIcons::CLOCK, "Recent Activity");
        
        let filtered_items = self.get_filtered_and_sorted_items();
        
        if filtered_items.is_empty() {
            components::card_container(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(RichText::new(NeonIcons::CLOCK)
                        .size(48.0)
                        .color(NeonTheme::SECONDARY_TEXT));
                    ui.add_space(16.0);
                    
                    let message = if self.history_items.is_empty() {
                        "No browsing history yet"
                    } else if !self.search_query.is_empty() {
                        "No history items match your search"
                    } else {
                        "No history items in the selected time range"
                    };
                    
                    ui.label(RichText::new(message)
                        .size(18.0)
                        .color(NeonTheme::SECONDARY_TEXT));
                    ui.add_space(40.0);
                });
            });
        } else {
            // Clone items to avoid borrowing issues
            let items_to_render: Vec<HistoryItem> = filtered_items.iter().map(|item| (*item).clone()).collect();
            for item in items_to_render {
                self.render_history_item(ui, &item);
                ui.add_space(4.0);
            }
        }
    }
}

impl HistoryPage {
    fn render_history_item(&mut self, ui: &mut Ui, item: &HistoryItem) {
        let is_selected = self.selected_items.contains(&item.id);
        
        let frame_color = if is_selected {
            NeonTheme::NEON_CYAN.linear_multiply(0.2)
        } else {
            NeonTheme::CARD_BG
        };
        
        let response = egui::Frame::none()
            .fill(frame_color)
            .rounding(8.0)
            .stroke(egui::Stroke::new(1.0, if is_selected { 
                NeonTheme::NEON_CYAN 
            } else { 
                NeonTheme::BORDER_COLOR 
            }))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Checkbox for selection
                    let mut selected = is_selected;
                    if ui.checkbox(&mut selected, "").changed() {
                        if selected {
                            self.selected_items.push(item.id.clone());
                        } else {
                            self.selected_items.retain(|id| id != &item.id);
                        }
                    }
                    
                    ui.add_space(8.0);
                    
                    // Favicon placeholder
                    ui.label(RichText::new(NeonIcons::GLOBE)
                        .size(16.0)
                        .color(NeonTheme::SECONDARY_TEXT));
                    
                    ui.add_space(8.0);
                    
                    // Title and URL
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            // Title (clickable)
                            let title_response = ui.add(egui::Label::new(
                                RichText::new(&item.title)
                                    .size(14.0)
                                    .color(NeonTheme::PRIMARY_TEXT)
                            ).sense(egui::Sense::click()));
                            
                            if title_response.clicked() {
                                // TODO: Navigate to URL
                            }
                            
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                // Time ago
                                ui.label(RichText::new(Self::format_time_ago(item.visit_time))
                                    .size(12.0)
                                    .color(NeonTheme::SECONDARY_TEXT));
                                
                                if self.show_details {
                                    ui.add_space(10.0);
                                    
                                    // Visit count
                                    ui.label(RichText::new(format!("{} visits", item.visit_count))
                                        .size(12.0)
                                        .color(NeonTheme::SECONDARY_TEXT));
                                }
                            });
                        });
                        
                        // URL
                        ui.horizontal(|ui| {
                            let url_response = ui.add(egui::Label::new(
                                RichText::new(&item.url)
                                    .size(12.0)
                                    .color(NeonTheme::NEON_CYAN)
                            ).sense(egui::Sense::click()));
                            
                            if url_response.clicked() {
                                // TODO: Navigate to URL
                            }
                        });
                        
                        if self.show_details {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                
                                if ui.small_button("Remove from history").clicked() {
                                    // TODO: Remove this item
                                }
                                
                                if ui.small_button("Copy URL").clicked() {
                                    // TODO: Copy URL to clipboard
                                }
                            });
                        }
                    });
                });
            })
            .response;
        
        // Right-click context menu
        response.context_menu(|ui| {
            if ui.button("Open in new tab").clicked() {
                // TODO: Open in new tab
                ui.close_menu();
            }
            
            if ui.button("Copy URL").clicked() {
                // TODO: Copy URL to clipboard
                ui.close_menu();
            }
            
            ui.separator();
            
            if ui.button("Remove from history").clicked() {
                // TODO: Remove this item
                ui.close_menu();
            }
        });
    }
}