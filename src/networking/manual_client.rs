use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::TlsConnector;
use rustls::ClientConfig;
use webpki_roots;
use crate::networking::{HttpResponse, temp_storage::TempStorageManager};

#[derive(Debug, Clone, Copy)]
pub enum FetchPhase {
    Resolving,
    Connecting,
    TlsHandshake,
    SendingRequest,
    ReadingHeaders,
    ReadingBody,
    Redirecting,
    Completed,
}

#[derive(Debug)]
pub struct ManualFetchResult {
    pub response: HttpResponse,
    pub phases: Vec<FetchPhase>,
    pub redirects: Vec<String>,
}

#[derive(Clone)]
pub struct ManualHttpClient {
    tls_config: Arc<ClientConfig>,
    timeout: Duration,
    max_redirects: usize,
    max_body_size: usize,
}

// Threshold for when to use temporary file storage instead of memory (5MB)
const TEMP_FILE_THRESHOLD: usize = 5 * 1024 * 1024;

impl ManualHttpClient {
    pub fn new() -> Result<Self> {
        // Install default crypto provider for rustls
        let _ = rustls::crypto::ring::default_provider().install_default();
        
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        
        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
            
        Ok(Self {
            tls_config: Arc::new(config),
            timeout: Duration::from_secs(30),
            max_redirects: 10,
            max_body_size: 50 * 1024 * 1024, // 50MB for better big site compatibility
        })
    }

    pub async fn fetch(&self, url: &str) -> Result<ManualFetchResult> {
        let mut current_url = url.to_string();
        let mut redirects = Vec::new();
        let mut phases = Vec::new();

        // Handle common URL corrections
        if !current_url.starts_with("http://") && 
           !current_url.starts_with("https://") && 
           !current_url.starts_with("neon://") && 
           !current_url.starts_with("about:") {
            current_url = format!("https://{}", current_url);
        }

        for redirect_count in 0..=self.max_redirects {
            phases.push(FetchPhase::Resolving);
            
            let parsed = reqwest::Url::parse(&current_url)
                .map_err(|e| anyhow!("Invalid URL '{}': {}", current_url, e))?;
            
            let host = parsed.host_str()
                .ok_or_else(|| anyhow!("Missing host in URL: {}", current_url))?
                .to_string();
            
            let port = parsed.port_or_known_default()
                .ok_or_else(|| anyhow!("Cannot determine port for URL: {}", current_url))?;
            
            let is_https = parsed.scheme() == "https";
            
            let path_and_query = {
                let mut pq = parsed.path().to_string();
                if let Some(q) = parsed.query() { 
                    pq.push('?'); 
                    pq.push_str(q); 
                }
                if pq.is_empty() || pq == "/" { 
                    pq = "/".to_string(); 
                }
                pq
            };

            phases.push(FetchPhase::Connecting);
            
            // Enhanced DNS resolution with multiple attempts
            let addr_iter = match tokio::time::timeout(
                Duration::from_secs(10), 
                tokio::net::lookup_host((host.as_str(), port))
            ).await {
                Ok(Ok(iter)) => iter,
                Ok(Err(e)) => return Err(anyhow!("DNS resolution failed for {}: {}", host, e)),
                Err(_) => return Err(anyhow!("DNS resolution timeout for {}", host)),
            };

            let mut last_err = None;
            let mut stream_opt = None;
            
            // Try connecting to multiple resolved addresses
            for addr in addr_iter {
                match tokio::time::timeout(self.timeout, TcpStream::connect(addr)).await {
                    Ok(Ok(s)) => { 
                        stream_opt = Some(s); 
                        break; 
                    },
                    Ok(Err(e)) => { 
                        last_err = Some(e); 
                        continue; 
                    },
                    Err(_) => continue, // Try next address on timeout
                }
            }
            
            let stream = stream_opt.ok_or_else(|| {
                anyhow!("Failed to connect to {}: {:?}", host, last_err)
            })?;

            // Attempt the actual HTTP request
            match self.fetch_single_round(
                is_https, 
                host, 
                port, 
                path_and_query, 
                stream, 
                redirects.clone(), 
                phases.clone(), 
                current_url.clone()
            ).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.starts_with("REDIRECT:") {
                        let new_url = err_str.strip_prefix("REDIRECT:").unwrap_or(&current_url);
                        redirects.push(current_url.clone());
                        current_url = new_url.to_string();
                        phases.push(FetchPhase::Redirecting);
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        
        Err(anyhow!("Too many redirects ({}), stopped at: {}", self.max_redirects, current_url))
    }

    async fn fetch_single_round(
        &self,
        is_https: bool,
        host: String,
        _port: u16,
        path_and_query: String,
        stream_plain: TcpStream,
        redirects: Vec<String>,
        mut phases: Vec<FetchPhase>,
        original_url: String,
    ) -> Result<ManualFetchResult> {
        enum Conn { 
            Plain(TcpStream), 
            Tls(tokio_rustls::client::TlsStream<TcpStream>) 
        }
        
        let mut conn = if is_https {
            phases.push(FetchPhase::TlsHandshake);
            let connector = TlsConnector::from(self.tls_config.clone());
            
            let domain = rustls::pki_types::ServerName::try_from(host.clone())
                .map_err(|_| anyhow!("Invalid hostname for TLS: {}", host))?;
            
            let tls_stream = tokio::time::timeout(
                Duration::from_secs(20), 
                connector.connect(domain, stream_plain)
            )
            .await
            .map_err(|_| anyhow!("TLS handshake timeout after 20s"))?
            .map_err(|e| {
                let err_str = e.to_string();
                if err_str.contains("close_notify") || err_str.contains("CloseNotify") {
                    anyhow!("TLS session closed by server (close_notify)")
                } else if err_str.contains("certificate") {
                    anyhow!("TLS certificate validation failed: {}", e)
                } else if err_str.contains("protocol") {
                    anyhow!("TLS protocol version mismatch: {}", e)
                } else {
                    anyhow!("TLS handshake failed: {}", e)
                }
            })?;
            Conn::Tls(tls_stream)
        } else {
            Conn::Plain(stream_plain)
        };

        phases.push(FetchPhase::SendingRequest);
        
        // Enhanced HTTP request with comprehensive headers
        let request_headers = format!(
            "GET {} HTTP/1.1\r\n\
            Host: {}\r\n\
            User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) NeonSearch/1.0 Chrome/120.0.0.0 Safari/537.36\r\n\
            Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7\r\n\
            Accept-Language: en-US,en;q=0.9\r\n\
            Accept-Encoding: gzip, deflate, br, zstd\r\n\
            DNT: 1\r\n\
            Connection: close\r\n\
            Upgrade-Insecure-Requests: 1\r\n\
            Sec-Fetch-Dest: document\r\n\
            Sec-Fetch-Mode: navigate\r\n\
            Sec-Fetch-Site: none\r\n\
            Sec-Fetch-User: ?1\r\n\
            Cache-Control: max-age=0\r\n\
            \r\n", 
            path_and_query, host
        );

        match &mut conn {
            Conn::Plain(s) => {
                s.write_all(request_headers.as_bytes()).await
                    .map_err(|e| anyhow!("Failed to send HTTP request: {}", e))?;
            },
            Conn::Tls(s) => {
                s.write_all(request_headers.as_bytes()).await
                    .map_err(|e| anyhow!("Failed to send HTTPS request: {}", e))?;
            },
        }

        phases.push(FetchPhase::ReadingHeaders);
        let mut raw = Vec::new();
        let mut buf = [0u8; 8192]; // Larger buffer for better performance
        let mut header_end = None;

        // Read response headers with timeout
        loop {
            let n = match tokio::time::timeout(Duration::from_secs(10), async {
                match &mut conn {
                    Conn::Plain(s) => s.read(&mut buf).await,
                    Conn::Tls(s) => s.read(&mut buf).await,
                }
            }).await {
                Ok(Ok(n)) => n,
                Ok(Err(e)) => {
                    return Err(anyhow!("Network read error while reading headers: {}", e));
                },
                Err(_) => {
                    return Err(anyhow!("Timeout while reading HTTP response headers"));
                }
            };
            
            if n == 0 { 
                if raw.is_empty() {
                    return Err(anyhow!("Server closed connection without sending response"));
                }
                break; 
            }
            
            raw.extend_from_slice(&buf[..n]);
            
            // Look for end of headers (double CRLF)
            if let Some(pos) = twoway::find_bytes(&raw, b"\r\n\r\n") {
                header_end = Some(pos + 4);
                break;
            }
            
            // Fallback: also check for double LF (some servers use this)
            if header_end.is_none() {
                if let Some(pos) = twoway::find_bytes(&raw, b"\n\n") {
                    header_end = Some(pos + 2);
                    break;
                }
            }
            
            // Prevent memory exhaustion from malformed responses
            if raw.len() > 128 * 1024 { 
                return Err(anyhow!("HTTP response headers too large (>128KB) - possible malformed response"));
            }
        }
        
        let header_end = header_end.ok_or_else(|| {
            anyhow!("Incomplete HTTP response headers received from server")
        })?;
        
        let (header_bytes, body_bytes) = raw.split_at(header_end);
        let header_text = String::from_utf8_lossy(header_bytes);
        
        // Parse status line
        let mut lines = header_text.lines().filter(|l| !l.trim().is_empty());
        let status_line = lines.next().ok_or_else(|| {
            anyhow!("Missing HTTP status line in response")
        })?;
        
        let mut parts = status_line.split_whitespace();
        let http_version = parts.next().unwrap_or("HTTP/1.0");
        let status_code: u16 = parts.next()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| anyhow!("Invalid HTTP status code in response: {}", status_line))?;
        let status_text = parts.collect::<Vec<_>>().join(" ");
        
        // Parse response headers
        let mut headers = HashMap::new();
        for line in lines {
            let line = line.trim();
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase(); // Normalize header names
                let value = value.trim().to_string();
                headers.insert(key, value);
            }
        }
        
        // Log response info for debugging
        println!("HTTP Response: {} {} {} (Content-Length: {:?}, Content-Encoding: {:?})", 
                 http_version, status_code, status_text,
                 headers.get("content-length"),
                 headers.get("content-encoding"));

        phases.push(FetchPhase::ReadingBody);
        
        // Determine body reading strategy
        let transfer_encoding = headers.get("transfer-encoding")
            .cloned()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let content_length = headers.get("content-length")
            .and_then(|v| v.parse::<usize>().ok());

        let mut body = body_bytes.to_vec();
        let read_timeout = Duration::from_secs(15);
        
        if transfer_encoding.contains("chunked") {
            // Handle chunked transfer encoding
            let mut remainder = Vec::new();
            let mut total_read = 0;
            
            loop {
                let n = match tokio::time::timeout(read_timeout, async {
                    match &mut conn {
                        Conn::Plain(s) => s.read(&mut buf).await,
                        Conn::Tls(s) => s.read(&mut buf).await,
                    }
                }).await {
                    Ok(Ok(n)) => n,
                    Ok(Err(e)) => {
                        println!("Read error during chunked transfer: {}", e);
                        break;
                    },
                    Err(_) => {
                        println!("Timeout during chunked transfer after {}KB", total_read / 1024);
                        break;
                    }
                };
                
                if n == 0 { break; }
                
                remainder.extend_from_slice(&buf[..n]);
                total_read += n;
                
                // Safety limits
                if remainder.len() > self.max_body_size {
                    println!("Chunked response truncated at {}MB", remainder.len() / 1024 / 1024);
                    break;
                }
            }
            
            body.extend_from_slice(&remainder);
            body = match decode_chunked(&body) { 
                Ok(decoded) => decoded,
                Err(e) => {
                    println!("Chunked decoding failed: {}, using raw data", e);
                    body
                }
            };
            
        } else if let Some(expected_len) = content_length {
            // Handle Content-Length specified response
            let mut total_read = body.len();
            
            while total_read < expected_len && total_read < self.max_body_size {
                let n = match tokio::time::timeout(read_timeout, async {
                    match &mut conn {
                        Conn::Plain(s) => s.read(&mut buf).await,
                        Conn::Tls(s) => s.read(&mut buf).await,
                    }
                }).await {
                    Ok(Ok(n)) => n,
                    Ok(Err(e)) => {
                        println!("Read error during content-length transfer: {}", e);
                        break;
                    },
                    Err(_) => {
                        println!("Timeout during content-length transfer at {}/{}KB", 
                                total_read / 1024, expected_len / 1024);
                        break;
                    }
                };
                
                if n == 0 { break; }
                
                body.extend_from_slice(&buf[..n]);
                total_read += n;
            }
            
            if total_read >= self.max_body_size {
                println!("Content-length response truncated at {}MB (expected {}MB)", 
                        total_read / 1024 / 1024, expected_len / 1024 / 1024);
            }
            
        } else {
            // Read until connection closes (HTTP/1.0 style or Connection: close)
            let mut total_read = body.len();
            
            loop {
                let n = match tokio::time::timeout(read_timeout, async {
                    match &mut conn {
                        Conn::Plain(s) => s.read(&mut buf).await,
                        Conn::Tls(s) => s.read(&mut buf).await,
                    }
                }).await {
                    Ok(Ok(n)) => n,
                    Ok(Err(_)) => break, // Connection closed or error
                    Err(_) => break, // timeout - assume end of response
                };
                
                if n == 0 { break; }
                
                body.extend_from_slice(&buf[..n]);
                total_read += n;
                
                if total_read > self.max_body_size {
                    println!("Connection-close response truncated at {}MB", total_read / 1024 / 1024);
                    break;
                }
            }
        }
        
        println!("Final response body size: {}KB", body.len() / 1024);

        // Handle redirects
        if (300..400).contains(&status_code) {
            if let Some(location) = headers.get("location") {
                let new_url = if location.starts_with("http://") || location.starts_with("https://") {
                    location.clone()
                } else {
                    // Handle relative redirects
                    let base = reqwest::Url::parse(&original_url)
                        .map_err(|e| anyhow!("Cannot parse base URL for redirect: {}", e))?;
                    base.join(location)
                        .map_err(|e| anyhow!("Cannot resolve redirect URL '{}': {}", location, e))?
                        .to_string()
                };
                
                println!("Redirect {} -> {}", status_code, new_url);
                phases.push(FetchPhase::Redirecting);
                return Err(anyhow!("REDIRECT:{}", new_url));
            } else {
                return Err(anyhow!("Redirect response {} without Location header", status_code));
            }
        }

        phases.push(FetchPhase::Completed);
        
        // Convert lowercase headers back to standard case for compatibility
        let mut final_headers = HashMap::new();
        for (k, v) in headers {
            let standard_key = match k.as_str() {
                "content-type" => "Content-Type".to_string(),
                "content-length" => "Content-Length".to_string(),
                "content-encoding" => "Content-Encoding".to_string(),
                "location" => "Location".to_string(),
                "set-cookie" => "Set-Cookie".to_string(),
                _ => k,
            };
            final_headers.insert(standard_key, v);
        }
        
        // Decide whether to use memory or temporary file storage
        let response = if body.len() > TEMP_FILE_THRESHOLD {
            println!("Large content ({}KB) - using temporary file storage", body.len() / 1024);
            
            // Store content in temporary file
            let temp_manager = TempStorageManager::new()
                .map_err(|e| anyhow!("Failed to create temp storage manager: {}", e))?;
            
            let content_type = final_headers.get("Content-Type").cloned();
            let temp_file = temp_manager.store_content(&body, content_type)
                .map_err(|e| anyhow!("Failed to store content in temp file: {}", e))?;
                
            println!("Content stored in temporary file: {:?}", temp_file.path);
            HttpResponse::new_with_temp_file(status_code, status_text, final_headers, temp_file)
        } else {
            // Use traditional in-memory storage for smaller content
            HttpResponse::new(status_code, status_text, final_headers, body)
        };
        
        Ok(ManualFetchResult { 
            response, 
            phases, 
            redirects 
        })
    }
}

fn decode_chunked(input: &[u8]) -> Result<Vec<u8>> {
    let mut i = 0;
    let mut out = Vec::new();
    let mut chunk_count = 0;
    
    while i < input.len() && chunk_count < 10000 { // Prevent infinite loops
        // Find the end of the chunk size line
        let line_end = if let Some(crlf_pos) = twoway::find_bytes(&input[i..], b"\r\n") {
            i + crlf_pos
        } else if let Some(lf_pos) = memchr::memchr(b'\n', &input[i..]) {
            i + lf_pos
        } else {
            break; // No more complete lines
        };
        
        // Parse chunk size
        let line = &input[i..line_end];
        let line_str = std::str::from_utf8(line).unwrap_or("").trim();
        
        // Move past the line ending
        i = line_end;
        if i < input.len() && input[i] == b'\r' { i += 1; }
        if i < input.len() && input[i] == b'\n' { i += 1; }
        
        if line_str.is_empty() { 
            continue; 
        }
        
        // Parse hexadecimal chunk size, ignoring chunk extensions (after semicolon)
        let size_str = line_str.split(';').next().unwrap_or(line_str).trim();
        let chunk_size = match usize::from_str_radix(size_str, 16) {
            Ok(size) => size,
            Err(_) => {
                println!("Invalid chunk size: '{}'", size_str);
                break;
            }
        };
        
        if chunk_size == 0 {
            // End of chunks
            break;
        }
        
        // Check if we have enough data for this chunk
        if i + chunk_size > input.len() {
            println!("Incomplete chunk: need {} bytes but only {} available", chunk_size, input.len() - i);
            break;
        }
        
        // Extract chunk data
        out.extend_from_slice(&input[i..i + chunk_size]);
        i += chunk_size;
        
        // Skip trailing CRLF after chunk data
        if i < input.len() && input[i] == b'\r' { i += 1; }
        if i < input.len() && input[i] == b'\n' { i += 1; }
        
        chunk_count += 1;
    }
    
    if chunk_count >= 10000 {
        return Err(anyhow!("Too many chunks (possible infinite loop)"));
    }
    
    println!("Decoded {} chunks, total size: {}KB", chunk_count, out.len() / 1024);
    Ok(out)
}
