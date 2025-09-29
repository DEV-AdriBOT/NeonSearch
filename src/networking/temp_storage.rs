use std::fs;
use std::path::{Path, PathBuf};
use std::io::{Write, Read};
use anyhow::{Result, anyhow};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Maximum content size for temporary files (100MB)
const MAX_TEMP_FILE_SIZE: usize = 100 * 1024 * 1024;

/// Manages temporary file storage for web page content
#[derive(Clone)]
pub struct TempStorageManager {
    temp_dir: PathBuf,
    active_files: Arc<Mutex<HashMap<String, PathBuf>>>,
}

/// Represents a temporary file containing page content
#[derive(Debug, Clone)]
pub struct TempFile {
    pub id: String,
    pub path: PathBuf,
    pub size: usize,
    pub content_type: Option<String>,
}

impl TempStorageManager {
    /// Create a new temporary storage manager
    pub fn new() -> Result<Self> {
        let temp_dir = std::env::temp_dir().join("neonsearch_cache");
        
        // Create temporary directory if it doesn't exist
        if !temp_dir.exists() {
            fs::create_dir_all(&temp_dir)
                .map_err(|e| anyhow!("Failed to create temp directory: {}", e))?;
        }
        
        // Clean up any existing files from previous runs
        Self::cleanup_directory(&temp_dir)?;
        
        Ok(Self {
            temp_dir,
            active_files: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Store content in a temporary file and return the file info
    pub fn store_content(&self, content: &[u8], content_type: Option<String>) -> Result<TempFile> {
        if content.len() > MAX_TEMP_FILE_SIZE {
            return Err(anyhow!("Content too large: {} bytes (max: {})", 
                              content.len(), MAX_TEMP_FILE_SIZE));
        }

        let file_id = Uuid::new_v4().to_string();
        let file_path = self.temp_dir.join(format!("{}.cache", file_id));
        
        // Write content to file
        let mut file = fs::File::create(&file_path)
            .map_err(|e| anyhow!("Failed to create temp file: {}", e))?;
        
        file.write_all(content)
            .map_err(|e| anyhow!("Failed to write to temp file: {}", e))?;
        
        file.flush()
            .map_err(|e| anyhow!("Failed to flush temp file: {}", e))?;

        // Track the file
        {
            let mut active_files = self.active_files.lock().unwrap();
            active_files.insert(file_id.clone(), file_path.clone());
        }

        Ok(TempFile {
            id: file_id,
            path: file_path,
            size: content.len(),
            content_type,
        })
    }

    /// Read content from a temporary file
    pub fn read_content(&self, temp_file: &TempFile) -> Result<Vec<u8>> {
        let mut file = fs::File::open(&temp_file.path)
            .map_err(|e| anyhow!("Failed to open temp file: {}", e))?;

        let mut content = Vec::with_capacity(temp_file.size);
        file.read_to_end(&mut content)
            .map_err(|e| anyhow!("Failed to read temp file: {}", e))?;

        Ok(content)
    }

    /// Read content as string with UTF-8 decoding
    pub fn read_content_as_string(&self, temp_file: &TempFile) -> Result<String> {
        let content = self.read_content(temp_file)?;
        String::from_utf8(content)
            .map_err(|e| anyhow!("Failed to decode content as UTF-8: {}", e))
    }

    /// Remove a specific temporary file
    pub fn remove_file(&self, temp_file: &TempFile) -> Result<()> {
        // Remove from tracking
        {
            let mut active_files = self.active_files.lock().unwrap();
            active_files.remove(&temp_file.id);
        }

        // Remove file from disk
        if temp_file.path.exists() {
            fs::remove_file(&temp_file.path)
                .map_err(|e| anyhow!("Failed to remove temp file: {}", e))?;
        }

        Ok(())
    }

    /// Remove all temporary files for cleanup
    pub fn cleanup_all(&self) -> Result<()> {
        let mut active_files = self.active_files.lock().unwrap();
        
        for (_, path) in active_files.drain() {
            if path.exists() {
                let _ = fs::remove_file(&path); // Best effort cleanup
            }
        }

        Ok(())
    }

    /// Get information about a temporary file by ID
    pub fn get_file_info(&self, file_id: &str) -> Option<PathBuf> {
        let active_files = self.active_files.lock().unwrap();
        active_files.get(file_id).cloned()
    }

    /// Get the number of active temporary files
    pub fn active_file_count(&self) -> usize {
        let active_files = self.active_files.lock().unwrap();
        active_files.len()
    }

    /// Get total size of all temporary files
    pub fn total_size(&self) -> Result<u64> {
        let active_files = self.active_files.lock().unwrap();
        let mut total_size = 0u64;

        for (_, path) in active_files.iter() {
            if path.exists() {
                let metadata = fs::metadata(path)
                    .map_err(|e| anyhow!("Failed to get file metadata: {}", e))?;
                total_size += metadata.len();
            }
        }

        Ok(total_size)
    }

    /// Clean up directory by removing all files
    fn cleanup_directory(dir: &Path) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(dir)
            .map_err(|e| anyhow!("Failed to read temp directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| anyhow!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "cache") {
                let _ = fs::remove_file(&path); // Best effort cleanup
            }
        }

        Ok(())
    }
}

impl Drop for TempStorageManager {
    fn drop(&mut self) {
        // Cleanup on drop - best effort
        let _ = self.cleanup_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_read_content() {
        let manager = TempStorageManager::new().unwrap();
        let content = b"Hello, NeonSearch!";
        
        let temp_file = manager.store_content(content, Some("text/plain".to_string())).unwrap();
        assert_eq!(temp_file.size, content.len());
        
        let read_content = manager.read_content(&temp_file).unwrap();
        assert_eq!(read_content, content);
        
        let read_string = manager.read_content_as_string(&temp_file).unwrap();
        assert_eq!(read_string, "Hello, NeonSearch!");
        
        manager.remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_file_size_limit() {
        let manager = TempStorageManager::new().unwrap();
        let large_content = vec![0u8; MAX_TEMP_FILE_SIZE + 1];
        
        let result = manager.store_content(&large_content, None);
        assert!(result.is_err());
    }
}
