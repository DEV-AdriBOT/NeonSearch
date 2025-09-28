use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use anyhow::Result;
use crate::networking::{HttpResponse, manual_client::ManualHttpClient};

// Response cache entry
#[derive(Clone)]
struct CacheEntry {
    response: HttpResponse,
    created_at: Instant,
    expires_at: Option<Instant>,
    etag: Option<String>,
    last_modified: Option<String>,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            Instant::now() > expires
        } else {
            // Default TTL of 5 minutes for responses without explicit caching headers
            Instant::now().duration_since(self.created_at) > Duration::from_secs(300)
        }
    }

    fn is_stale(&self) -> bool {
        // Consider entry stale after 1 hour regardless of cache headers
        Instant::now().duration_since(self.created_at) > Duration::from_secs(3600)
    }
}

// DNS cache entry
#[derive(Clone)]
struct DnsEntry {
    addresses: Vec<std::net::SocketAddr>,
    cached_at: Instant,
}

impl DnsEntry {
    fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.cached_at) > Duration::from_secs(300) // 5 minutes
    }
}

// Performance-optimized HTTP client with caching
pub struct CachingHttpClient {
    manual_client: ManualHttpClient,
    response_cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    dns_cache: Arc<RwLock<HashMap<String, DnsEntry>>>,
    max_cache_size: usize,
    enable_caching: bool,
}

impl CachingHttpClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            manual_client: ManualHttpClient::new()?,
            response_cache: Arc::new(RwLock::new(HashMap::new())),
            dns_cache: Arc::new(RwLock::new(HashMap::new())),
            max_cache_size: 100, // Maximum 100 cached responses
            enable_caching: true,
        })
    }

    pub async fn fetch(&self, url: &str) -> Result<HttpResponse> {
        let cache_key = self.generate_cache_key(url);
        
        // Check cache first if caching is enabled
        if self.enable_caching {
            if let Some(cached) = self.get_cached_response(&cache_key).await {
                if !cached.is_expired() {
                    println!("🚀 Cache hit for {}", url);
                    return Ok(cached.response);
                } else if !cached.is_stale() {
                    // Entry is expired but not stale, try conditional request
                    if let Ok(response) = self.conditional_fetch(url, &cached).await {
                        return Ok(response);
                    }
                }
            }
        }

        // Cache miss or error - fetch normally
        println!("🌐 Cache miss for {} - fetching", url);
        let fetch_result = self.manual_client.fetch(url).await?;
        let response = fetch_result.response;
        
        // Cache the response if caching is enabled
        if self.enable_caching && response.is_success() {
            self.cache_response(cache_key, &response).await;
        }

        Ok(response)
    }

    async fn conditional_fetch(&self, url: &str, cached_entry: &CacheEntry) -> Result<HttpResponse> {
        // Try to use ETag or Last-Modified for conditional requests
        // For now, just refetch - in a full implementation we'd send If-None-Match/If-Modified-Since headers
        println!("🔄 Conditional fetch for {}", url);
        let fetch_result = self.manual_client.fetch(url).await?;
        Ok(fetch_result.response)
    }

    async fn get_cached_response(&self, cache_key: &str) -> Option<CacheEntry> {
        let cache = self.response_cache.read().await;
        cache.get(cache_key).cloned()
    }

    async fn cache_response(&self, cache_key: String, response: &HttpResponse) {
        let mut cache = self.response_cache.write().await;
        
        // Implement LRU eviction if cache is full
        if cache.len() >= self.max_cache_size {
            // Remove oldest entries
            let entries: Vec<_> = cache.iter().map(|(k, v)| (k.clone(), v.created_at)).collect();
            let mut sorted_entries = entries;
            sorted_entries.sort_by(|a, b| a.1.cmp(&b.1));
            
            // Remove oldest 20% of entries
            let to_remove = self.max_cache_size / 5;
            for (key, _) in sorted_entries.into_iter().take(to_remove) {
                cache.remove(&key);
            }
        }

        // Parse cache control headers
        let (expires_at, etag, last_modified) = self.parse_cache_headers(response);
        
        let entry = CacheEntry {
            response: response.clone(),
            created_at: Instant::now(),
            expires_at,
            etag,
            last_modified,
        };

        cache.insert(cache_key, entry);
    }

    fn parse_cache_headers(&self, response: &HttpResponse) -> (Option<Instant>, Option<String>, Option<String>) {
        let mut expires_at = None;
        let mut etag = None;
        let mut last_modified = None;

        // Look for cache-related headers (case-insensitive)
        for (key, value) in &response.headers {
            let key_lower = key.to_lowercase();
            match key_lower.as_str() {
                "cache-control" => {
                    if let Some(max_age) = self.extract_max_age(value) {
                        expires_at = Some(Instant::now() + Duration::from_secs(max_age));
                    } else if value.contains("no-cache") || value.contains("no-store") {
                        // Don't cache responses with no-cache/no-store
                        return (Some(Instant::now()), None, None);
                    }
                },
                "expires" => {
                    // Parse HTTP date format - for now just use a default TTL
                    if expires_at.is_none() {
                        expires_at = Some(Instant::now() + Duration::from_secs(300));
                    }
                },
                "etag" => {
                    etag = Some(value.clone());
                },
                "last-modified" => {
                    last_modified = Some(value.clone());
                },
                _ => {}
            }
        }

        (expires_at, etag, last_modified)
    }

    fn extract_max_age(&self, cache_control: &str) -> Option<u64> {
        // Simple max-age extraction
        for directive in cache_control.split(',') {
            let directive = directive.trim();
            if directive.starts_with("max-age=") {
                if let Ok(age) = directive[8..].parse::<u64>() {
                    return Some(age);
                }
            }
        }
        None
    }

    fn generate_cache_key(&self, url: &str) -> String {
        // Simple cache key - in production might want to normalize URLs
        url.to_string()
    }

    pub async fn clear_cache(&self) {
        let mut response_cache = self.response_cache.write().await;
        let mut dns_cache = self.dns_cache.write().await;
        response_cache.clear();
        dns_cache.clear();
        println!("🧹 Cleared all caches");
    }

    pub async fn get_cache_stats(&self) -> CacheStats {
        let response_cache = self.response_cache.read().await;
        let dns_cache = self.dns_cache.read().await;
        
        let mut expired_responses = 0;
        for entry in response_cache.values() {
            if entry.is_expired() {
                expired_responses += 1;
            }
        }

        let mut expired_dns = 0;
        for entry in dns_cache.values() {
            if entry.is_expired() {
                expired_dns += 1;
            }
        }

        CacheStats {
            response_entries: response_cache.len(),
            dns_entries: dns_cache.len(),
            expired_responses,
            expired_dns,
            cache_size_limit: self.max_cache_size,
        }
    }

    pub fn set_caching_enabled(&mut self, enabled: bool) {
        self.enable_caching = enabled;
        println!("🔧 HTTP caching {}", if enabled { "enabled" } else { "disabled" });
    }

    pub async fn preload_resources(&self, urls: Vec<String>) {
        println!("📦 Preloading {} resources in background", urls.len());
        
        let tasks: Vec<_> = urls.into_iter().map(|url| {
            let client = self.clone();
            tokio::spawn(async move {
                if let Err(e) = client.fetch(&url).await {
                    eprintln!("Preload failed for {}: {}", url, e);
                } else {
                    println!("✅ Preloaded {}", url);
                }
            })
        }).collect();

        // Wait for all preload tasks to complete (with timeout)
        let _ = tokio::time::timeout(Duration::from_secs(30), async {
            for task in tasks {
                let _ = task.await;
            }
        }).await;
    }
}

impl Clone for CachingHttpClient {
    fn clone(&self) -> Self {
        Self {
            manual_client: self.manual_client.clone(),
            response_cache: Arc::clone(&self.response_cache),
            dns_cache: Arc::clone(&self.dns_cache),
            max_cache_size: self.max_cache_size,
            enable_caching: self.enable_caching,
        }
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub response_entries: usize,
    pub dns_entries: usize,
    pub expired_responses: usize,
    pub expired_dns: usize,
    pub cache_size_limit: usize,
}

// Memory management utilities
pub struct MemoryManager {
    max_memory_mb: usize,
}

impl MemoryManager {
    pub fn new(max_memory_mb: usize) -> Self {
        Self { max_memory_mb }
    }

    pub fn check_memory_usage(&self) -> MemoryStats {
        // In a real implementation, we'd use system calls to get actual memory usage
        // For now, return mock data
        MemoryStats {
            used_mb: 45, // Mock value
            max_mb: self.max_memory_mb,
            percentage: 45.0 / self.max_memory_mb as f64 * 100.0,
        }
    }

    pub fn should_trigger_cleanup(&self) -> bool {
        let stats = self.check_memory_usage();
        stats.percentage > 80.0 // Trigger cleanup at 80% memory usage
    }

    pub async fn cleanup_if_needed(&self, client: &CachingHttpClient) {
        if self.should_trigger_cleanup() {
            println!("🧹 Memory usage high, triggering cleanup");
            client.clear_cache().await;
            
            // Additional cleanup could include:
            // - Clearing old tab content
            // - Reducing image cache size
            // - Garbage collecting unused resources
        }
    }
}

#[derive(Debug)]
pub struct MemoryStats {
    pub used_mb: usize,
    pub max_mb: usize,
    pub percentage: f64,
}

// Background task manager for performance optimizations
pub struct BackgroundTaskManager {
    active_tasks: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

impl BackgroundTaskManager {
    pub fn new() -> Self {
        Self {
            active_tasks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn spawn_cache_cleanup(&self, client: CachingHttpClient) {
        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes
            loop {
                interval.tick().await;
                
                // Clean up expired cache entries
                let stats = client.get_cache_stats().await;
                if stats.expired_responses > 10 || stats.expired_dns > 10 {
                    println!("🧹 Background cleanup: removing expired cache entries");
                    // In a real implementation, we'd remove only expired entries
                    // For now, just report the stats
                    println!("Cache stats: {:?}", stats);
                }
            }
        });

        let mut tasks = self.active_tasks.write().await;
        tasks.push(task);
    }

    pub async fn spawn_memory_monitor(&self, memory_manager: MemoryManager, client: CachingHttpClient) {
        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Every minute
            loop {
                interval.tick().await;
                memory_manager.cleanup_if_needed(&client).await;
            }
        });

        let mut tasks = self.active_tasks.write().await;
        tasks.push(task);
    }

    pub async fn shutdown(&self) {
        let mut tasks = self.active_tasks.write().await;
        for task in tasks.drain(..) {
            task.abort();
        }
        println!("🛑 Background tasks shut down");
    }
}