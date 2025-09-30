use eframe::egui::{Context, Ui, RichText, Layout, Align};
use crate::pages::{CustomPage, components};
use crate::ui::theme::NeonTheme;
use crate::ui::icons::NeonIcons;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub url: String,
    pub folder_id: Option<String>,
    pub created_at: SystemTime,
    pub favicon_url: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BookmarkFolder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub created_at: SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    List,
    Grid,
    Tree,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortBy {
    Name,
    Url,
    DateAdded,
    Folder,
}

pub struct BookmarksPage {
    url: String,
    title: String,
    bookmarks: Vec<Bookmark>,
    folders: Vec<BookmarkFolder>,
    search_query: String,
    current_folder: Option<String>,
    view_mode: ViewMode,
    sort_by: SortBy,
    ascending: bool,
    selected_items: Vec<String>,
    show_add_dialog: bool,
    new_bookmark_title: String,
    new_bookmark_url: String,
    new_bookmark_folder: Option<String>,
}

impl BookmarksPage {
    pub fn new() -> Self {
        // Create sample folders
        let mut folders = Vec::new();
        
        folders.push(BookmarkFolder {
            id: "folder_1".to_string(),
            name: "Development".to_string(),
            parent_id: None,
            created_at: SystemTime::now(),
        });
        
        folders.push(BookmarkFolder {
            id: "folder_2".to_string(),
            name: "Learning".to_string(),
            parent_id: None,
            created_at: SystemTime::now(),
        });
        
        folders.push(BookmarkFolder {
            id: "folder_3".to_string(),
            name: "Rust".to_string(),
            parent_id: Some("folder_1".to_string()),
            created_at: SystemTime::now(),
        });
        
        // Create sample bookmarks
        let mut bookmarks = Vec::new();
        
        bookmarks.push(Bookmark {
            id: "bookmark_1".to_string(),
            title: "GitHub".to_string(),
            url: "https://github.com".to_string(),
            folder_id: Some("folder_1".to_string()),
            created_at: SystemTime::now(),
            favicon_url: Some("https://github.com/favicon.ico".to_string()),
            tags: vec!["code".to_string(), "git".to_string()],
            description: Some("The complete developer platform".to_string()),
        });
        
        bookmarks.push(Bookmark {
            id: "bookmark_2".to_string(),
            title: "Rust Programming Language".to_string(),
            url: "https://www.rust-lang.org/".to_string(),
            folder_id: Some("folder_3".to_string()),
            created_at: SystemTime::now(),
            favicon_url: Some("https://www.rust-lang.org/favicon.ico".to_string()),
            tags: vec!["rust".to_string(), "programming".to_string(), "systems".to_string()],
            description: Some("A language empowering everyone to build reliable and efficient software".to_string()),
        });
        
        bookmarks.push(Bookmark {
            id: "bookmark_3".to_string(),
            title: "MDN Web Docs".to_string(),
            url: "https://developer.mozilla.org/".to_string(),
            folder_id: Some("folder_2".to_string()),
            created_at: SystemTime::now(),
            favicon_url: Some("https://developer.mozilla.org/favicon.ico".to_string()),
            tags: vec!["web".to_string(), "docs".to_string(), "reference".to_string()],
            description: Some("Web development documentation and tutorials".to_string()),
        });
        
        bookmarks.push(Bookmark {
            id: "bookmark_4".to_string(),
            title: "Crates.io".to_string(),
            url: "https://crates.io".to_string(),
            folder_id: Some("folder_3".to_string()),
            created_at: SystemTime::now(),
            favicon_url: Some("https://crates.io/favicon.ico".to_string()),
            tags: vec!["rust".to_string(), "packages".to_string(), "crates".to_string()],
            description: Some("The Rust community's crate registry".to_string()),
        });
        
        bookmarks.push(Bookmark {
            id: "bookmark_5".to_string(),
            title: "Stack Overflow".to_string(),
            url: "https://stackoverflow.com".to_string(),
            folder_id: None, // Root level
            created_at: SystemTime::now(),
            favicon_url: Some("https://stackoverflow.com/favicon.ico".to_string()),
            tags: vec!["programming".to_string(), "questions".to_string(), "help".to_string()],
            description: Some("Where developers learn, share, & build careers".to_string()),
        });
        
        Self {
            url: "neon://bookmarks".to_string(),
            title: "Bookmarks".to_string(),
            bookmarks,
            folders,
            search_query: String::new(),
            current_folder: None,
            view_mode: ViewMode::List,
            sort_by: SortBy::Name,
            ascending: true,
            selected_items: Vec::new(),
            show_add_dialog: false,
            new_bookmark_title: String::new(),
            new_bookmark_url: String::new(),
            new_bookmark_folder: None,
        }
    }
    
    fn get_filtered_bookmarks(&self) -> Vec<&Bookmark> {
        self.bookmarks.iter()
            .filter(|bookmark| {
                // Folder filter
                let in_current_folder = match &self.current_folder {
                    None => bookmark.folder_id.is_none(), // Root level
                    Some(folder_id) => bookmark.folder_id.as_ref() == Some(folder_id),
                };
                
                // Search filter
                let matches_search = self.search_query.is_empty() || 
                    bookmark.title.to_lowercase().contains(&self.search_query.to_lowercase()) ||
                    bookmark.url.to_lowercase().contains(&self.search_query.to_lowercase()) ||
                    bookmark.tags.iter().any(|tag| tag.to_lowercase().contains(&self.search_query.to_lowercase()));
                
                in_current_folder && matches_search
            })
            .collect()
    }
    
    fn get_current_folder_name(&self) -> String {
        match &self.current_folder {
            None => "All Bookmarks".to_string(),
            Some(folder_id) => {
                self.folders.iter()
                    .find(|folder| &folder.id == folder_id)
                    .map(|folder| folder.name.clone())
                    .unwrap_or_else(|| "Unknown Folder".to_string())
            }
        }
    }
    
    fn get_subfolders(&self, parent_id: Option<&String>) -> Vec<&BookmarkFolder> {
        self.folders.iter()
            .filter(|folder| folder.parent_id.as_ref() == parent_id)
            .collect()
    }
}

impl CustomPage for BookmarksPage {
    fn get_url(&self) -> &str {
        &self.url
    }
    
    fn get_title(&self) -> &str {
        &self.title
    }
    
    fn render(&mut self, ui: &mut Ui, _ctx: &Context) {
        components::page_header(
            ui, 
            "Bookmarks", 
            Some("Organize and manage your saved bookmarks")
        );
        
        // Top toolbar
        components::card_container(ui, |ui| {
            ui.vertical(|ui| {
                // First row: Navigation and actions
                ui.horizontal(|ui| {
                    // Back button (if in a folder)
                    if self.current_folder.is_some() {
                        if ui.button(RichText::new(format!("{} Back", NeonIcons::ARROW_LEFT))
                            .color(NeonTheme::NEON_CYAN)).clicked() {
                            self.current_folder = None;
                        }
                        ui.add_space(8.0);
                    }
                    
                    // Current location
                    ui.label(RichText::new(format!("{} {}", NeonIcons::FOLDER, self.get_current_folder_name()))
                        .size(16.0)
                        .strong()
                        .color(NeonTheme::PRIMARY_TEXT));
                    
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // Add bookmark button
                        if ui.button(RichText::new(format!("{} Add Bookmark", NeonIcons::PLUS))
                            .color(NeonTheme::NEON_CYAN)).clicked() {
                            self.show_add_dialog = true;
                            self.new_bookmark_title.clear();
                            self.new_bookmark_url.clear();
                            self.new_bookmark_folder = self.current_folder.clone();
                        }
                        
                        ui.add_space(8.0);
                        
                        // Import/Export buttons
                        if ui.button(RichText::new(format!("{} Import", NeonIcons::UPLOAD))
                            .color(NeonTheme::SECONDARY_TEXT)).clicked() {
                            // TODO: Import bookmarks
                        }
                        
                        if ui.button(RichText::new(format!("{} Export", NeonIcons::DOWNLOAD))
                            .color(NeonTheme::SECONDARY_TEXT)).clicked() {
                            // TODO: Export bookmarks
                        }
                    });
                });
                
                ui.add_space(12.0);
                
                // Second row: Search and view options
                ui.horizontal(|ui| {
                    // Search box
                    ui.label(RichText::new(NeonIcons::SEARCH).color(NeonTheme::SECONDARY_TEXT));
                    ui.add(egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Search bookmarks, tags, or URLs...")
                        .desired_width(300.0));
                    
                    ui.add_space(20.0);
                    
                    // View mode selector
                    ui.label("View:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.view_mode, ViewMode::List, "List");
                        ui.selectable_value(&mut self.view_mode, ViewMode::Grid, "Grid");
                        ui.selectable_value(&mut self.view_mode, ViewMode::Tree, "Tree");
                    });
                    
                    ui.add_space(20.0);
                    
                    // Sort options
                    ui.label("Sort by:");
                    egui::ComboBox::from_id_salt("sort_bookmarks")
                        .selected_text(format!("{:?}", self.sort_by))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.sort_by, SortBy::Name, "Name");
                            ui.selectable_value(&mut self.sort_by, SortBy::Url, "URL");
                            ui.selectable_value(&mut self.sort_by, SortBy::DateAdded, "Date Added");
                            ui.selectable_value(&mut self.sort_by, SortBy::Folder, "Folder");
                        });
                    
                    // Sort direction
                    let sort_icon = if self.ascending { "↑" } else { "↓" };
                    if ui.button(sort_icon).clicked() {
                        self.ascending = !self.ascending;
                    }
                });
            });
        });
        
        ui.add_space(16.0);
        
        // Add bookmark dialog
        if self.show_add_dialog {
            self.render_add_bookmark_dialog(ui);
        }
        
        // Main content area
        ui.horizontal_top(|ui| {
            // Left sidebar: Folders
            ui.vertical(|ui| {
                ui.set_min_width(200.0);
                
                components::card_container(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Folders")
                            .size(16.0)
                            .strong()
                            .color(NeonTheme::PRIMARY_TEXT));
                        
                        ui.add_space(8.0);
                        
                        // Root folder
                        let is_root_selected = self.current_folder.is_none();
                        let root_color = if is_root_selected {
                            NeonTheme::NEON_CYAN
                        } else {
                            NeonTheme::SECONDARY_TEXT
                        };
                        
                        let root_response = ui.horizontal(|ui| {
                            ui.label(RichText::new(NeonIcons::FOLDER).color(root_color));
                            ui.label(RichText::new("All Bookmarks").color(root_color));
                        }).response;
                        
                        if root_response.clicked() {
                            self.current_folder = None;
                        }
                        
                        ui.add_space(4.0);
                        
                        // Folder list
                        let subfolders: Vec<BookmarkFolder> = self.get_subfolders(None).iter().map(|f| (*f).clone()).collect();
                        for folder in subfolders {
                            self.render_folder_item(ui, &folder);
                        }
                        
                        ui.add_space(12.0);
                        
                        // Add folder button
                        if ui.button(RichText::new(format!("{} New Folder", NeonIcons::PLUS))
                            .color(NeonTheme::SECONDARY_TEXT)).clicked() {
                            // TODO: Show add folder dialog
                        }
                    });
                });
            });
            
            ui.add_space(20.0);
            
            // Right content: Bookmarks
            ui.vertical(|ui| {
                let filtered_bookmarks = self.get_filtered_bookmarks();
                
                if filtered_bookmarks.is_empty() {
                    components::card_container(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            ui.label(RichText::new(NeonIcons::BOOKMARK)
                                .size(48.0)
                                .color(NeonTheme::SECONDARY_TEXT));
                            ui.add_space(16.0);
                            
                            let message = if self.bookmarks.is_empty() {
                                "No bookmarks yet"
                            } else if !self.search_query.is_empty() {
                                "No bookmarks match your search"
                            } else {
                                "No bookmarks in this folder"
                            };
                            
                            ui.label(RichText::new(message)
                                .size(18.0)
                                .color(NeonTheme::SECONDARY_TEXT));
                            ui.add_space(40.0);
                        });
                    });
                } else {
                    components::section_header(ui, NeonIcons::BOOKMARK, 
                        &format!("Bookmarks ({})", filtered_bookmarks.len()));
                    
                    match self.view_mode {
                        ViewMode::List => {
                            let bookmarks_to_render: Vec<Bookmark> = filtered_bookmarks.iter().map(|b| (*b).clone()).collect();
                            for bookmark in bookmarks_to_render {
                                self.render_bookmark_list_item(ui, &bookmark);
                                ui.add_space(4.0);
                            }
                        },
                        ViewMode::Grid => {
                            let bookmarks_clone: Vec<Bookmark> = filtered_bookmarks.iter().map(|b| (*b).clone()).collect();
                            let bookmark_refs: Vec<&Bookmark> = bookmarks_clone.iter().collect();
                            self.render_bookmarks_grid(ui, &bookmark_refs);
                        },
                        ViewMode::Tree => {
                            let bookmarks_clone: Vec<Bookmark> = filtered_bookmarks.iter().map(|b| (*b).clone()).collect();
                            let bookmark_refs: Vec<&Bookmark> = bookmarks_clone.iter().collect();
                            self.render_bookmarks_tree(ui, &bookmark_refs);
                        },
                    }
                }
            });
        });
    }
}

impl BookmarksPage {
    fn render_folder_item(&mut self, ui: &mut Ui, folder: &BookmarkFolder) {
        let is_selected = self.current_folder.as_ref() == Some(&folder.id);
        let color = if is_selected {
            NeonTheme::NEON_CYAN
        } else {
            NeonTheme::SECONDARY_TEXT
        };
        
        let response = ui.horizontal(|ui| {
            ui.add_space(16.0); // Indent
            ui.label(RichText::new(NeonIcons::FOLDER).color(color));
            ui.label(RichText::new(&folder.name).color(color));
        }).response;
        
        if response.clicked() {
            self.current_folder = Some(folder.id.clone());
        }
        
        // Show subfolders if this folder is selected
        if is_selected {
            let subfolders: Vec<BookmarkFolder> = self.get_subfolders(Some(&folder.id)).iter().map(|f| (*f).clone()).collect();
            for subfolder in subfolders {
                ui.horizontal(|ui| {
                    ui.add_space(32.0); // Double indent
                    ui.label(RichText::new(NeonIcons::FOLDER).color(NeonTheme::SECONDARY_TEXT));
                    ui.label(RichText::new(&subfolder.name).color(NeonTheme::SECONDARY_TEXT));
                });
            }
        }
    }
    
    fn render_bookmark_list_item(&mut self, ui: &mut Ui, bookmark: &Bookmark) {
        let is_selected = self.selected_items.contains(&bookmark.id);
        
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
                    // Favicon
                    ui.label(RichText::new(NeonIcons::GLOBE)
                        .size(16.0)
                        .color(NeonTheme::SECONDARY_TEXT));
                    
                    ui.add_space(8.0);
                    
                    // Bookmark info
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            // Title (clickable)
                            let title_response = ui.add(egui::Label::new(
                                RichText::new(&bookmark.title)
                                    .size(14.0)
                                    .strong()
                                    .color(NeonTheme::PRIMARY_TEXT)
                            ).sense(egui::Sense::click()));
                            
                            if title_response.clicked() {
                                // TODO: Navigate to bookmark URL
                                println!("Opening: {}", bookmark.url);
                            }
                            
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                // Action buttons
                                if ui.small_button("Edit").clicked() {
                                    // TODO: Edit bookmark
                                }
                                
                                if ui.small_button("Delete").clicked() {
                                    // TODO: Delete bookmark
                                }
                            });
                        });
                        
                        // URL
                        ui.label(RichText::new(&bookmark.url)
                            .size(12.0)
                            .color(NeonTheme::NEON_CYAN));
                        
                        // Description (if available)
                        if let Some(description) = &bookmark.description {
                            ui.label(RichText::new(description)
                                .size(11.0)
                                .color(NeonTheme::SECONDARY_TEXT));
                        }
                        
                        // Tags
                        if !bookmark.tags.is_empty() {
                            ui.horizontal(|ui| {
                                for tag in &bookmark.tags {
                                    ui.small(RichText::new(format!("#{}", tag))
                                        .color(NeonTheme::NEON_PURPLE));
                                }
                            });
                        }
                    });
                });
            })
            .response;
        
        // Context menu
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
            
            if ui.button("Edit bookmark").clicked() {
                // TODO: Edit bookmark
                ui.close_menu();
            }
            
            if ui.button("Delete bookmark").clicked() {
                // TODO: Delete bookmark
                ui.close_menu();
            }
        });
    }
    
    fn render_bookmarks_grid(&mut self, ui: &mut Ui, bookmarks: &[&Bookmark]) {
        ui.label("Grid view coming soon...");
        // TODO: Implement grid layout
    }
    
    fn render_bookmarks_tree(&mut self, ui: &mut Ui, bookmarks: &[&Bookmark]) {
        ui.label("Tree view coming soon...");
        // TODO: Implement tree layout with folders
    }
    
    fn render_add_bookmark_dialog(&mut self, ui: &mut Ui) {
        // Modal dialog for adding bookmarks
        egui::Window::new("Add Bookmark")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.label("Title:");
                    ui.text_edit_singleline(&mut self.new_bookmark_title);
                    
                    ui.add_space(8.0);
                    
                    ui.label("URL:");
                    ui.text_edit_singleline(&mut self.new_bookmark_url);
                    
                    ui.add_space(8.0);
                    
                    ui.label("Folder:");
                    egui::ComboBox::from_label("")
                        .selected_text(self.new_bookmark_folder.as_ref()
                            .and_then(|id| self.folders.iter().find(|f| &f.id == id))
                            .map(|f| f.name.clone())
                            .unwrap_or_else(|| "Root".to_string()))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.new_bookmark_folder, None, "Root");
                            for folder in &self.folders {
                                ui.selectable_value(&mut self.new_bookmark_folder, 
                                    Some(folder.id.clone()), &folder.name);
                            }
                        });
                    
                    ui.add_space(16.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Add Bookmark").clicked() {
                            // Create new bookmark
                            let new_bookmark = Bookmark {
                                id: format!("bookmark_{}", self.bookmarks.len() + 1),
                                title: self.new_bookmark_title.clone(),
                                url: self.new_bookmark_url.clone(),
                                folder_id: self.new_bookmark_folder.clone(),
                                created_at: SystemTime::now(),
                                favicon_url: None,
                                tags: Vec::new(),
                                description: None,
                            };
                            
                            self.bookmarks.push(new_bookmark);
                            self.show_add_dialog = false;
                        }
                        
                        if ui.button("Cancel").clicked() {
                            self.show_add_dialog = false;
                        }
                    });
                });
            });
    }
}