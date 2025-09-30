use eframe::egui::{Context, Ui, RichText, Layout, Align};
use crate::pages::{CustomPage, components};
use crate::ui::theme::NeonTheme;
use crate::ui::icons::NeonIcons;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct DownloadItem {
    pub id: String,
    pub filename: String,
    pub url: String,
    pub file_size: u64,
    pub downloaded_size: u64,
    pub status: DownloadStatus,
    pub start_time: SystemTime,
    pub download_path: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DownloadStatus {
    InProgress,
    Completed,
    Paused,
    Failed(String),
    Cancelled,
}

pub struct DownloadsPage {
    url: String,
    title: String,
    downloads: Vec<DownloadItem>,
    search_query: String,
    show_only_active: bool,
    selected_download: Option<String>,
}

impl DownloadsPage {
    pub fn new() -> Self {
        // Create some sample download items for demonstration
        let mut downloads = Vec::new();
        
        // Sample completed download
        downloads.push(DownloadItem {
            id: "download_1".to_string(),
            filename: "neon_browser_setup.exe".to_string(),
            url: "https://example.com/downloads/neon_browser_setup.exe".to_string(),
            file_size: 45_000_000, // 45MB
            downloaded_size: 45_000_000,
            status: DownloadStatus::Completed,
            start_time: SystemTime::now(),
            download_path: "/Users/Downloads/neon_browser_setup.exe".to_string(),
            mime_type: "application/x-msdownload".to_string(),
        });
        
        // Sample in-progress download
        downloads.push(DownloadItem {
            id: "download_2".to_string(),
            filename: "large_dataset.zip".to_string(),
            url: "https://example.com/data/large_dataset.zip".to_string(),
            file_size: 150_000_000, // 150MB
            downloaded_size: 75_000_000, // 50% complete
            status: DownloadStatus::InProgress,
            start_time: SystemTime::now(),
            download_path: "/Users/Downloads/large_dataset.zip".to_string(),
            mime_type: "application/zip".to_string(),
        });
        
        // Sample failed download
        downloads.push(DownloadItem {
            id: "download_3".to_string(),
            filename: "document.pdf".to_string(),
            url: "https://example.com/docs/document.pdf".to_string(),
            file_size: 2_500_000, // 2.5MB
            downloaded_size: 1_200_000, // Partially downloaded
            status: DownloadStatus::Failed("Network connection lost".to_string()),
            start_time: SystemTime::now(),
            download_path: "/Users/Downloads/document.pdf".to_string(),
            mime_type: "application/pdf".to_string(),
        });
        
        Self {
            url: "neon://downloads".to_string(),
            title: "Downloads".to_string(),
            downloads,
            search_query: String::new(),
            show_only_active: false,
            selected_download: None,
        }
    }
    
    fn format_file_size(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
    
    fn get_status_icon_and_color(status: &DownloadStatus) -> (&'static str, egui::Color32) {
        match status {
            DownloadStatus::InProgress => (NeonIcons::DOWNLOAD, NeonTheme::NEON_CYAN),
            DownloadStatus::Completed => (NeonIcons::CHECK_CIRCLE, NeonTheme::success_color()),
            DownloadStatus::Paused => (NeonIcons::PAUSE, NeonTheme::warning_color()),
            DownloadStatus::Failed(_) => (NeonIcons::WARNING, NeonTheme::error_color()),
            DownloadStatus::Cancelled => (NeonIcons::CROSS, NeonTheme::SECONDARY_TEXT),
        }
    }
}

impl CustomPage for DownloadsPage {
    fn get_url(&self) -> &str {
        &self.url
    }
    
    fn get_title(&self) -> &str {
        &self.title
    }
    
    fn render(&mut self, ui: &mut Ui, _ctx: &Context) {
        components::page_header(
            ui, 
            "Downloads", 
            Some("View and manage your downloads")
        );
        
        // Top toolbar with search and filters
        components::card_container(ui, |ui| {
            ui.horizontal(|ui| {
                // Search box
                ui.label(RichText::new(NeonIcons::SEARCH).color(NeonTheme::SECONDARY_TEXT));
                ui.add(egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("Search downloads...")
                    .desired_width(250.0));
                
                ui.add_space(20.0);
                
                // Filter checkbox
                ui.checkbox(&mut self.show_only_active, "Show only active downloads");
                
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    // Clear all button
                    if ui.button(RichText::new("Clear All Completed")
                        .color(NeonTheme::SECONDARY_TEXT)).clicked() {
                        self.downloads.retain(|d| d.status != DownloadStatus::Completed);
                    }
                    
                    ui.add_space(10.0);
                    
                    // Refresh button
                    if ui.button(RichText::new(format!("{} Refresh", NeonIcons::REFRESH))
                        .color(NeonTheme::NEON_CYAN)).clicked() {
                        // TODO: Refresh download list
                    }
                });
            });
        });
        
        ui.add_space(16.0);
        
        // Downloads list
        components::section_header(ui, NeonIcons::DOWNLOAD, "Download History");
        
        // Filter downloads based on search and active filter
        let filtered_downloads: Vec<&DownloadItem> = self.downloads.iter()
            .filter(|download| {
                // Search filter
                let matches_search = self.search_query.is_empty() || 
                    download.filename.to_lowercase().contains(&self.search_query.to_lowercase()) ||
                    download.url.to_lowercase().contains(&self.search_query.to_lowercase());
                
                // Active filter
                let matches_active_filter = if self.show_only_active {
                    matches!(download.status, DownloadStatus::InProgress | DownloadStatus::Paused)
                } else {
                    true
                };
                
                matches_search && matches_active_filter
            })
            .collect();
        
        if filtered_downloads.is_empty() {
            components::card_container(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(RichText::new(NeonIcons::DOWNLOAD)
                        .size(48.0)
                        .color(NeonTheme::SECONDARY_TEXT));
                    ui.add_space(16.0);
                    
                    let message = if self.downloads.is_empty() {
                        "No downloads yet"
                    } else if !self.search_query.is_empty() {
                        "No downloads match your search"
                    } else {
                        "No active downloads"
                    };
                    
                    ui.label(RichText::new(message)
                        .size(18.0)
                        .color(NeonTheme::SECONDARY_TEXT));
                    ui.add_space(40.0);
                });
            });
        } else {
            // Clone items to avoid borrowing issues
            let downloads_to_render: Vec<DownloadItem> = filtered_downloads.iter().map(|item| (*item).clone()).collect();
            for download in downloads_to_render {
                self.render_download_item(ui, &download);
                ui.add_space(8.0);
            }
        }
    }
}

impl DownloadsPage {
    fn render_download_item(&mut self, ui: &mut Ui, download: &DownloadItem) {
        let is_selected = self.selected_download.as_ref() == Some(&download.id);
        
        let frame_color = if is_selected {
            NeonTheme::NEON_CYAN.linear_multiply(0.3)
        } else {
            NeonTheme::CARD_BG
        };
        
        let response = egui::Frame::none()
            .fill(frame_color)
            .rounding(12.0)
            .stroke(egui::Stroke::new(1.0, if is_selected { 
                NeonTheme::NEON_CYAN 
            } else { 
                NeonTheme::BORDER_COLOR 
            }))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Status icon
                    let (icon, color) = Self::get_status_icon_and_color(&download.status);
                    ui.label(RichText::new(icon).size(24.0).color(color));
                    
                    ui.add_space(12.0);
                    
                    // File info
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&download.filename)
                                .size(16.0)
                                .strong()
                                .color(NeonTheme::PRIMARY_TEXT));
                                
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(Self::format_file_size(download.file_size))
                                    .color(NeonTheme::SECONDARY_TEXT));
                            });
                        });
                        
                        ui.add_space(4.0);
                        
                        // URL
                        ui.label(RichText::new(&download.url)
                            .size(12.0)
                            .color(NeonTheme::SECONDARY_TEXT));
                        
                        ui.add_space(8.0);
                        
                        // Progress bar and status
                        match &download.status {
                            DownloadStatus::InProgress => {
                                let progress = download.downloaded_size as f32 / download.file_size as f32;
                                let progress_bar = egui::ProgressBar::new(progress)
                                    .text(format!("{}% - {} / {}", 
                                        (progress * 100.0) as u32,
                                        Self::format_file_size(download.downloaded_size),
                                        Self::format_file_size(download.file_size)))
                                    .fill(NeonTheme::NEON_CYAN);
                                ui.add(progress_bar);
                            },
                            DownloadStatus::Completed => {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("✓ Download completed")
                                        .color(NeonTheme::success_color()));
                                    
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui.button("Open Folder").clicked() {
                                            // TODO: Open download folder
                                        }
                                        
                                        if ui.button("Open File").clicked() {
                                            // TODO: Open downloaded file
                                        }
                                    });
                                });
                            },
                            DownloadStatus::Paused => {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("⏸ Download paused")
                                        .color(NeonTheme::warning_color()));
                                    
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui.button("Resume").clicked() {
                                            // TODO: Resume download
                                        }
                                        
                                        if ui.button("Cancel").clicked() {
                                            // TODO: Cancel download
                                        }
                                    });
                                });
                            },
                            DownloadStatus::Failed(error) => {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("❌ Failed: {}", error))
                                        .color(NeonTheme::error_color()));
                                    
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui.button("Retry").clicked() {
                                            // TODO: Retry download
                                        }
                                        
                                        if ui.button("Remove").clicked() {
                                            // TODO: Remove from list
                                        }
                                    });
                                });
                            },
                            DownloadStatus::Cancelled => {
                                ui.label(RichText::new("✕ Download cancelled")
                                    .color(NeonTheme::SECONDARY_TEXT));
                            },
                        }
                    });
                });
            })
            .response;
        
        if response.clicked() {
            self.selected_download = if is_selected {
                None
            } else {
                Some(download.id.clone())
            };
        }
    }
}