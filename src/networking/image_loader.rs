use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use tokio::sync::Mutex;
use image::{DynamicImage, ImageFormat};
use egui::{ColorImage, TextureHandle, Context};
use crate::networking::manual_client::ManualHttpClient;

#[derive(Clone)]
pub struct ImageCache {
    cache: Arc<Mutex<HashMap<String, Arc<ColorImage>>>>,
    egui_textures: Arc<Mutex<HashMap<String, TextureHandle>>>,
}

impl ImageCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            egui_textures: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_image(&self, url: &str, ctx: &Context) -> Option<TextureHandle> {
        // Check if we already have the texture
        {
            let textures = self.egui_textures.lock().await;
            if let Some(texture) = textures.get(url) {
                return Some(texture.clone());
            }
        }

        // Check if we have the image data cached
        {
            let cache = self.cache.lock().await;
            if let Some(color_image) = cache.get(url) {
                let texture = ctx.load_texture(
                    format!("img_{}", url.chars().take(50).collect::<String>()),
                    color_image.as_ref().clone(),
                    Default::default()
                );
                
                // Store the texture for future use
                let mut textures = self.egui_textures.lock().await;
                textures.insert(url.to_string(), texture.clone());
                return Some(texture);
            }
        }

        None
    }

    pub async fn load_image(&self, url: &str, client: &ManualHttpClient) -> Result<Arc<ColorImage>> {
        // Check cache first
        {
            let cache = self.cache.lock().await;
            if let Some(image) = cache.get(url) {
                return Ok(image.clone());
            }
        }

        println!("Loading image: {}", url);
        
        // Fetch the image
        let fetch_result = client.fetch(url).await
            .map_err(|e| anyhow!("Failed to fetch image {}: {}", url, e))?;
        
        if !fetch_result.response.is_success() {
            return Err(anyhow!("Image request failed with status {}: {}", 
                             fetch_result.response.status_code, url));
        }

        // Detect image format from content-type or URL extension
        let content_type = fetch_result.response.content_type()
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        
        let format = detect_image_format(&content_type, url, &fetch_result.response.body)?;
        
        // Load and convert the image
        let dynamic_image = image::load_from_memory_with_format(&fetch_result.response.body, format)
            .map_err(|e| anyhow!("Failed to decode image {}: {}", url, e))?;
        
        let color_image = convert_to_color_image(dynamic_image)?;
        let arc_image = Arc::new(color_image);

        // Cache the result
        {
            let mut cache = self.cache.lock().await;
            cache.insert(url.to_string(), arc_image.clone());
        }

        println!("Successfully loaded and cached image: {} ({}x{})", 
                url, arc_image.width(), arc_image.height());
        
        Ok(arc_image)
    }

    pub async fn preload_favicon(&self, base_url: &str, html_content: &str, client: &ManualHttpClient) {
        let favicon_urls = extract_favicon_urls(base_url, html_content);
        
        for favicon_url in favicon_urls {
            if let Ok(_) = self.load_image(&favicon_url, client).await {
                println!("Preloaded favicon: {}", favicon_url);
                break; // Stop at first successful favicon
            }
        }
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.lock().await;
        let mut textures = self.egui_textures.lock().await;
        cache.clear();
        textures.clear();
    }
}

fn detect_image_format(content_type: &str, url: &str, data: &[u8]) -> Result<ImageFormat> {
    // Try content-type first
    if content_type.contains("png") {
        return Ok(ImageFormat::Png);
    } else if content_type.contains("jpeg") || content_type.contains("jpg") {
        return Ok(ImageFormat::Jpeg);
    } else if content_type.contains("gif") {
        return Ok(ImageFormat::Gif);
    } else if content_type.contains("webp") {
        return Ok(ImageFormat::WebP);
    } else if content_type.contains("bmp") {
        return Ok(ImageFormat::Bmp);
    } else if content_type.contains("svg") {
        return Err(anyhow!("SVG format not supported yet"));
    } else if content_type.contains("icon") || content_type.contains("x-icon") {
        return Ok(ImageFormat::Ico);
    }

    // Try URL extension
    let url_lower = url.to_lowercase();
    if url_lower.ends_with(".png") {
        return Ok(ImageFormat::Png);
    } else if url_lower.ends_with(".jpg") || url_lower.ends_with(".jpeg") {
        return Ok(ImageFormat::Jpeg);
    } else if url_lower.ends_with(".gif") {
        return Ok(ImageFormat::Gif);
    } else if url_lower.ends_with(".webp") {
        return Ok(ImageFormat::WebP);
    } else if url_lower.ends_with(".bmp") {
        return Ok(ImageFormat::Bmp);
    } else if url_lower.ends_with(".ico") {
        return Ok(ImageFormat::Ico);
    }

    // Try to detect from file signature (magic bytes)
    if data.len() >= 8 {
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
            return Ok(ImageFormat::Png);
        } else if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Ok(ImageFormat::Jpeg);
        } else if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            return Ok(ImageFormat::Gif);
        } else if data.starts_with(b"RIFF") && data.len() >= 12 && &data[8..12] == b"WEBP" {
            return Ok(ImageFormat::WebP);
        } else if data.starts_with(&[0x42, 0x4D]) {
            return Ok(ImageFormat::Bmp);
        } else if data.starts_with(&[0x00, 0x00, 0x01, 0x00]) {
            return Ok(ImageFormat::Ico);
        }
    }

    // Default fallback
    Err(anyhow!("Could not determine image format for {}", url))
}

fn convert_to_color_image(dynamic_image: DynamicImage) -> Result<ColorImage> {
    let rgba_image = dynamic_image.to_rgba8();
    let size = [rgba_image.width() as usize, rgba_image.height() as usize];
    let pixels = rgba_image.as_flat_samples();
    
    if size[0] > 2048 || size[1] > 2048 {
        // Resize large images to prevent memory issues
        let resized = dynamic_image.resize(512, 512, image::imageops::FilterType::Lanczos3);
        return convert_to_color_image(resized);
    }
    
    let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
    Ok(color_image)
}

fn extract_favicon_urls(base_url: &str, html_content: &str) -> Vec<String> {
    let mut urls = Vec::new();
    
    // Parse base URL
    let base = match reqwest::Url::parse(base_url) {
        Ok(url) => url,
        Err(_) => return urls,
    };

    // Look for favicon links in HTML
    let html_lower = html_content.to_lowercase();
    let lines: Vec<&str> = html_lower.split('\n').collect();
    
    for line in lines {
        if line.contains("rel") && (line.contains("icon") || line.contains("shortcut")) {
            if let Some(href_start) = line.find("href") {
                let after_href = &line[href_start + 4..];
                if let Some(eq_pos) = after_href.find('=') {
                    let after_eq = &after_href[eq_pos + 1..].trim_start();
                    let quote_char = if after_eq.starts_with('"') { '"' } else if after_eq.starts_with('\'') { '\'' } else { ' ' };
                    
                    if quote_char != ' ' {
                        let after_quote = &after_eq[1..];
                        if let Some(end_quote) = after_quote.find(quote_char) {
                            let href_value = &after_quote[..end_quote];
                            
                            // Resolve relative URL
                            if let Ok(full_url) = base.join(href_value) {
                                urls.push(full_url.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // Add default favicon locations
    if let Ok(favicon_url) = base.join("/favicon.ico") {
        urls.push(favicon_url.to_string());
    }
    if let Ok(favicon_url) = base.join("/favicon.png") {
        urls.push(favicon_url.to_string());
    }
    
    urls
}

// Default fallback favicon as a simple colored square
pub fn create_fallback_favicon(ctx: &Context, url: &str) -> TextureHandle {
    // Create a simple 16x16 colored square based on URL hash
    let mut hash = 0u32;
    for byte in url.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }
    
    let r = ((hash >> 16) & 0xFF) as u8;
    let g = ((hash >> 8) & 0xFF) as u8;
    let b = (hash & 0xFF) as u8;
    
    let size = [16, 16];
    let mut pixels = vec![0u8; size[0] * size[1] * 4];
    
    for i in 0..size[0] * size[1] {
        let base = i * 4;
        pixels[base] = r;
        pixels[base + 1] = g;
        pixels[base + 2] = b;
        pixels[base + 3] = 255; // Alpha
    }
    
    let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);
    ctx.load_texture(
        format!("fallback_{}", hash),
        color_image,
        Default::default()
    )
}