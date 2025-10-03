use anyhow::{Result, Context, anyhow};
use chrono::Utc;
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::storage::{DownloadsDatabase, DownloadRecord, DownloadState};

const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks
const MAX_CONCURRENT_DOWNLOADS: usize = 3;
const MAX_RETRY_ATTEMPTS: usize = 3;
const INITIAL_RETRY_DELAY: Duration = Duration::from_secs(2);

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub speed_bps: u64, // Bytes per second
    pub progress_percent: f32,
    pub eta_seconds: Option<u64>,
    pub status: DownloadState,
}

#[derive(Debug, Clone)]
pub enum DownloadEvent {
    Started(String),
    Progress(DownloadProgress),
    Completed(String, PathBuf),
    Failed(String, String),
    Paused(String),
    Resumed(String),
    Cancelled(String),
}

#[derive(Clone)]
pub struct DownloadTask {
    pub id: String,
    pub url: String,
    pub save_path: PathBuf,
    pub filename: String,
    pub resume_from: u64,
}

struct ActiveDownload {
    task: DownloadTask,
    cancel_tx: mpsc::Sender<()>,
    start_time: Instant,
    last_progress: u64,
    last_progress_time: Instant,
}

pub struct DownloadManager {
    db: Arc<DownloadsDatabase>,
    active_downloads: Arc<Mutex<HashMap<String, ActiveDownload>>>,
    event_tx: mpsc::UnboundedSender<DownloadEvent>,
    event_rx: Arc<Mutex<mpsc::UnboundedReceiver<DownloadEvent>>>,
    download_semaphore: Arc<tokio::sync::Semaphore>,
    throttle_bps: Option<u64>, // Bandwidth throttling in bytes per second
}

impl DownloadManager {
    pub fn new(db_path: &Path) -> Result<Self> {
        let db = Arc::new(DownloadsDatabase::new(db_path)?);
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        Ok(Self {
            db,
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
            download_semaphore: Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_DOWNLOADS)),
            throttle_bps: None,
        })
    }
    
    /// Set bandwidth throttle in bytes per second
    pub fn set_bandwidth_throttle(&mut self, bps: Option<u64>) {
        self.throttle_bps = bps;
    }
    
    /// Start a new download
    pub async fn start_download(&self, url: String, save_path: PathBuf) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        
        // Extract filename from URL or path
        let filename = save_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("download")
            .to_string();
        
        // Create download record
        let record = DownloadRecord {
            id: id.clone(),
            filename: filename.clone(),
            url: url.clone(),
            file_size: None,
            downloaded_bytes: 0,
            status: DownloadState::Pending,
            mime_type: None,
            save_path: save_path.to_string_lossy().to_string(),
            checksum: None,
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
        };
        
        self.db.insert(&record)?;
        
        let task = DownloadTask {
            id: id.clone(),
            url,
            save_path,
            filename,
            resume_from: 0,
        };
        
        self.spawn_download(task).await?;
        
        Ok(id)
    }
    
    /// Resume a paused download
    pub async fn resume_download(&self, id: &str) -> Result<()> {
        let record = self.db.get_by_id(id)?
            .ok_or_else(|| anyhow!("Download not found"))?;
        
        if record.status != DownloadState::Paused && record.status != DownloadState::Failed {
            return Err(anyhow!("Download cannot be resumed from status: {:?}", record.status));
        }
        
        let task = DownloadTask {
            id: id.to_string(),
            url: record.url,
            save_path: PathBuf::from(record.save_path),
            filename: record.filename,
            resume_from: record.downloaded_bytes,
        };
        
        self.spawn_download(task).await?;
        
        Ok(())
    }
    
    /// Pause an active download
    pub async fn pause_download(&self, id: &str) -> Result<()> {
        let mut active = self.active_downloads.lock().unwrap();
        
        if let Some(download) = active.remove(id) {
            let _ = download.cancel_tx.send(()).await;
            
            // Update database
            if let Ok(Some(mut record)) = self.db.get_by_id(id) {
                record.status = DownloadState::Paused;
                record.updated_at = Utc::now();
                let _ = self.db.update(&record);
            }
            
            let _ = self.event_tx.send(DownloadEvent::Paused(id.to_string()));
        }
        
        Ok(())
    }
    
    /// Cancel a download
    pub async fn cancel_download(&self, id: &str) -> Result<()> {
        let mut active = self.active_downloads.lock().unwrap();
        
        if let Some(download) = active.remove(id) {
            let _ = download.cancel_tx.send(()).await;
            
            // Clean up partial file
            let _ = tokio::fs::remove_file(&download.task.save_path).await;
            
            // Update database
            if let Ok(Some(mut record)) = self.db.get_by_id(id) {
                record.status = DownloadState::Cancelled;
                record.updated_at = Utc::now();
                let _ = self.db.update(&record);
            }
            
            let _ = self.event_tx.send(DownloadEvent::Cancelled(id.to_string()));
        }
        
        Ok(())
    }
    
    /// Get download progress
    pub fn get_progress(&self, id: &str) -> Option<DownloadProgress> {
        let active = self.active_downloads.lock().unwrap();
        
        if let Some(download) = active.get(id) {
            if let Ok(Some(record)) = self.db.get_by_id(id) {
                let elapsed = download.start_time.elapsed().as_secs_f64();
                let speed_bps = if elapsed > 0.0 {
                    (download.last_progress as f64 / elapsed) as u64
                } else {
                    0
                };
                
                let progress_percent = if let Some(total) = record.file_size {
                    (record.downloaded_bytes as f32 / total as f32) * 100.0
                } else {
                    0.0
                };
                
                let eta_seconds = if speed_bps > 0 && record.file_size.is_some() {
                    let remaining = record.file_size.unwrap() - record.downloaded_bytes;
                    Some(remaining / speed_bps)
                } else {
                    None
                };
                
                return Some(DownloadProgress {
                    id: id.to_string(),
                    downloaded_bytes: record.downloaded_bytes,
                    total_bytes: record.file_size,
                    speed_bps,
                    progress_percent,
                    eta_seconds,
                    status: record.status,
                });
            }
        }
        
        None
    }
    
    /// Get all active downloads
    pub fn get_active_downloads(&self) -> Vec<String> {
        let active = self.active_downloads.lock().unwrap();
        active.keys().cloned().collect()
    }
    
    /// Poll for download events
    pub fn poll_events(&self) -> Vec<DownloadEvent> {
        let mut events = Vec::new();
        let mut rx = self.event_rx.lock().unwrap();
        
        while let Ok(event) = rx.try_recv() {
            events.push(event);
        }
        
        events
    }
    
    /// Spawn a download task
    async fn spawn_download(&self, task: DownloadTask) -> Result<()> {
        let (cancel_tx, cancel_rx) = mpsc::channel(1);
        
        let active_download = ActiveDownload {
            task: task.clone(),
            cancel_tx,
            start_time: Instant::now(),
            last_progress: task.resume_from,
            last_progress_time: Instant::now(),
        };
        
        {
            let mut active = self.active_downloads.lock().unwrap();
            active.insert(task.id.clone(), active_download);
        }
        
        let db = self.db.clone();
        let event_tx = self.event_tx.clone();
        let active_downloads = self.active_downloads.clone();
        let semaphore = self.download_semaphore.clone();
        let throttle = self.throttle_bps;
        
        tokio::spawn(async move {
            // Acquire semaphore permit (limits concurrent downloads)
            let _permit = semaphore.acquire().await.unwrap();
            
            let result = Self::download_file(
                &task,
                db.clone(),
                event_tx.clone(),
                cancel_rx,
                throttle,
            ).await;
            
            // Remove from active downloads
            {
                let mut active = active_downloads.lock().unwrap();
                active.remove(&task.id);
            }
            
            // Handle result
            match result {
                Ok(path) => {
                    let _ = event_tx.send(DownloadEvent::Completed(task.id.clone(), path));
                }
                Err(e) => {
                    let _ = event_tx.send(DownloadEvent::Failed(task.id.clone(), e.to_string()));
                }
            }
        });
        
        Ok(())
    }
    
    /// Download a file with retry logic
    async fn download_file(
        task: &DownloadTask,
        db: Arc<DownloadsDatabase>,
        event_tx: mpsc::UnboundedSender<DownloadEvent>,
        mut cancel_rx: mpsc::Receiver<()>,
        throttle_bps: Option<u64>,
    ) -> Result<PathBuf> {
        let mut retry_count = 0;
        let mut retry_delay = INITIAL_RETRY_DELAY;
        
        loop {
            let result = Self::download_file_attempt(
                task,
                db.clone(),
                event_tx.clone(),
                &mut cancel_rx,
                throttle_bps,
            ).await;
            
            match result {
                Ok(path) => return Ok(path),
                Err(e) if retry_count < MAX_RETRY_ATTEMPTS => {
                    retry_count += 1;
                    println!("Download attempt {} failed: {}. Retrying in {:?}...", 
                             retry_count, e, retry_delay);
                    
                    tokio::time::sleep(retry_delay).await;
                    retry_delay *= 2; // Exponential backoff
                }
                Err(e) => {
                    // Max retries exceeded, update status to failed
                    if let Ok(Some(mut record)) = db.get_by_id(&task.id) {
                        record.status = DownloadState::Failed;
                        record.error_message = Some(e.to_string());
                        record.updated_at = Utc::now();
                        let _ = db.update(&record);
                    }
                    return Err(e);
                }
            }
        }
    }
    
    /// Single download attempt
    async fn download_file_attempt(
        task: &DownloadTask,
        db: Arc<DownloadsDatabase>,
        event_tx: mpsc::UnboundedSender<DownloadEvent>,
        cancel_rx: &mut mpsc::Receiver<()>,
        throttle_bps: Option<u64>,
    ) -> Result<PathBuf> {
        // Create client
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()?;
        
        // Make HEAD request to get file size
        let head_response = client.head(&task.url).send().await?;
        let total_bytes = head_response
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());
        
        let mime_type = head_response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        
        // Update record with file size
        if let Ok(Some(mut record)) = db.get_by_id(&task.id) {
            record.file_size = total_bytes;
            record.mime_type = mime_type;
            record.status = DownloadState::InProgress;
            record.updated_at = Utc::now();
            let _ = db.update(&record);
        }
        
        let _ = event_tx.send(DownloadEvent::Started(task.id.clone()));
        
        // Build download request with Range header if resuming
        let mut request = client.get(&task.url);
        if task.resume_from > 0 {
            request = request.header(
                reqwest::header::RANGE,
                format!("bytes={}-", task.resume_from),
            );
        }
        
        let response = request.send().await?;
        
        if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
            return Err(anyhow!("HTTP error: {}", response.status()));
        }
        
        // Ensure parent directory exists
        if let Some(parent) = task.save_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        // Open file for writing (append if resuming)
        let mut file = if task.resume_from > 0 {
            tokio::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(&task.save_path)
                .await?
        } else {
            File::create(&task.save_path).await?
        };
        
        let mut downloaded = task.resume_from;
        let mut hasher = Sha256::new();
        let start_time = Instant::now();
        let mut last_progress_time = Instant::now();
        
        // Stream download
        let mut stream = response.bytes_stream();
        use futures_core::Stream;
        use futures_util::StreamExt;
        
        while let Some(chunk) = tokio::select! {
            chunk = stream.next() => chunk,
            _ = cancel_rx.recv() => {
                // Download cancelled
                return Err(anyhow!("Download cancelled"));
            }
        } {
            let chunk = chunk?;
            
            // Apply bandwidth throttling if enabled
            if let Some(max_bps) = throttle_bps {
                let elapsed = last_progress_time.elapsed().as_secs_f64();
                if elapsed > 0.0 {
                    let current_bps = chunk.len() as f64 / elapsed;
                    if current_bps > max_bps as f64 {
                        let delay = Duration::from_secs_f64(chunk.len() as f64 / max_bps as f64 - elapsed);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
            
            file.write_all(&chunk).await?;
            hasher.update(&chunk);
            downloaded += chunk.len() as u64;
            
            // Update progress every 500ms
            if last_progress_time.elapsed() >= Duration::from_millis(500) {
                let elapsed = start_time.elapsed().as_secs_f64();
                let speed_bps = if elapsed > 0.0 {
                    ((downloaded - task.resume_from) as f64 / elapsed) as u64
                } else {
                    0
                };
                
                let progress_percent = if let Some(total) = total_bytes {
                    (downloaded as f32 / total as f32) * 100.0
                } else {
                    0.0
                };
                
                let eta_seconds = if speed_bps > 0 && total_bytes.is_some() {
                    let remaining = total_bytes.unwrap() - downloaded;
                    Some(remaining / speed_bps)
                } else {
                    None
                };
                
                // Update database
                if let Ok(Some(mut record)) = db.get_by_id(&task.id) {
                    record.downloaded_bytes = downloaded;
                    record.updated_at = Utc::now();
                    let _ = db.update(&record);
                }
                
                // Send progress event
                let progress = DownloadProgress {
                    id: task.id.clone(),
                    downloaded_bytes: downloaded,
                    total_bytes,
                    speed_bps,
                    progress_percent,
                    eta_seconds,
                    status: DownloadState::InProgress,
                };
                
                let _ = event_tx.send(DownloadEvent::Progress(progress));
                last_progress_time = Instant::now();
            }
        }
        
        file.flush().await?;
        
        // Calculate checksum
        let checksum = format!("{:x}", hasher.finalize());
        
        // Update record as completed
        if let Ok(Some(mut record)) = db.get_by_id(&task.id) {
            record.downloaded_bytes = downloaded;
            record.status = DownloadState::Completed;
            record.checksum = Some(checksum);
            record.updated_at = Utc::now();
            record.completed_at = Some(Utc::now());
            let _ = db.update(&record);
        }
        
        Ok(task.save_path.clone())
    }
    
    /// Verify file integrity using checksum
    pub async fn verify_file(path: &Path, expected_checksum: &str) -> Result<bool> {
        use tokio::io::AsyncReadExt;
        
        let mut file = File::open(path).await?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; CHUNK_SIZE];
        
        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        
        let checksum = format!("{:x}", hasher.finalize());
        Ok(checksum == expected_checksum)
    }
    
    /// Get download history from database
    pub fn get_download_history(&self) -> Result<Vec<DownloadRecord>> {
        self.db.get_all()
    }
    
    /// Get downloads by status
    pub fn get_downloads_by_status(&self, status: DownloadState) -> Result<Vec<DownloadRecord>> {
        self.db.get_by_status(status)
    }
    
    /// Search downloads
    pub fn search_downloads(&self, query: &str) -> Result<Vec<DownloadRecord>> {
        self.db.search(query)
    }
    
    /// Delete download record
    pub fn delete_download(&self, id: &str) -> Result<()> {
        self.db.delete(id)
    }
    
    /// Cleanup old completed downloads
    pub fn cleanup_old_downloads(&self, days: i64) -> Result<usize> {
        self.db.cleanup_old_completed(days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_download_manager_creation() -> Result<()> {
        let test_db = std::env::temp_dir().join(format!("test_dm_{}.db", Uuid::new_v4()));
        let manager = DownloadManager::new(&test_db)?;
        assert_eq!(manager.get_active_downloads().len(), 0);
        Ok(())
    }
}
