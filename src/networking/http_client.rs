use crate::networking::{HttpRequest, HttpResponse};
use anyhow::Result;
use std::collections::HashMap;
use std::time::Duration;

pub async fn send_request(request: HttpRequest) -> Result<HttpResponse> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("NeonSearch/0.1.0 (Custom Browser by NeonDev)")
        .build()?;
    
    let mut req_builder = match request.method.to_uppercase().as_str() {
        "GET" => client.get(&request.url),
        "POST" => client.post(&request.url),
        "PUT" => client.put(&request.url),
        "DELETE" => client.delete(&request.url),
        "HEAD" => client.head(&request.url),
        "PATCH" => client.patch(&request.url),
        _ => client.get(&request.url), // Default to GET for unknown methods
    };
    
    // Add headers
    for (name, value) in &request.headers {
        req_builder = req_builder.header(name, value);
    }
    
    // Add body if present
    if let Some(body) = request.body {
        req_builder = req_builder.body(body);
    }
    
    let response = req_builder.send().await?;
    
    let status_code = response.status().as_u16();
    let status_text = response.status().canonical_reason()
        .unwrap_or("Unknown Status")
        .to_string();
    
    let mut headers = HashMap::new();
    for (name, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(name.to_string(), value_str.to_string());
        }
    }
    
    let body = response.bytes().await?.to_vec();
    
    Ok(HttpResponse::new(status_code, status_text, headers, body))
}

pub async fn download_file(url: &str) -> Result<Vec<u8>> {
    let request = HttpRequest::new_get(url.to_string());
    let response = send_request(request).await?;
    
    if response.is_success() {
        Ok(response.body)
    } else {
        Err(anyhow::anyhow!("Failed to download file: HTTP {}", response.status_code))
    }
}

pub fn is_secure_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("wss://")
}

pub fn get_domain_from_url(url: &str) -> Option<String> {
    if let Ok(parsed) = reqwest::Url::parse(url) {
        parsed.host_str().map(|s| s.to_string())
    } else {
        None
    }
}