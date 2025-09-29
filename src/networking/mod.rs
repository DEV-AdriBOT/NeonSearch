pub mod http_client;
pub mod url_parser;
pub mod cookie_manager;
pub mod manual_client;
pub mod image_loader;
pub mod performance;
pub mod temp_storage;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use temp_storage::{TempStorageManager, TempFile};

// Maximum content size to prevent memory exhaustion (50MB)
const MAX_CONTENT_SIZE: usize = 50 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    // Content storage - either in memory or temporary file
    pub body: Vec<u8>,  // Keep for small content/backward compatibility
    pub temp_file: Option<TempFile>,  // Use for large content
    // Cache for decompressed content to prevent re-processing
    cached_string: Arc<Mutex<Option<String>>>,
}

impl HttpRequest {
    pub fn new_get(url: String) -> Self {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), 
                      "NeonSearch/0.1.0 (Custom Browser by NeonDev)".to_string());
        headers.insert("Accept".to_string(), 
                      "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string());
        headers.insert("Accept-Language".to_string(), "en-US,en;q=0.5".to_string());
        headers.insert("Accept-Encoding".to_string(), "gzip, deflate".to_string());
        headers.insert("Connection".to_string(), "keep-alive".to_string());
        
        Self {
            method: "GET".to_string(),
            url,
            headers,
            body: None,
        }
    }
    
    pub fn new_post(url: String, body: Vec<u8>) -> Self {
        let mut request = Self::new_get(url);
        request.method = "POST".to_string();
        let body_len = body.len();
        request.body = Some(body);
        request.headers.insert("Content-Length".to_string(), body_len.to_string());
        request
    }
}

impl HttpResponse {
    pub fn new(status_code: u16, status_text: String, headers: HashMap<String, String>, body: Vec<u8>) -> Self {
        Self {
            status_code,
            status_text,
            headers,
            body,
            temp_file: None,
            cached_string: Arc::new(Mutex::new(None)),
        }
    }

    pub fn new_with_temp_file(status_code: u16, status_text: String, headers: HashMap<String, String>, temp_file: TempFile) -> Self {
        Self {
            status_code,
            status_text,
            headers,
            body: Vec::new(),  // Empty body when using temp file
            temp_file: Some(temp_file),
            cached_string: Arc::new(Mutex::new(None)),
        }
    }

    pub fn body_as_string(&self) -> Result<String> {
        // Check cache first to prevent re-decompression
        if let Ok(cache) = self.cached_string.lock() {
            if let Some(cached) = cache.as_ref() {
                return Ok(cached.clone());
            }
        }
        
        let result = self.decompress_body_internal();
        
        // Cache the result
        if let Ok(ref content) = result {
            if let Ok(mut cache) = self.cached_string.lock() {
                *cache = Some(content.clone());
            }
        }
        
        result
    }
    
    fn decompress_body_internal(&self) -> Result<String> {
        // Get content from either memory or temporary file
        let mut data = if let Some(ref temp_file) = self.temp_file {
            // Read content from temporary file
            let temp_manager = TempStorageManager::new()?;
            temp_manager.read_content(temp_file)?
        } else {
            // Use in-memory content
            if self.body.len() > MAX_CONTENT_SIZE {
                return Err(anyhow::anyhow!(
                    "Content too large: {} bytes (max: {} bytes). Use streaming for large sites.",
                    self.body.len(),
                    MAX_CONTENT_SIZE
                ));
            }
            self.body.clone()
        };
        
        if let Some(encoding) = self.get_header("Content-Encoding").or_else(|| self.get_header("content-encoding")) {
            let encodings: Vec<String> = encoding.split(',').map(|e| e.trim().to_lowercase()).collect();
            
            // Handle multiple encodings in reverse order (last applied first)
            for enc in encodings.iter().rev() {
                data = match enc.as_str() {
                    "gzip" | "x-gzip" => {
                        println!("Decompressing gzip content ({} bytes)", data.len());
                        match decode_gzip(&data) {
                            Ok(decompressed) => {
                                if decompressed.len() > MAX_CONTENT_SIZE {
                                    println!("Decompressed content too large ({} bytes), truncating", decompressed.len());
                                    decompressed[..MAX_CONTENT_SIZE].to_vec()
                                } else {
                                    decompressed
                                }
                            },
                            Err(_) => {
                                println!("Gzip decompression failed, using original data");
                                data
                            }
                        }
                    },
                    "deflate" => {
                        println!("Decompressing deflate content ({} bytes)", data.len());
                        match decode_deflate(&data) {
                            Ok(decompressed) => {
                                if decompressed.len() > MAX_CONTENT_SIZE {
                                    println!("Decompressed content too large ({} bytes), truncating", decompressed.len());
                                    decompressed[..MAX_CONTENT_SIZE].to_vec()
                                } else {
                                    decompressed
                                }
                            },
                            Err(_) => {
                                println!("Deflate decompression failed, using original data");
                                data
                            }
                        }
                    },
                    "br" | "brotli" => {
                        println!("Decompressing brotli content ({} bytes)", data.len());
                        match decode_brotli(&data) {
                            Ok(decompressed) => {
                                if decompressed.len() > MAX_CONTENT_SIZE {
                                    println!("Decompressed content too large ({} bytes), truncating", decompressed.len());
                                    decompressed[..MAX_CONTENT_SIZE].to_vec()
                                } else {
                                    decompressed
                                }
                            },
                            Err(_) => {
                                println!("Brotli decompression failed, using original data");
                                data
                            }
                        }
                    },
                    "zstd" => {
                        println!("Attempting to decompress zstd content ({} bytes)", data.len());
                        match decode_zstd(&data) {
                            Ok(decompressed) => {
                                if decompressed.len() > MAX_CONTENT_SIZE {
                                    println!("Decompressed content too large ({} bytes), truncating", decompressed.len());
                                    decompressed[..MAX_CONTENT_SIZE].to_vec()
                                } else {
                                    decompressed
                                }
                            },
                            Err(_) => {
                                println!("ZSTD decompression failed, using original data");
                                data
                            }
                        }
                    },
                    "identity" | "" => {
                        data // No compression
                    },
                    _ => {
                        println!("Unknown content encoding: {}", enc);
                        data // Unknown encoding, use as-is
                    }
                };
            }
        }
        
        // Convert to string with better error handling
        match String::from_utf8(data.clone()) {
            Ok(text) => Ok(text),
            Err(_) => {
                // Try as latin-1 if UTF-8 fails
                let text = data.iter().map(|&b| b as char).collect();
                println!("Content converted from bytes to latin-1 (non-UTF8)");
                Ok(text)
            }
        }
    }
    
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }
    
    pub fn is_redirect(&self) -> bool {
        self.status_code >= 300 && self.status_code < 400
    }
    
    /// Get a partial preview of the content for very large sites
    pub fn body_as_preview(&self, max_chars: usize) -> Result<String> {
        let full_content = self.body_as_string()?;
        if full_content.len() <= max_chars {
            Ok(full_content)
        } else {
            let preview = full_content.chars().take(max_chars).collect::<String>();
            Ok(format!("{}...\n\n[Content truncated - {} of {} characters shown]", 
                      preview, max_chars, full_content.len()))
        }
    }
    
    /// Check if this response represents very large content
    pub fn is_large_content(&self) -> bool {
        self.body.len() > MAX_CONTENT_SIZE / 2 // Half of max size threshold
    }
    
    /// Get raw body content as bytes (from memory or temp file)
    pub fn get_raw_body(&self) -> Result<Vec<u8>> {
        if let Some(ref temp_file) = self.temp_file {
            let temp_manager = TempStorageManager::new()?;
            temp_manager.read_content(temp_file)
        } else {
            Ok(self.body.clone())
        }
    }

    /// Check if response uses temporary file storage
    pub fn uses_temp_file(&self) -> bool {
        self.temp_file.is_some()
    }

    /// Get temporary file info if using file storage
    pub fn get_temp_file(&self) -> Option<&TempFile> {
        self.temp_file.as_ref()
    }

    /// Cleanup temporary file if present
    pub fn cleanup_temp_file(&self) -> Result<()> {
        if let Some(ref temp_file) = self.temp_file {
            let temp_manager = TempStorageManager::new()?;
            temp_manager.remove_file(temp_file)?;
        }
        Ok(())
    }

    pub fn get_header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }
    
    pub fn content_type(&self) -> Option<&String> {
        self.get_header("content-type")
            .or_else(|| self.get_header("Content-Type"))
    }
}

fn decode_gzip(input: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::GzDecoder;
    use std::io::Read;
    
    if input.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut decoder = GzDecoder::new(input);
    let mut output = Vec::new();
    
    match decoder.read_to_end(&mut output) {
        Ok(_) => Ok(output),
        Err(e) => {
            println!("Gzip decompression failed: {}", e);
            // Return original data if decompression fails
            Ok(input.to_vec())
        }
    }
}

fn decode_deflate(input: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::DeflateDecoder;
    use std::io::Read;
    
    if input.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut decoder = DeflateDecoder::new(input);
    let mut output = Vec::new();
    
    match decoder.read_to_end(&mut output) {
        Ok(_) => Ok(output),
        Err(e) => {
            println!("Deflate decompression failed: {}", e);
            // Try raw deflate instead of zlib deflate
            use flate2::read::DeflateDecoder;
            let mut raw_decoder = DeflateDecoder::new(input);
            let mut raw_output = Vec::new();
            match raw_decoder.read_to_end(&mut raw_output) {
                Ok(_) => Ok(raw_output),
                Err(_) => Ok(input.to_vec()) // Return original on failure
            }
        }
    }
}

fn decode_brotli(input: &[u8]) -> Result<Vec<u8>> {
    use std::io::Read;
    
    if input.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut decoder = brotli::Decompressor::new(input, 8192);
    let mut output = Vec::new();
    
    match decoder.read_to_end(&mut output) {
        Ok(_) => {
            println!("Brotli decompressed: {} -> {} bytes", input.len(), output.len());
            Ok(output)
        },
        Err(e) => {
            println!("Brotli decompression failed: {}", e);
            Ok(input.to_vec()) // Return original data on failure
        }
    }
}

fn decode_zstd(input: &[u8]) -> Result<Vec<u8>> {
    if input.is_empty() {
        return Ok(Vec::new());
    }
    
    // For now, return original data since zstd is less common
    // In a full implementation, we'd add zstd dependency and decode here
    println!("ZSTD compression detected but not supported yet");
    Ok(input.to_vec())
}

fn decode_identity(input: &[u8]) -> Result<Vec<u8>> {
    Ok(input.to_vec())
}

pub async fn fetch(url: &str) -> Result<HttpResponse> {
    // Simple wrapper retained for backward compatibility; cookie/header logic now lives in UI fetch_url
    let request = HttpRequest::new_get(url.to_string());
    http_client::send_request(request).await
}