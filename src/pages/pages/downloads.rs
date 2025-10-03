use eframe::egui::{Context, Ui, RichText, Layout, Align};
use crate::pages::{CustomPage, components};
use crate::ui::theme::NeonTheme;
use crate::ui::icons::NeonIcons;
use std::time::SystemTime;
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use crate::engine::download_manager::{DownloadManager, DownloadEvent};
use crate::storage::DownloadState;

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

impl From<DownloadState> for DownloadStatus {
    fn from(state: DownloadState) -> Self {
        match state {
            DownloadState::Pending | DownloadState::InProgress => DownloadStatus::InProgress,
            DownloadState::Completed => DownloadStatus::Completed,
            DownloadState::Paused => DownloadStatus::Paused,
            DownloadState::Failed => DownloadStatus::Failed("Download failed".to_string()),
            DownloadState::Cancelled => DownloadStatus::Cancelled,
        }
    }
}

pub struct DownloadsPage {
    url: String,
    title: String,
    downloads: Vec<DownloadItem>,
    search_query: String,
    show_only_active: bool,
    selected_download: Option<String>,
    download_manager: Option<Arc<Mutex<DownloadManager>>>,
    last_update: SystemTime,
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
        
        // Try to initialize download manager with persistent storage
        let download_manager = Self::init_download_manager();
        
        Self {
            url: "neon://downloads".to_string(),
            title: "Downloads".to_string(),
            downloads,
            search_query: String::new(),
            show_only_active: false,
            selected_download: None,
            download_manager,
            last_update: SystemTime::now(),
        }
    }
    
    fn init_download_manager() -> Option<Arc<Mutex<DownloadManager>>> {
        // Create data directory for downloads database
        let data_dir = dirs::data_dir()
            .or_else(|| std::env::current_dir().ok())
            .map(|d| d.join("NeonSearch"));
        
        if let Some(dir) = data_dir {
            let _ = std::fs::create_dir_all(&dir);
            let db_path = dir.join("downloads.db");
            
            match DownloadManager::new(&db_path) {
                Ok(manager) => {
                    println!("Download manager initialized with database: {:?}", db_path);
                    Some(Arc::new(Mutex::new(manager)))
                }
                Err(e) => {
                    eprintln!("Failed to initialize download manager: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }
    
    pub fn set_download_manager(&mut self, manager: Arc<Mutex<DownloadManager>>) {
        self.download_manager = Some(manager);
    }
    
    fn sync_with_manager(&mut self) {
        // Only update every second to avoid excessive DB queries
        if let Ok(elapsed) = self.last_update.elapsed() {
            if elapsed.as_secs() < 1 {
                return;
            }
        }
        
        self.last_update = SystemTime::now();
        
        if let Some(ref manager) = self.download_manager {
            if let Ok(mgr) = manager.lock() {
                // Poll for events
                let events = mgr.poll_events();
                for event in events {
                    match event {
                        DownloadEvent::Progress(progress) => {
                            // Update progress for existing download
                            if let Some(item) = self.downloads.iter_mut().find(|d| d.id == progress.id) {
                                item.downloaded_size = progress.downloaded_bytes;
                                item.file_size = progress.total_bytes.unwrap_or(item.file_size);
                            }
                        }
                        DownloadEvent::Completed(id, _path) => {
                            if let Some(item) = self.downloads.iter_mut().find(|d| d.id == id) {
                                item.status = DownloadStatus::Completed;
                            }
                        }
                        DownloadEvent::Failed(id, error) => {
                            if let Some(item) = self.downloads.iter_mut().find(|d| d.id == id) {
                                item.status = DownloadStatus::Failed(error);
                            }
                        }
                        DownloadEvent::Paused(id) => {
                            if let Some(item) = self.downloads.iter_mut().find(|d| d.id == id) {
                                item.status = DownloadStatus::Paused;
                            }
                        }
                        DownloadEvent::Cancelled(id) => {
                            if let Some(item) = self.downloads.iter_mut().find(|d| d.id == id) {
                                item.status = DownloadStatus::Cancelled;
                            }
                        }
                        _ => {}
                    }
                }
                
                // Sync downloads from database
                if let Ok(records) = mgr.get_download_history() {
                    // Update or add download items
                    for record in records {
                        let exists = self.downloads.iter().any(|d| d.id == record.id);
                        
                        if !exists {
                            self.downloads.push(DownloadItem {
                                id: record.id,
                                filename: record.filename,
                                url: record.url,
                                file_size: record.file_size.unwrap_or(0),
                                downloaded_size: record.downloaded_bytes,
                                status: record.status.into(),
                                start_time: SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(
                                    record.created_at.timestamp() as u64
                                ),
                                download_path: record.save_path,
                                mime_type: record.mime_type.unwrap_or_default(),
                            });
                        }
                    }
                }
            }
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
        // Sync with download manager
        self.sync_with_manager();
        
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
                                let progress = if download.file_size > 0 {
                                    download.downloaded_size as f32 / download.file_size as f32
                                } else {
                                    0.0
                                };
                                
                                // Get real-time progress from manager if available
                                let progress_info = if let Some(ref mgr) = self.download_manager {
                                    if let Ok(m) = mgr.lock() {
                                        m.get_progress(&download.id)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };
                                
                                let progress_text = if let Some(info) = progress_info {
                                    let speed_str = if info.speed_bps > 1024 * 1024 {
                                        format!("{:.2} MB/s", info.speed_bps as f64 / (1024.0 * 1024.0))
                                    } else if info.speed_bps > 1024 {
                                        format!("{:.1} KB/s", info.speed_bps as f64 / 1024.0)
                                    } else {
                                        format!("{} B/s", info.speed_bps)
                                    };
                                    
                                    let eta_str = if let Some(eta) = info.eta_seconds {
                                        if eta > 3600 {
                                            format!(" - {}h {}m remaining", eta / 3600, (eta % 3600) / 60)
                                        } else if eta > 60 {
                                            format!(" - {}m {}s remaining", eta / 60, eta % 60)
                                        } else {
                                            format!(" - {}s remaining", eta)
                                        }
                                    } else {
                                        String::new()
                                    };
                                    
                                    format!("{:.1}% - {} / {} - {}{}", 
                                        info.progress_percent,
                                        Self::format_file_size(info.downloaded_bytes),
                                        Self::format_file_size(info.total_bytes.unwrap_or(download.file_size)),
                                        speed_str,
                                        eta_str)
                                } else {
                                    format!("{}% - {} / {}", 
                                        (progress * 100.0) as u32,
                                        Self::format_file_size(download.downloaded_size),
                                        Self::format_file_size(download.file_size))
                                };
                                
                                let progress_bar = egui::ProgressBar::new(progress)
                                    .text(progress_text)
                                    .fill(NeonTheme::NEON_CYAN);
                                ui.add(progress_bar);
                                
                                // Add pause and cancel buttons
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    if ui.button(RichText::new(format!("{} Pause", NeonIcons::PAUSE))
                                        .color(NeonTheme::warning_color())).clicked() {
                                        if let Some(ref mgr) = self.download_manager {
                                            let mgr = mgr.clone();
                                            let id = download.id.clone();
                                            std::thread::spawn(move || {
                                                let rt = tokio::runtime::Runtime::new().unwrap();
                                                rt.block_on(async {
                                                    if let Ok(m) = mgr.lock() {
                                                        let _ = m.pause_download(&id).await;
                                                    }
                                                });
                                            });
                                        }
                                    }
                                    
                                    if ui.button(RichText::new(format!("{} Cancel", NeonIcons::CROSS))
                                        .color(NeonTheme::error_color())).clicked() {
                                        if let Some(ref mgr) = self.download_manager {
                                            let mgr = mgr.clone();
                                            let id = download.id.clone();
                                            std::thread::spawn(move || {
                                                let rt = tokio::runtime::Runtime::new().unwrap();
                                                rt.block_on(async {
                                                    if let Ok(m) = mgr.lock() {
                                                        let _ = m.cancel_download(&id).await;
                                                    }
                                                });
                                            });
                                        }
                                    }
                                });
                            },
                            DownloadStatus::Completed => {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("✓ Download completed")
                                        .color(NeonTheme::success_color()));
                                    
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui.button(RichText::new(format!("{} Open Folder", NeonIcons::FOLDER_OPEN))
                                            .color(NeonTheme::NEON_CYAN)).clicked() {
                                            Self::open_file_location(&download.download_path);
                                        }
                                        
                                        if ui.button(RichText::new(format!("{} Open File", NeonIcons::PLAY))
                                            .color(NeonTheme::NEON_CYAN)).clicked() {
                                            Self::open_file(&download.download_path);
                                        }
                                    });
                                });
                            },
                            DownloadStatus::Paused => {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("⏸ Download paused")
                                        .color(NeonTheme::warning_color()));
                                    
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui.button(RichText::new(format!("{} Resume", NeonIcons::PLAY))
                                            .color(NeonTheme::NEON_CYAN)).clicked() {
                                            if let Some(ref mgr) = self.download_manager {
                                                let mgr = mgr.clone();
                                                let id = download.id.clone();
                                                std::thread::spawn(move || {
                                                    let rt = tokio::runtime::Runtime::new().unwrap();
                                                    rt.block_on(async {
                                                        if let Ok(m) = mgr.lock() {
                                                            let _ = m.resume_download(&id).await;
                                                        }
                                                    });
                                                });
                                            }
                                        }
                                        
                                        if ui.button(RichText::new(format!("{} Cancel", NeonIcons::CROSS))
                                            .color(NeonTheme::error_color())).clicked() {
                                            if let Some(ref mgr) = self.download_manager {
                                                let mgr = mgr.clone();
                                                let id = download.id.clone();
                                                std::thread::spawn(move || {
                                                    let rt = tokio::runtime::Runtime::new().unwrap();
                                                    rt.block_on(async {
                                                        if let Ok(m) = mgr.lock() {
                                                            let _ = m.cancel_download(&id).await;
                                                        }
                                                    });
                                                });
                                            }
                                        }
                                    });
                                });
                            },
                            DownloadStatus::Failed(error) => {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("❌ Failed: {}", error))
                                        .color(NeonTheme::error_color()));
                                    
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui.button(RichText::new(format!("{} Retry", NeonIcons::REFRESH))
                                            .color(NeonTheme::NEON_CYAN)).clicked() {
                                            if let Some(ref mgr) = self.download_manager {
                                                let mgr = mgr.clone();
                                                let id = download.id.clone();
                                                std::thread::spawn(move || {
                                                    let rt = tokio::runtime::Runtime::new().unwrap();
                                                    rt.block_on(async {
                                                        if let Ok(m) = mgr.lock() {
                                                            let _ = m.resume_download(&id).await;
                                                        }
                                                    });
                                                });
                                            }
                                        }
                                        
                                        if ui.button(RichText::new(format!("{} Remove", NeonIcons::DELETE))
                                            .color(NeonTheme::error_color())).clicked() {
                                            if let Some(ref mgr) = self.download_manager {
                                                if let Ok(m) = mgr.lock() {
                                                    let _ = m.delete_download(&download.id);
                                                }
                                            }
                                            self.downloads.retain(|d| d.id != download.id);
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
    
    /// Open a file with the system default application
    fn open_file(path: &str) {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("cmd")
                .args(&["/C", "start", "", path])
                .spawn();
        }
        
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg(path)
                .spawn();
        }
        
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(path)
                .spawn();
        }
    }
    
    /// Open file location in system file browser
    fn open_file_location(path: &str) {
        let path_buf = PathBuf::from(path);
        let parent = path_buf.parent().unwrap_or(Path::new("."));
        
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer")
                .arg(parent)
                .spawn();
        }
        
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg(parent)
                .spawn();
        }
        
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn();
        }
    }
}