use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
    pub folder: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct BookmarkManager {
    bookmarks: Vec<Bookmark>,
    folders: Vec<String>,
    search_query: String,
    show_add_dialog: bool,
    new_bookmark_title: String,
    new_bookmark_url: String,
    selected_folder: Option<String>,
}

impl BookmarkManager {
    pub fn new() -> Self {
        let mut manager = Self {
            bookmarks: Vec::new(),
            folders: Vec::new(),
            search_query: String::new(),
            show_add_dialog: false,
            new_bookmark_title: String::new(),
            new_bookmark_url: String::new(),
            selected_folder: None,
        };
        
        // Add some default bookmarks
        manager.add_default_bookmarks();
        
        manager
    }
    
    fn add_default_bookmarks(&mut self) {
        self.bookmarks.push(Bookmark {
            title: "NeonDev GitHub".to_string(),
            url: "https://github.com/neondev".to_string(),
            folder: Some("Development".to_string()),
            created_at: chrono::Utc::now(),
        });
        
        self.bookmarks.push(Bookmark {
            title: "Rust Programming Language".to_string(),
            url: "https://www.rust-lang.org/".to_string(),
            folder: Some("Development".to_string()),
            created_at: chrono::Utc::now(),
        });
        
        self.folders.push("Development".to_string());
        self.folders.push("News".to_string());
        self.folders.push("Entertainment".to_string());
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<String> {
        let mut navigate_to = None;
        
        ui.heading("📖 Bookmarks");
        ui.separator();
        
        // Search bar
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.search_query);
            
            if ui.button("➕ Add").clicked() {
                self.show_add_dialog = true;
            }
        });
        
        ui.separator();
        
        // Filter bookmarks based on search
        let filtered_bookmarks: Vec<_> = self.bookmarks
            .iter()
            .filter(|bookmark| {
                if self.search_query.is_empty() {
                    true
                } else {
                    bookmark.title.to_lowercase().contains(&self.search_query.to_lowercase()) ||
                    bookmark.url.to_lowercase().contains(&self.search_query.to_lowercase())
                }
            })
            .collect();
        
        // Group bookmarks by folder
        let mut grouped_bookmarks: HashMap<Option<String>, Vec<&Bookmark>> = HashMap::new();
        for bookmark in filtered_bookmarks {
            grouped_bookmarks
                .entry(bookmark.folder.clone())
                .or_insert_with(Vec::new)
                .push(bookmark);
        }
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Show ungrouped bookmarks first
            if let Some(ungrouped) = grouped_bookmarks.get(&None) {
                for bookmark in ungrouped {
                    if self.show_bookmark(ui, bookmark) {
                        navigate_to = Some(bookmark.url.clone());
                    }
                }
                if !ungrouped.is_empty() {
                    ui.separator();
                }
            }
            
            // Show grouped bookmarks
            for (folder, bookmarks) in grouped_bookmarks {
                if let Some(folder_name) = folder {
                    ui.collapsing(&folder_name, |ui| {
                        for bookmark in bookmarks {
                            if self.show_bookmark(ui, bookmark) {
                                navigate_to = Some(bookmark.url.clone());
                            }
                        }
                    });
                }
            }
        });
        
        // Add bookmark dialog - handle separately to avoid borrowing conflicts
        if self.show_add_dialog {
            let mut add_clicked = false;
            let mut cancel_clicked = false;
            
            egui::Window::new("Add Bookmark")
                .open(&mut self.show_add_dialog)
                .show(ui.ctx(), |ui| {
                    ui.label("Title:");
                    ui.text_edit_singleline(&mut self.new_bookmark_title);
                    
                    ui.label("URL:");
                    ui.text_edit_singleline(&mut self.new_bookmark_url);
                    
                    ui.label("Folder (optional):");
                    egui::ComboBox::from_label("")
                        .selected_text(self.selected_folder.as_deref().unwrap_or("None"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_folder, None, "None");
                            for folder in &self.folders.clone() {
                                ui.selectable_value(&mut self.selected_folder, Some(folder.clone()), folder);
                            }
                        });
                    
                    ui.horizontal(|ui| {
                        if ui.button("Add").clicked() {
                            add_clicked = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancel_clicked = true;
                        }
                    });
                });
                
            if add_clicked {
                self.add_bookmark();
                self.show_add_dialog = false;
            }
            if cancel_clicked {
                self.show_add_dialog = false;
            }
        }
        
        navigate_to
    }
    
    fn show_bookmark(&self, ui: &mut egui::Ui, bookmark: &Bookmark) -> bool {
        ui.horizontal(|ui| {
            let response = ui.selectable_label(false, &bookmark.title);
            if ui.small_button("🗑").on_hover_text("Delete bookmark").clicked() {
                // Bookmark deletion implementation pending
            }
            response.clicked()
        }).inner
    }
    
    fn add_bookmark(&mut self) {
        if !self.new_bookmark_title.is_empty() && !self.new_bookmark_url.is_empty() {
            self.bookmarks.push(Bookmark {
                title: self.new_bookmark_title.clone(),
                url: self.new_bookmark_url.clone(),
                folder: self.selected_folder.clone(),
                created_at: chrono::Utc::now(),
            });
            
            // Clear form
            self.new_bookmark_title.clear();
            self.new_bookmark_url.clear();
            self.selected_folder = None;
            self.show_add_dialog = false;
        }
    }
    
    pub fn add_current_page(&mut self, title: String, url: String) {
        self.bookmarks.push(Bookmark {
            title,
            url,
            folder: None,
            created_at: chrono::Utc::now(),
        });
    }
}