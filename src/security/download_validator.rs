use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};

/// List of allowed file extensions (can be configured)
const ALLOWED_EXTENSIONS: &[&str] = &[
    // Documents
    "pdf", "doc", "docx", "txt", "rtf", "odt", "xls", "xlsx", "ppt", "pptx", "csv",
    // Archives
    "zip", "tar", "gz", "bz2", "7z", "rar", "xz",
    // Images
    "jpg", "jpeg", "png", "gif", "bmp", "svg", "webp", "ico",
    // Audio
    "mp3", "wav", "ogg", "flac", "m4a", "aac",
    // Video
    "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm",
    // Code/Text
    "html", "css", "js", "json", "xml", "yaml", "yml", "md", "rs", "py", "java", "c", "cpp",
    // Data
    "db", "sqlite", "sql",
];

/// List of executable file extensions that require warnings
const EXECUTABLE_EXTENSIONS: &[&str] = &[
    "exe", "bat", "cmd", "com", "msi", "scr", "vbs", "js", "jar",
    "app", "dmg", "pkg", "deb", "rpm", "sh", "run",
];

/// Common MIME types for validation
const SAFE_MIME_TYPES: &[&str] = &[
    "text/plain",
    "text/html",
    "text/css",
    "text/javascript",
    "application/json",
    "application/xml",
    "application/pdf",
    "application/zip",
    "application/x-tar",
    "application/x-gzip",
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
    "audio/mpeg",
    "audio/wav",
    "video/mp4",
    "video/webm",
];

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Safe,
    RequiresConfirmation(String),
    Rejected(String),
}

pub struct DownloadValidator;

impl DownloadValidator {
    /// Validate a filename for security issues
    pub fn validate_filename(filename: &str) -> Result<String> {
        // Check for empty filename
        if filename.trim().is_empty() {
            return Err(anyhow!("Filename cannot be empty"));
        }
        
        // Sanitize filename to prevent path traversal
        let sanitized = Self::sanitize_filename(filename);
        
        // Check if sanitized filename is valid
        if sanitized.is_empty() {
            return Err(anyhow!("Invalid filename after sanitization"));
        }
        
        Ok(sanitized)
    }
    
    /// Sanitize filename to prevent path traversal and other security issues
    pub fn sanitize_filename(filename: &str) -> String {
        // Remove any path components
        let filename = Path::new(filename)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(filename);
        
        // Remove dangerous characters
        let sanitized: String = filename
            .chars()
            .filter(|c| {
                // Allow alphanumeric, basic punctuation, and common filename chars
                c.is_alphanumeric()
                    || *c == '.'
                    || *c == '-'
                    || *c == '_'
                    || *c == ' '
                    || *c == '('
                    || *c == ')'
                    || *c == '['
                    || *c == ']'
            })
            .collect();
        
        // Trim whitespace and dots from edges
        let sanitized = sanitized.trim().trim_matches('.');
        
        // If filename becomes empty after sanitization, generate a default
        if sanitized.is_empty() {
            format!("download_{}", chrono::Utc::now().timestamp())
        } else {
            sanitized.to_string()
        }
    }
    
    /// Validate file extension
    pub fn validate_extension(filename: &str) -> ValidationResult {
        let path = Path::new(filename);
        
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase());
        
        match extension {
            Some(ext) if EXECUTABLE_EXTENSIONS.contains(&ext.as_str()) => {
                ValidationResult::RequiresConfirmation(format!(
                    "This file has an executable extension (.{}). It could potentially harm your computer. Are you sure you want to download it?",
                    ext
                ))
            }
            Some(ext) if ALLOWED_EXTENSIONS.contains(&ext.as_str()) => {
                ValidationResult::Safe
            }
            Some(ext) => {
                ValidationResult::RequiresConfirmation(format!(
                    "This file has an uncommon extension (.{}). It may not be safe. Do you want to proceed?",
                    ext
                ))
            }
            None => {
                ValidationResult::RequiresConfirmation(
                    "This file has no extension. It may not be safe. Do you want to proceed?".to_string()
                )
            }
        }
    }
    
    /// Validate MIME type
    pub fn validate_mime_type(mime_type: &str) -> ValidationResult {
        let mime_lower = mime_type.to_lowercase();
        
        // Check if it's a safe MIME type
        if SAFE_MIME_TYPES.iter().any(|&safe| mime_lower.starts_with(safe)) {
            return ValidationResult::Safe;
        }
        
        // Check for executable MIME types
        if mime_lower.contains("application/x-msdownload")
            || mime_lower.contains("application/x-executable")
            || mime_lower.contains("application/x-msdos-program")
            || mime_lower.contains("application/x-sh")
        {
            return ValidationResult::RequiresConfirmation(
                "This file appears to be executable and could potentially harm your computer. Are you sure you want to download it?".to_string()
            );
        }
        
        // Unknown MIME type - require confirmation
        ValidationResult::RequiresConfirmation(format!(
            "This file has an uncommon type ({}). It may not be safe. Do you want to proceed?",
            mime_type
        ))
    }
    
    /// Check available disk space
    pub fn check_disk_space(save_path: &Path, required_bytes: u64) -> Result<bool> {
        // Get the parent directory or use current directory
        let dir = save_path.parent().unwrap_or_else(|| Path::new("."));
        
        // Platform-specific disk space check
        #[cfg(target_family = "unix")]
        {
            use std::ffi::CString;
            use std::mem;
            
            let path_cstr = CString::new(dir.to_string_lossy().as_bytes())
                .map_err(|_| anyhow!("Invalid path"))?;
            
            unsafe {
                let mut stat: libc::statvfs = mem::zeroed();
                if libc::statvfs(path_cstr.as_ptr(), &mut stat) == 0 {
                    let available_bytes = stat.f_bavail as u64 * stat.f_bsize as u64;
                    // Require at least 100MB buffer plus file size
                    let required_with_buffer = required_bytes + 100 * 1024 * 1024;
                    return Ok(available_bytes >= required_with_buffer);
                }
            }
        }
        
        #[cfg(target_family = "windows")]
        {
            // For Windows, we'd use GetDiskFreeSpaceEx
            // For now, we'll be conservative and just assume there's space
            // A full implementation would use winapi
        }
        
        // If we can't check, assume there's space
        Ok(true)
    }
    
    /// Validate download URL
    pub fn validate_url(url: &str) -> Result<()> {
        // Basic URL validation
        if url.trim().is_empty() {
            return Err(anyhow!("URL cannot be empty"));
        }
        
        // Check for supported protocols
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(anyhow!("Only HTTP and HTTPS protocols are supported"));
        }
        
        // Check for localhost/private IPs to prevent SSRF
        let url_parsed = url::Url::parse(url)
            .map_err(|e| anyhow!("Invalid URL: {}", e))?;
        
        if let Some(host) = url_parsed.host_str() {
            if host == "localhost"
                || host == "127.0.0.1"
                || host.starts_with("192.168.")
                || host.starts_with("10.")
                || host.starts_with("172.16.")
                || host.starts_with("172.17.")
                || host.starts_with("172.18.")
                || host.starts_with("172.19.")
                || host.starts_with("172.20.")
                || host.starts_with("172.21.")
                || host.starts_with("172.22.")
                || host.starts_with("172.23.")
                || host.starts_with("172.24.")
                || host.starts_with("172.25.")
                || host.starts_with("172.26.")
                || host.starts_with("172.27.")
                || host.starts_with("172.28.")
                || host.starts_with("172.29.")
                || host.starts_with("172.30.")
                || host.starts_with("172.31.")
            {
                return Err(anyhow!("Downloads from private networks are not allowed"));
            }
        }
        
        Ok(())
    }
    
    /// Generate a safe download path
    pub fn generate_safe_path(download_dir: &Path, filename: &str) -> PathBuf {
        let sanitized = Self::sanitize_filename(filename);
        let mut path = download_dir.join(&sanitized);
        
        // If file exists, append a number
        let mut counter = 1;
        while path.exists() {
            let stem = Path::new(&sanitized)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("download");
            
            let extension = Path::new(&sanitized)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            
            let new_filename = if extension.is_empty() {
                format!("{} ({})", stem, counter)
            } else {
                format!("{} ({}).{}", stem, counter, extension)
            };
            
            path = download_dir.join(new_filename);
            counter += 1;
            
            // Safety limit to prevent infinite loop
            if counter > 1000 {
                break;
            }
        }
        
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sanitize_filename() {
        // Path traversal is removed by taking only the filename
        assert_eq!(
            DownloadValidator::sanitize_filename("../../../etc/passwd"),
            "passwd"
        );
        
        assert_eq!(
            DownloadValidator::sanitize_filename("test<>file.txt"),
            "testfile.txt"
        );
        
        assert_eq!(
            DownloadValidator::sanitize_filename("normal_file-123.pdf"),
            "normal_file-123.pdf"
        );
    }
    
    #[test]
    fn test_validate_extension() {
        assert_eq!(
            DownloadValidator::validate_extension("document.pdf"),
            ValidationResult::Safe
        );
        
        match DownloadValidator::validate_extension("virus.exe") {
            ValidationResult::RequiresConfirmation(_) => {}
            _ => panic!("Expected RequiresConfirmation for .exe"),
        }
    }
    
    #[test]
    fn test_validate_mime_type() {
        assert_eq!(
            DownloadValidator::validate_mime_type("application/pdf"),
            ValidationResult::Safe
        );
        
        match DownloadValidator::validate_mime_type("application/x-msdownload") {
            ValidationResult::RequiresConfirmation(_) => {}
            _ => panic!("Expected RequiresConfirmation for executable MIME"),
        }
    }
    
    #[test]
    fn test_validate_url() {
        assert!(DownloadValidator::validate_url("https://example.com/file.pdf").is_ok());
        assert!(DownloadValidator::validate_url("http://example.com/file.pdf").is_ok());
        assert!(DownloadValidator::validate_url("ftp://example.com/file.pdf").is_err());
        assert!(DownloadValidator::validate_url("https://localhost/file.pdf").is_err());
        assert!(DownloadValidator::validate_url("https://192.168.1.1/file.pdf").is_err());
    }
}
