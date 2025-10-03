// Security features for the browser

pub mod csp;
pub mod https_validator;
pub mod sandbox;
pub mod download_validator;

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use crate::networking::HttpResponse;

// Enhanced security manager
pub struct SecurityManager {
    hsts_cache: HashMap<String, HstsEntry>,
    csp_policies: HashMap<String, CspPolicy>,
    secure_contexts: HashSet<String>,
    blocked_domains: HashSet<String>,
    trusted_domains: HashSet<String>,
    enforce_https: bool,
}

#[derive(Clone)]
struct HstsEntry {
    max_age: Duration,
    include_subdomains: bool,
    created_at: Instant,
}

impl HstsEntry {
    fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.created_at) > self.max_age
    }
}

#[derive(Clone)]
struct CspPolicy {
    directives: HashMap<String, Vec<String>>,
}

impl SecurityManager {
    pub fn new() -> Self {
        let mut blocked_domains = HashSet::new();
        blocked_domains.insert("malware-example.com".to_string());
        blocked_domains.insert("phishing-test.org".to_string());
        
        Self {
            hsts_cache: HashMap::new(),
            csp_policies: HashMap::new(),
            secure_contexts: HashSet::new(),
            blocked_domains,
            trusted_domains: HashSet::new(),
            enforce_https: false,
        }
    }

    pub fn process_security_headers(&mut self, url: &str, response: &HttpResponse) -> SecurityReport {
        let mut report = SecurityReport::new(url);
        let domain = Self::extract_domain(url);
        
        // Process HSTS header
        if let Some(hsts_value) = response.get_header("Strict-Transport-Security") {
            match self.parse_hsts_header(hsts_value) {
                Ok(entry) => {
                    self.hsts_cache.insert(domain.clone(), entry);
                    report.hsts_enabled = true;
                    report.security_score += 20;
                },
                Err(e) => {
                    report.warnings.push(format!("Invalid HSTS header: {}", e));
                }
            }
        }

        // Check connection security
        if url.starts_with("https://") {
            report.secure_connection = true;
            report.security_score += 30;
            self.secure_contexts.insert(domain);
        } else if url.starts_with("http://") {
            report.warnings.push("Insecure HTTP connection".to_string());
        }

        report
    }

    pub fn should_upgrade_to_https(&self, url: &str) -> bool {
        let domain = Self::extract_domain(url);
        self.hsts_cache.get(&domain).map_or(false, |entry| !entry.is_expired())
    }

    pub fn is_domain_blocked(&self, url: &str) -> bool {
        let domain = Self::extract_domain(url);
        self.blocked_domains.contains(&domain)
    }

    fn parse_hsts_header(&self, header_value: &str) -> Result<HstsEntry> {
        let mut max_age = None;
        let mut include_subdomains = false;

        for directive in header_value.split(';') {
            let directive = directive.trim();
            if directive.starts_with("max-age=") {
                max_age = Some(directive[8..].parse::<u64>()?);
            } else if directive == "includeSubDomains" {
                include_subdomains = true;
            }
        }

        let max_age = max_age.ok_or_else(|| anyhow!("Missing max-age directive"))?;
        
        Ok(HstsEntry {
            max_age: Duration::from_secs(max_age),
            include_subdomains,
            created_at: Instant::now(),
        })
    }

    fn extract_domain(url: &str) -> String {
        if let Ok(parsed) = reqwest::Url::parse(url) {
            parsed.host_str().unwrap_or("").to_string()
        } else {
            String::new()
        }
    }
}

#[derive(Debug)]
pub struct SecurityReport {
    pub url: String,
    pub secure_connection: bool,
    pub hsts_enabled: bool,
    pub csp_enabled: bool,
    pub frame_protection: bool,
    pub content_type_protection: bool,
    pub security_score: u32,
    pub warnings: Vec<String>,
}

impl SecurityReport {
    fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            secure_connection: false,
            hsts_enabled: false,
            csp_enabled: false,
            frame_protection: false,
            content_type_protection: false,
            security_score: 0,
            warnings: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SecurityLevel {
    Poor,
    Fair,
    Good,
    Excellent,
}

impl SecurityManager {
    pub fn is_url_safe(&self, url: &str) -> bool {
        // Basic URL safety checks
        if url.starts_with("javascript:") || url.starts_with("data:") {
            return false;
        }
        
        if let Some(domain) = crate::networking::http_client::get_domain_from_url(url) {
            if self.blocked_domains.contains(&domain) {
                return false;
            }
            
            if self.enforce_https && !url.starts_with("https://") && !url.starts_with("about:") {
                return false;
            }
        }
        
        true
    }
    
    pub fn add_trusted_domain(&mut self, domain: String) {
        self.trusted_domains.insert(domain);
    }
    
    pub fn add_blocked_domain(&mut self, domain: String) {
        self.blocked_domains.insert(domain);
    }
    
    pub fn set_enforce_https(&mut self, enforce: bool) {
        self.enforce_https = enforce;
    }
    
    pub fn validate_certificate(&self, _domain: &str) -> bool {
        // Certificate validation via rustls and webpki-roots
        true
    }
}