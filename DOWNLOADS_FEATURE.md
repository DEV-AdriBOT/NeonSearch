# NeonSearch Download Manager

## Overview

The NeonSearch browser includes a fully functional download manager with real-time progress tracking, resume capability, and comprehensive security features.

## Features

### Core Functionality

- **Concurrent Downloads**: Download up to 3 files simultaneously
- **Resume Support**: Interrupted downloads can be resumed from where they stopped
- **Real-time Progress**: See download speed (MB/s, KB/s, B/s), percentage, and estimated time remaining
- **Persistent History**: All downloads are saved to a SQLite database
- **File Integrity**: SHA-256 checksums verify completed downloads
- **Smart Retry**: Failed downloads automatically retry with exponential backoff

### Security

- **URL Validation**: Prevents downloads from localhost/private networks (SSRF protection)
- **Filename Sanitization**: Removes path traversal attacks and dangerous characters
- **Extension Validation**: Warns about executable and uncommon file types
- **MIME Type Validation**: Verifies file types match expectations
- **Disk Space Check**: Ensures sufficient space before starting (Unix platforms)
- **Safe Path Generation**: Automatically handles duplicate filenames

### User Interface

- **Downloads Page**: Access via `neon://downloads` in the address bar
- **Progress Indicators**: Visual progress bars with detailed statistics
- **Control Buttons**: Pause, Resume, Cancel, Retry operations
- **File Operations**: 
  - Open downloaded file (cross-platform)
  - Open containing folder (cross-platform)
- **Search & Filter**: Find downloads by filename or URL
- **Status Filtering**: View only active downloads

## Usage

### Accessing the Downloads Page

1. Navigate to `neon://downloads` in the address bar
2. Or use the browser's menu to access Downloads

### Downloading a File

The download manager will be triggered automatically when you click on download links in web pages. Downloads will:
1. Be validated for security
2. Start automatically (or require confirmation for executable files)
3. Appear in the downloads page with real-time progress
4. Be saved to your default downloads folder

### Managing Downloads

#### Active Downloads
- **Pause**: Click the "⏸ Pause" button to temporarily stop a download
- **Cancel**: Click the "✗ Cancel" button to stop and remove the download
- **Progress**: Watch real-time speed and ETA indicators

#### Paused Downloads
- **Resume**: Click the "▶ Resume" button to continue the download
- **Cancel**: Click the "✗ Cancel" button to permanently cancel

#### Failed Downloads
- **Retry**: Click the "↻ Retry" button to attempt the download again
- **Remove**: Click the "🗑 Remove" button to delete the record

#### Completed Downloads
- **Open File**: Click the "▶ Open File" button to open with the default application
- **Open Folder**: Click the "📂 Open Folder" button to view in file manager

### Search and Filter

- **Search Box**: Type to search by filename or URL
- **Show Only Active**: Toggle to display only in-progress and paused downloads
- **Clear Completed**: Remove all completed downloads from the list

## Architecture

### Components

```
src/
├── storage/
│   ├── downloads_db.rs       # SQLite database for download history
│   └── mod.rs
├── engine/
│   ├── download_manager.rs   # Core download engine
│   └── mod.rs
├── security/
│   ├── download_validator.rs # Security validation
│   └── mod.rs
└── pages/
    └── pages/
        └── downloads.rs       # UI for downloads page
```

### Data Flow

1. **Download Request** → Validation → Database Record Creation
2. **Download Task** → Semaphore Acquisition → HTTP Request
3. **Data Stream** → Chunk Writing → Progress Updates
4. **Completion** → Checksum Calculation → Database Update
5. **UI Polling** → Event Processing → Display Update

### Database Schema

```sql
CREATE TABLE downloads (
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
);
```

## Configuration

### Download Directory

By default, downloads are saved to:
- **Windows**: `%USERPROFILE%\Downloads`
- **macOS**: `~/Downloads`
- **Linux**: `~/Downloads`

The download directory can be configured in the browser settings.

### Concurrent Downloads

The maximum number of simultaneous downloads is set to 3 by default. This is controlled by a semaphore in the `DownloadManager`.

### Database Location

The downloads database is stored at:
```
<data_dir>/NeonSearch/downloads.db
```

Where `<data_dir>` is:
- **Windows**: `%APPDATA%`
- **macOS**: `~/Library/Application Support`
- **Linux**: `~/.local/share`

## API Usage (For Developers)

### Starting a Download

```rust
use std::path::PathBuf;
use crate::engine::download_manager::DownloadManager;

// Create download manager
let db_path = PathBuf::from("path/to/downloads.db");
let manager = DownloadManager::new(&db_path)?;

// Start a download
let download_id = manager.start_download(
    "https://example.com/file.pdf".to_string(),
    PathBuf::from("/Users/Downloads/file.pdf")
).await?;

println!("Download started with ID: {}", download_id);
```

### Monitoring Progress

```rust
// Poll for events
let events = manager.poll_events();
for event in events {
    match event {
        DownloadEvent::Progress(progress) => {
            println!("Download {} is {}% complete at {} bytes/s", 
                progress.id, 
                progress.progress_percent,
                progress.speed_bps
            );
        }
        DownloadEvent::Completed(id, path) => {
            println!("Download {} completed: {:?}", id, path);
        }
        DownloadEvent::Failed(id, error) => {
            println!("Download {} failed: {}", id, error);
        }
        _ => {}
    }
}
```

### Controlling Downloads

```rust
// Pause a download
manager.pause_download("download-id").await?;

// Resume a download
manager.resume_download("download-id").await?;

// Cancel a download
manager.cancel_download("download-id").await?;
```

### Querying Download History

```rust
// Get all downloads
let all_downloads = manager.get_download_history()?;

// Get downloads by status
let active_downloads = manager.get_downloads_by_status(DownloadState::InProgress)?;

// Search downloads
let results = manager.search_downloads("search query")?;
```

## Security Best Practices

### For Users

1. **Be cautious with executable files**: The browser will warn you when downloading .exe, .bat, .sh, and other executable formats
2. **Verify file sources**: Only download files from trusted websites
3. **Check file sizes**: Ensure the downloaded file size matches expectations
4. **Scan downloads**: Use antivirus software to scan downloaded files before opening

### For Developers

1. **Always validate URLs**: Use `DownloadValidator::validate_url()` before starting downloads
2. **Sanitize filenames**: Use `DownloadValidator::validate_filename()` to prevent path traversal
3. **Check disk space**: Use `DownloadValidator::check_disk_space()` before large downloads
4. **Validate file types**: Use `DownloadValidator::validate_extension()` and `validate_mime_type()`
5. **Verify checksums**: Use the SHA-256 checksum to ensure file integrity

## Testing

### Unit Tests

```bash
# Test storage layer
cargo test storage::downloads_db::tests

# Test download manager
cargo test engine::download_manager::tests

# Test security validator
cargo test security::download_validator::tests
```

### Integration Testing

1. Build the browser: `cargo build --release`
2. Run the browser: `./target/release/neonsearch`
3. Navigate to `neon://downloads`
4. Test various download scenarios:
   - Small file downloads
   - Large file downloads
   - Pause and resume
   - Failed downloads (disconnect network)
   - Concurrent downloads

## Troubleshooting

### Downloads Not Starting

1. Check the console for error messages
2. Verify the URL is valid and accessible
3. Ensure sufficient disk space is available
4. Check file permissions in the download directory

### Downloads Failing

1. Check network connectivity
2. Verify the remote server is responding
3. Check if the download is being blocked by security validation
4. Review error messages in the downloads page

### Resume Not Working

1. Ensure the server supports HTTP Range requests
2. Check if the partial file still exists
3. Verify the download wasn't cancelled (cancelled downloads can't be resumed)

### Database Issues

If the downloads database becomes corrupted:
1. Close the browser
2. Delete the database file at `<data_dir>/NeonSearch/downloads.db`
3. Restart the browser (a new database will be created)

## Performance Considerations

- **Memory Usage**: The download manager streams files directly to disk, so memory usage stays constant regardless of file size
- **Concurrent Downloads**: Limited to 3 simultaneous downloads to prevent overwhelming the network or disk I/O
- **Chunk Size**: Downloads are processed in 64KB chunks for optimal performance
- **Database Size**: Old completed downloads can be cleaned up using `cleanup_old_downloads(days)`

## Future Enhancements

Potential features for future development:

- [ ] Desktop notifications when downloads complete
- [ ] Context menu integration in web pages
- [ ] Drag-and-drop URL support
- [ ] Download speed graphs
- [ ] Bandwidth throttling controls in UI
- [ ] Download scheduling
- [ ] Category-based organization
- [ ] Cloud backup integration
- [ ] Automatic virus scanning
- [ ] Browser restart recovery for in-progress downloads

## Contributing

When contributing to the download manager:

1. Follow existing code patterns
2. Add tests for new functionality
3. Update this documentation
4. Ensure cross-platform compatibility
5. Test security implications

## License

Part of the NeonSearch browser project. See the main LICENSE file for details.
