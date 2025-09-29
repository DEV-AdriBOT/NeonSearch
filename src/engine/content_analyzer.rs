use anyhow::Result;

#[derive(Debug)]
pub struct ContentAnalyzer;

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub truncated_content: String,
    pub original_size: usize,
    pub final_size: usize,
    pub compression_ratio: f32,
}

impl ContentAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub fn analyze_content(&self, html: &str) -> Result<AnalysisResult> {
        let original_size = html.len();
        let max_size = 5 * 1024 * 1024;
        
        let truncated_content = if original_size > max_size {
            html.chars().take(max_size).collect()
        } else {
            html.to_string()
        };

        let final_size = truncated_content.len();
        let compression_ratio = if original_size > 0 {
            final_size as f32 / original_size as f32
        } else {
            1.0
        };

        Ok(AnalysisResult {
            truncated_content,
            original_size,
            final_size,
            compression_ratio,
        })
    }
}
