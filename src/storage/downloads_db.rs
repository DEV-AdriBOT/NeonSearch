use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub enum DownloadState {
    Pending,
    InProgress,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl DownloadState {
    pub fn to_string(&self) -> &'static str {
        match self {
            DownloadState::Pending => "pending",
            DownloadState::InProgress => "in_progress",
            DownloadState::Paused => "paused",
            DownloadState::Completed => "completed",
            DownloadState::Failed => "failed",
            DownloadState::Cancelled => "cancelled",
        }
    }
    
    pub fn from_string(s: &str) -> Result<Self> {
        match s {
            "pending" => Ok(DownloadState::Pending),
            "in_progress" => Ok(DownloadState::InProgress),
            "paused" => Ok(DownloadState::Paused),
            "completed" => Ok(DownloadState::Completed),
            "failed" => Ok(DownloadState::Failed),
            "cancelled" => Ok(DownloadState::Cancelled),
            _ => Err(anyhow::anyhow!("Unknown download state: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DownloadRecord {
    pub id: String,
    pub filename: String,
    pub url: String,
    pub file_size: Option<u64>,
    pub downloaded_bytes: u64,
    pub status: DownloadState,
    pub mime_type: Option<String>,
    pub save_path: String,
    pub checksum: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub struct DownloadsDatabase {
    conn: Arc<Mutex<Connection>>,
}

impl DownloadsDatabase {
    /// Create a new downloads database at the specified path
    pub fn new(db_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create database directory")?;
        }
        
        let conn = Connection::open(db_path)
            .context("Failed to open downloads database")?;
        
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        db.initialize_schema()?;
        Ok(db)
    }
    
    /// Initialize the database schema
    fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS downloads (
                id TEXT PRIMARY KEY,
                filename TEXT NOT NULL,
                url TEXT NOT NULL,
                file_size INTEGER,
                downloaded_bytes INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL,
                mime_type TEXT,
                save_path TEXT NOT NULL,
                checksum TEXT,
                error_message TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                completed_at TEXT
            )",
            [],
        ).context("Failed to create downloads table")?;
        
        // Create indexes for common queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_downloads_status ON downloads(status)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_downloads_created_at ON downloads(created_at)",
            [],
        )?;
        
        Ok(())
    }
    
    /// Insert a new download record
    pub fn insert(&self, record: &DownloadRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT INTO downloads 
             (id, filename, url, file_size, downloaded_bytes, status, mime_type, 
              save_path, checksum, error_message, created_at, updated_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                record.id,
                record.filename,
                record.url,
                record.file_size,
                record.downloaded_bytes,
                record.status.to_string(),
                record.mime_type,
                record.save_path,
                record.checksum,
                record.error_message,
                record.created_at.to_rfc3339(),
                record.updated_at.to_rfc3339(),
                record.completed_at.map(|dt| dt.to_rfc3339()),
            ],
        ).context("Failed to insert download record")?;
        
        Ok(())
    }
    
    /// Update an existing download record
    pub fn update(&self, record: &DownloadRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "UPDATE downloads 
             SET filename = ?2, url = ?3, file_size = ?4, downloaded_bytes = ?5, 
                 status = ?6, mime_type = ?7, save_path = ?8, checksum = ?9, 
                 error_message = ?10, updated_at = ?11, completed_at = ?12
             WHERE id = ?1",
            params![
                record.id,
                record.filename,
                record.url,
                record.file_size,
                record.downloaded_bytes,
                record.status.to_string(),
                record.mime_type,
                record.save_path,
                record.checksum,
                record.error_message,
                record.updated_at.to_rfc3339(),
                record.completed_at.map(|dt| dt.to_rfc3339()),
            ],
        ).context("Failed to update download record")?;
        
        Ok(())
    }
    
    /// Get a download record by ID
    pub fn get_by_id(&self, id: &str) -> Result<Option<DownloadRecord>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, filename, url, file_size, downloaded_bytes, status, mime_type,
                    save_path, checksum, error_message, created_at, updated_at, completed_at
             FROM downloads WHERE id = ?1"
        )?;
        
        let mut rows = stmt.query(params![id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_record(row)?))
        } else {
            Ok(None)
        }
    }
    
    /// Get all download records
    pub fn get_all(&self) -> Result<Vec<DownloadRecord>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, filename, url, file_size, downloaded_bytes, status, mime_type,
                    save_path, checksum, error_message, created_at, updated_at, completed_at
             FROM downloads ORDER BY created_at DESC"
        )?;
        
        let rows = stmt.query_map([], |row| Ok(self.row_to_record(row)))?;
        
        let mut records = Vec::new();
        for row_result in rows {
            match row_result {
                Ok(Ok(record)) => records.push(record),
                _ => continue,
            }
        }
        
        Ok(records)
    }
    
    /// Get downloads by status
    pub fn get_by_status(&self, status: DownloadState) -> Result<Vec<DownloadRecord>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, filename, url, file_size, downloaded_bytes, status, mime_type,
                    save_path, checksum, error_message, created_at, updated_at, completed_at
             FROM downloads WHERE status = ?1 ORDER BY created_at DESC"
        )?;
        
        let rows = stmt.query_map(params![status.to_string()], |row| {
            Ok(self.row_to_record(row))
        })?;
        
        let mut records = Vec::new();
        for row_result in rows {
            match row_result {
                Ok(Ok(record)) => records.push(record),
                _ => continue,
            }
        }
        
        Ok(records)
    }
    
    /// Search downloads by filename or URL
    pub fn search(&self, query: &str) -> Result<Vec<DownloadRecord>> {
        let conn = self.conn.lock().unwrap();
        let search_pattern = format!("%{}%", query);
        
        let mut stmt = conn.prepare(
            "SELECT id, filename, url, file_size, downloaded_bytes, status, mime_type,
                    save_path, checksum, error_message, created_at, updated_at, completed_at
             FROM downloads 
             WHERE filename LIKE ?1 OR url LIKE ?1
             ORDER BY created_at DESC"
        )?;
        
        let rows = stmt.query_map(params![search_pattern], |row| {
            Ok(self.row_to_record(row))
        })?;
        
        let mut records = Vec::new();
        for row_result in rows {
            match row_result {
                Ok(Ok(record)) => records.push(record),
                _ => continue,
            }
        }
        
        Ok(records)
    }
    
    /// Delete a download record
    pub fn delete(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute("DELETE FROM downloads WHERE id = ?1", params![id])
            .context("Failed to delete download record")?;
        
        Ok(())
    }
    
    /// Delete all completed downloads older than the specified days
    pub fn cleanup_old_completed(&self, days: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let cutoff = Utc::now() - chrono::Duration::days(days);
        
        let deleted = conn.execute(
            "DELETE FROM downloads 
             WHERE status = 'completed' AND completed_at < ?1",
            params![cutoff.to_rfc3339()],
        ).context("Failed to cleanup old downloads")?;
        
        Ok(deleted)
    }
    
    /// Helper to convert a database row to a DownloadRecord
    fn row_to_record(&self, row: &rusqlite::Row) -> Result<DownloadRecord> {
        let created_at_str: String = row.get(10)?;
        let updated_at_str: String = row.get(11)?;
        let completed_at_str: Option<String> = row.get(12)?;
        
        Ok(DownloadRecord {
            id: row.get(0)?,
            filename: row.get(1)?,
            url: row.get(2)?,
            file_size: row.get(3)?,
            downloaded_bytes: row.get(4)?,
            status: DownloadState::from_string(&row.get::<_, String>(5)?)?,
            mime_type: row.get(6)?,
            save_path: row.get(7)?,
            checksum: row.get(8)?,
            error_message: row.get(9)?,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)?.with_timezone(&Utc),
            completed_at: completed_at_str
                .map(|s| DateTime::parse_from_rfc3339(&s).ok())
                .flatten()
                .map(|dt| dt.with_timezone(&Utc)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    fn create_test_db() -> Result<DownloadsDatabase> {
        let test_path = std::env::temp_dir().join(format!("test_downloads_{}.db", Uuid::new_v4()));
        DownloadsDatabase::new(&test_path)
    }
    
    #[test]
    fn test_insert_and_retrieve() -> Result<()> {
        let db = create_test_db()?;
        
        let record = DownloadRecord {
            id: "test-download-1".to_string(),
            filename: "test.pdf".to_string(),
            url: "https://example.com/test.pdf".to_string(),
            file_size: Some(1024000),
            downloaded_bytes: 512000,
            status: DownloadState::InProgress,
            mime_type: Some("application/pdf".to_string()),
            save_path: "/tmp/test.pdf".to_string(),
            checksum: None,
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
        };
        
        db.insert(&record)?;
        
        let retrieved = db.get_by_id("test-download-1")?;
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.filename, "test.pdf");
        assert_eq!(retrieved.downloaded_bytes, 512000);
        
        Ok(())
    }
    
    #[test]
    fn test_update() -> Result<()> {
        let db = create_test_db()?;
        
        let mut record = DownloadRecord {
            id: "test-download-2".to_string(),
            filename: "test2.zip".to_string(),
            url: "https://example.com/test2.zip".to_string(),
            file_size: Some(2048000),
            downloaded_bytes: 0,
            status: DownloadState::Pending,
            mime_type: Some("application/zip".to_string()),
            save_path: "/tmp/test2.zip".to_string(),
            checksum: None,
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
        };
        
        db.insert(&record)?;
        
        record.downloaded_bytes = 1024000;
        record.status = DownloadState::InProgress;
        record.updated_at = Utc::now();
        
        db.update(&record)?;
        
        let retrieved = db.get_by_id("test-download-2")?.unwrap();
        assert_eq!(retrieved.downloaded_bytes, 1024000);
        assert_eq!(retrieved.status, DownloadState::InProgress);
        
        Ok(())
    }
    
    #[test]
    fn test_get_by_status() -> Result<()> {
        let db = create_test_db()?;
        
        let record1 = DownloadRecord {
            id: "test-download-3".to_string(),
            filename: "test3.pdf".to_string(),
            url: "https://example.com/test3.pdf".to_string(),
            file_size: Some(1024000),
            downloaded_bytes: 1024000,
            status: DownloadState::Completed,
            mime_type: Some("application/pdf".to_string()),
            save_path: "/tmp/test3.pdf".to_string(),
            checksum: None,
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };
        
        db.insert(&record1)?;
        
        let completed = db.get_by_status(DownloadState::Completed)?;
        assert!(completed.len() >= 1);
        assert!(completed.iter().any(|r| r.id == "test-download-3"));
        
        Ok(())
    }
}
