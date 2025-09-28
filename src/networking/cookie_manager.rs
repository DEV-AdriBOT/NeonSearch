// Cookie management for web requests

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub expires: Option<DateTime<Utc>>,
    pub max_age: Option<i64>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
}

pub struct CookieManager {
    cookies: HashMap<String, Vec<Cookie>>,
}

impl CookieManager {
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }
    
    pub fn add_cookie(&mut self, cookie: Cookie, domain: &str) {
        let domain = domain.to_lowercase();
        self.cookies
            .entry(domain)
            .or_insert_with(Vec::new)
            .push(cookie);
    }
    
    pub fn get_cookies_for_domain(&self, domain: &str) -> Vec<&Cookie> {
        let domain = domain.to_lowercase();
        self.cookies
            .get(&domain)
            .map(|cookies| cookies.iter().collect())
            .unwrap_or_default()
    }
    
    pub fn parse_set_cookie_header(&mut self, header_value: &str, domain: &str) {
        if let Some(cookie) = Self::parse_cookie(header_value) {
            self.add_cookie(cookie, domain);
        }
    }
    
    fn parse_cookie(cookie_str: &str) -> Option<Cookie> {
        let parts: Vec<&str> = cookie_str.split(';').collect();
        if parts.is_empty() {
            return None;
        }
        
        // Parse name=value pair
        let name_value = parts[0].trim();
        let (name, value) = if let Some(eq_pos) = name_value.find('=') {
            (name_value[..eq_pos].trim(), name_value[eq_pos + 1..].trim())
        } else {
            (name_value, "")
        };
        
        let mut cookie = Cookie {
            name: name.to_string(),
            value: value.to_string(),
            domain: None,
            path: None,
            expires: None,
            max_age: None,
            secure: false,
            http_only: false,
            same_site: None,
        };
        
        // Parse attributes
        for part in parts.iter().skip(1) {
            let part = part.trim();
            
            if part.to_lowercase() == "secure" {
                cookie.secure = true;
            } else if part.to_lowercase() == "httponly" {
                cookie.http_only = true;
            } else if let Some(eq_pos) = part.find('=') {
                let attr_name = part[..eq_pos].trim().to_lowercase();
                let attr_value = part[eq_pos + 1..].trim();
                
                match attr_name.as_str() {
                    "domain" => cookie.domain = Some(attr_value.to_string()),
                    "path" => cookie.path = Some(attr_value.to_string()),
                    "expires" => {
                        // TODO: Parse HTTP date format
                    }
                    "max-age" => {
                        if let Ok(max_age) = attr_value.parse::<i64>() {
                            cookie.max_age = Some(max_age);
                        }
                    }
                    "samesite" => cookie.same_site = Some(attr_value.to_string()),
                    _ => {}
                }
            }
        }
        
        Some(cookie)
    }
    
    pub fn get_cookie_header_for_request(&self, domain: &str, path: &str, is_secure: bool) -> Option<String> {
        let cookies = self.get_cookies_for_domain(domain);
        let mut valid_cookies = Vec::new();
        
        for cookie in cookies {
            // Check if cookie is valid for this request
            if cookie.secure && !is_secure {
                continue;
            }
            
            if let Some(cookie_path) = &cookie.path {
                if !path.starts_with(cookie_path) {
                    continue;
                }
            }
            
            // TODO: Check expiration
            
            valid_cookies.push(format!("{}={}", cookie.name, cookie.value));
        }
        
        if valid_cookies.is_empty() {
            None
        } else {
            Some(valid_cookies.join("; "))
        }
    }
    
    pub fn clear_cookies_for_domain(&mut self, domain: &str) {
        let domain = domain.to_lowercase();
        self.cookies.remove(&domain);
    }
    
    pub fn clear_all_cookies(&mut self) {
        self.cookies.clear();
    }
}