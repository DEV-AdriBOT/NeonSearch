// URL parsing utilities

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ParsedUrl {
    pub scheme: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub path: String,
    pub query: HashMap<String, String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    pub fn parse(url: &str) -> Result<Self, String> {
        // Basic URL parsing - in a real implementation, this would be much more robust
        let mut parsed = ParsedUrl {
            scheme: String::new(),
            host: None,
            port: None,
            path: "/".to_string(),
            query: HashMap::new(),
            fragment: None,
        };
        
        let mut url = url.to_string();
        
        // Extract scheme
        if let Some(scheme_end) = url.find("://") {
            parsed.scheme = url[..scheme_end].to_lowercase();
            url = url[scheme_end + 3..].to_string();
        } else if url.starts_with("about:") {
            parsed.scheme = "about".to_string();
            parsed.path = url[6..].to_string();
            return Ok(parsed);
        } else {
            return Err("Invalid URL: missing scheme".to_string());
        }
        
        // Extract fragment
        if let Some(fragment_start) = url.rfind('#') {
            parsed.fragment = Some(url[fragment_start + 1..].to_string());
            url = url[..fragment_start].to_string();
        }
        
        // Extract query
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            for pair in query_string.split('&') {
                if let Some(eq_pos) = pair.find('=') {
                    let key = pair[..eq_pos].to_string();
                    let value = pair[eq_pos + 1..].to_string();
                    parsed.query.insert(key, value);
                } else if !pair.is_empty() {
                    parsed.query.insert(pair.to_string(), String::new());
                }
            }
            url = url[..query_start].to_string();
        }
        
        // Extract path
        if let Some(path_start) = url.find('/') {
            parsed.path = url[path_start..].to_string();
            url = url[..path_start].to_string();
        }
        
        // Extract host and port
        if !url.is_empty() {
            if let Some(port_start) = url.rfind(':') {
                if let Ok(port) = url[port_start + 1..].parse::<u16>() {
                    parsed.port = Some(port);
                    parsed.host = Some(url[..port_start].to_string());
                } else {
                    parsed.host = Some(url);
                }
            } else {
                parsed.host = Some(url);
            }
        }
        
        Ok(parsed)
    }
    
    pub fn to_string(&self) -> String {
        let mut result = format!("{}://", self.scheme);
        
        if let Some(host) = &self.host {
            result.push_str(host);
            if let Some(port) = self.port {
                result.push_str(&format!(":{}", port));
            }
        }
        
        result.push_str(&self.path);
        
        if !self.query.is_empty() {
            result.push('?');
            let query_pairs: Vec<String> = self.query.iter()
                .map(|(k, v)| if v.is_empty() { k.clone() } else { format!("{}={}", k, v) })
                .collect();
            result.push_str(&query_pairs.join("&"));
        }
        
        if let Some(fragment) = &self.fragment {
            result.push('#');
            result.push_str(fragment);
        }
        
        result
    }
}