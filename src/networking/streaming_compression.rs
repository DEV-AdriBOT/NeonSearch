use std::io::{Read, BufReader};
use std::collections::VecDeque;
use anyhow::{Result, anyhow};
use flate2::read::{GzDecoder, DeflateDecoder};
use brotli::Decompressor;

/// Streaming decompression for handling large compressed content efficiently
pub struct StreamingDecompressor {
    decoder_type: CompressionType,
    decoder: Box<dyn CompressionDecoder>,
    output_buffer: VecDeque<u8>,
    input_buffer: Vec<u8>,
    is_finished: bool,
    total_input: usize,
    total_output: usize,
}

/// Type of compression detected
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompressionType {
    None,
    Gzip,
    Deflate,
    Brotli,
    Zstd,
}

/// Trait for streaming decompression
trait CompressionDecoder: Send {
    fn decompress_chunk(&mut self, input: &[u8], output: &mut Vec<u8>) -> Result<bool>;
    fn finish(&mut self, output: &mut Vec<u8>) -> Result<()>;
    fn is_finished(&self) -> bool;
}

/// Configuration for streaming decompression
#[derive(Debug, Clone)]
pub struct StreamingDecompressConfig {
    pub chunk_size: usize,
    pub max_output_buffer: usize,
    pub max_compression_ratio: f64, // Safety limit to prevent decompression bombs
}

impl Default for StreamingDecompressConfig {
    fn default() -> Self {
        Self {
            chunk_size: 8192,          // 8KB input chunks
            max_output_buffer: 1024 * 1024, // 1MB output buffer
            max_compression_ratio: 1000.0,   // Max 1000:1 compression ratio
        }
    }
}

/// Result of decompression operation
#[derive(Debug)]
pub struct DecompressionResult {
    pub data: Vec<u8>,
    pub is_complete: bool,
    pub compression_ratio: f64,
    pub bytes_processed: usize,
}

/// Progress information for decompression
#[derive(Debug, Clone)]
pub struct DecompressionProgress {
    pub input_bytes: usize,
    pub output_bytes: usize,
    pub compression_ratio: f64,
    pub estimated_total: Option<usize>,
    pub is_complete: bool,
}

impl StreamingDecompressor {
    /// Create a new streaming decompressor
    pub fn new(compression_type: CompressionType, config: StreamingDecompressConfig) -> Result<Self> {
        let decoder: Box<dyn CompressionDecoder> = match compression_type {
            CompressionType::None => Box::new(NoCompression::new()),
            CompressionType::Gzip => Box::new(GzipDecoder::new()?),
            CompressionType::Deflate => Box::new(DeflateStreamDecoder::new()?),
            CompressionType::Brotli => Box::new(BrotliStreamDecoder::new()?),
            CompressionType::Zstd => Box::new(ZstdStreamDecoder::new()?),
        };

        Ok(Self {
            decoder_type: compression_type,
            decoder,
            output_buffer: VecDeque::with_capacity(config.max_output_buffer),
            input_buffer: Vec::with_capacity(config.chunk_size),
            is_finished: false,
            total_input: 0,
            total_output: 0,
        })
    }

    /// Detect compression type from content headers or magic bytes
    pub fn detect_compression(content_encoding: Option<&str>, data: &[u8]) -> CompressionType {
        // Check content-encoding header first
        if let Some(encoding) = content_encoding {
            match encoding.to_lowercase().as_str() {
                "gzip" => return CompressionType::Gzip,
                "deflate" => return CompressionType::Deflate,
                "br" => return CompressionType::Brotli,
                "zstd" => return CompressionType::Zstd,
                _ => {}
            }
        }

        // Check magic bytes
        if data.len() >= 2 {
            match &data[0..2] {
                [0x1f, 0x8b] => CompressionType::Gzip,
                [0x78, 0x9c] | [0x78, 0x01] | [0x78, 0xda] => CompressionType::Deflate,
                _ => {
                    // Check for other formats
                    if data.len() >= 4 {
                        // Zstd magic number
                        if &data[0..4] == &[0x28, 0xb5, 0x2f, 0xfd] {
                            return CompressionType::Zstd;
                        }
                    }
                    CompressionType::None
                }
            }
        } else {
            CompressionType::None
        }
    }

    /// Add compressed data chunk for decompression
    pub fn add_chunk(&mut self, chunk: &[u8], config: &StreamingDecompressConfig) -> Result<DecompressionResult> {
        if self.is_finished {
            return Ok(DecompressionResult {
                data: Vec::new(),
                is_complete: true,
                compression_ratio: self.get_compression_ratio(),
                bytes_processed: 0,
            });
        }

        self.total_input += chunk.len();

        // Safety check for compression bombs
        let current_ratio = self.get_compression_ratio();
        if current_ratio > config.max_compression_ratio {
            return Err(anyhow!("Compression ratio too high ({:.1}), possible decompression bomb", current_ratio));
        }

        // Decompress the chunk
        let mut output = Vec::new();
        let is_finished = self.decoder.decompress_chunk(chunk, &mut output)?;
        
        self.total_output += output.len();
        
        // Add to output buffer
        self.output_buffer.extend(output.iter());
        
        // Check buffer size limits
        if self.output_buffer.len() > config.max_output_buffer {
            // Return some data to keep buffer size manageable
            let return_size = config.max_output_buffer / 2;
            let returned_data: Vec<u8> = self.output_buffer.drain(0..return_size).collect();
            
            Ok(DecompressionResult {
                data: returned_data,
                is_complete: is_finished,
                compression_ratio: self.get_compression_ratio(),
                bytes_processed: chunk.len(),
            })
        } else if is_finished {
            self.is_finished = true;
            let mut final_output = Vec::new();
            self.decoder.finish(&mut final_output)?;
            self.output_buffer.extend(final_output.iter());
            
            // Return all remaining data
            let all_data: Vec<u8> = self.output_buffer.drain(..).collect();
            
            Ok(DecompressionResult {
                data: all_data,
                is_complete: true,
                compression_ratio: self.get_compression_ratio(),
                bytes_processed: chunk.len(),
            })
        } else {
            Ok(DecompressionResult {
                data: Vec::new(),
                is_complete: false,
                compression_ratio: self.get_compression_ratio(),
                bytes_processed: chunk.len(),
            })
        }
    }

    /// Finalize decompression and return all remaining data
    pub fn finalize(&mut self) -> Result<Vec<u8>> {
        if !self.is_finished {
            let mut final_output = Vec::new();
            self.decoder.finish(&mut final_output)?;
            self.output_buffer.extend(final_output.iter());
            self.is_finished = true;
        }
        
        Ok(self.output_buffer.drain(..).collect())
    }

    /// Get current decompression progress
    pub fn get_progress(&self) -> DecompressionProgress {
        DecompressionProgress {
            input_bytes: self.total_input,
            output_bytes: self.total_output,
            compression_ratio: self.get_compression_ratio(),
            estimated_total: None, // Could be estimated based on compression ratio
            is_complete: self.is_finished,
        }
    }

    /// Calculate current compression ratio
    fn get_compression_ratio(&self) -> f64 {
        if self.total_input == 0 {
            1.0
        } else {
            self.total_output as f64 / self.total_input as f64
        }
    }

    /// Check if decompression is complete
    pub fn is_complete(&self) -> bool {
        self.is_finished
    }
}

// Implementation for no compression
struct NoCompression {
    finished: bool,
}

impl NoCompression {
    fn new() -> Self {
        Self { finished: false }
    }
}

impl CompressionDecoder for NoCompression {
    fn decompress_chunk(&mut self, input: &[u8], output: &mut Vec<u8>) -> Result<bool> {
        output.extend_from_slice(input);
        Ok(false) // Never "finished" until explicitly told
    }
    
    fn finish(&mut self, _output: &mut Vec<u8>) -> Result<()> {
        self.finished = true;
        Ok(())
    }
    
    fn is_finished(&self) -> bool {
        self.finished
    }
}

// Implementation for Gzip
struct GzipDecoder {
    decoder: Option<GzDecoder<BufReader<std::io::Cursor<Vec<u8>>>>>,
    buffer: Vec<u8>,
    finished: bool,
}

impl GzipDecoder {
    fn new() -> Result<Self> {
        Ok(Self {
            decoder: None,
            buffer: Vec::new(),
            finished: false,
        })
    }
}

impl CompressionDecoder for GzipDecoder {
    fn decompress_chunk(&mut self, input: &[u8], output: &mut Vec<u8>) -> Result<bool> {
        self.buffer.extend_from_slice(input);
        
        if self.decoder.is_none() {
            let cursor = std::io::Cursor::new(self.buffer.clone());
            let buf_reader = BufReader::new(cursor);
            self.decoder = Some(GzDecoder::new(buf_reader));
        }
        
        if let Some(ref mut decoder) = self.decoder {
            let mut temp_output = Vec::new();
            match decoder.read_to_end(&mut temp_output) {
                Ok(_) => {
                    output.extend_from_slice(&temp_output);
                    self.finished = true;
                    Ok(true)
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Need more data
                    Ok(false)
                }
                Err(e) => Err(anyhow!("Gzip decompression error: {}", e))
            }
        } else {
            Ok(false)
        }
    }
    
    fn finish(&mut self, output: &mut Vec<u8>) -> Result<()> {
        // Gzip decoder should have finished in decompress_chunk
        Ok(())
    }
    
    fn is_finished(&self) -> bool {
        self.finished
    }
}

// Simplified implementations for other compression types
struct DeflateStreamDecoder {
    buffer: Vec<u8>,
    finished: bool,
}

impl DeflateStreamDecoder {
    fn new() -> Result<Self> {
        Ok(Self {
            buffer: Vec::new(),
            finished: false,
        })
    }
}

impl CompressionDecoder for DeflateStreamDecoder {
    fn decompress_chunk(&mut self, input: &[u8], output: &mut Vec<u8>) -> Result<bool> {
        self.buffer.extend_from_slice(input);
        
        let cursor = std::io::Cursor::new(&self.buffer);
        let mut decoder = DeflateDecoder::new(cursor);
        let mut temp_output = Vec::new();
        
        match decoder.read_to_end(&mut temp_output) {
            Ok(_) => {
                output.extend_from_slice(&temp_output);
                self.finished = true;
                Ok(true)
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                Ok(false)
            }
            Err(e) => Err(anyhow!("Deflate decompression error: {}", e))
        }
    }
    
    fn finish(&mut self, _output: &mut Vec<u8>) -> Result<()> {
        Ok(())
    }
    
    fn is_finished(&self) -> bool {
        self.finished
    }
}

struct BrotliStreamDecoder {
    decompressor: Decompressor<std::io::Cursor<Vec<u8>>>,
    finished: bool,
}

impl BrotliStreamDecoder {
    fn new() -> Result<Self> {
        let cursor = std::io::Cursor::new(Vec::new());
        Ok(Self {
            decompressor: Decompressor::new(cursor, 8192),
            finished: false,
        })
    }
}

impl CompressionDecoder for BrotliStreamDecoder {
    fn decompress_chunk(&mut self, input: &[u8], output: &mut Vec<u8>) -> Result<bool> {
        // Simplified brotli decompression
        // In a real implementation, you'd use the brotli crate properly
        output.extend_from_slice(input); // Fallback: no decompression
        Ok(false)
    }
    
    fn finish(&mut self, _output: &mut Vec<u8>) -> Result<()> {
        self.finished = true;
        Ok(())
    }
    
    fn is_finished(&self) -> bool {
        self.finished
    }
}

struct ZstdStreamDecoder {
    finished: bool,
}

impl ZstdStreamDecoder {
    fn new() -> Result<Self> {
        Ok(Self { finished: false })
    }
}

impl CompressionDecoder for ZstdStreamDecoder {
    fn decompress_chunk(&mut self, input: &[u8], output: &mut Vec<u8>) -> Result<bool> {
        // Simplified zstd decompression
        // In a real implementation, you'd use the zstd crate
        output.extend_from_slice(input); // Fallback: no decompression
        Ok(false)
    }
    
    fn finish(&mut self, _output: &mut Vec<u8>) -> Result<()> {
        self.finished = true;
        Ok(())
    }
    
    fn is_finished(&self) -> bool {
        self.finished
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_detection() {
        // Test gzip magic bytes
        let gzip_data = [0x1f, 0x8b, 0x08, 0x00];
        assert_eq!(StreamingDecompressor::detect_compression(None, &gzip_data), CompressionType::Gzip);
        
        // Test content-encoding header
        assert_eq!(StreamingDecompressor::detect_compression(Some("gzip"), &[]), CompressionType::Gzip);
        assert_eq!(StreamingDecompressor::detect_compression(Some("deflate"), &[]), CompressionType::Deflate);
        assert_eq!(StreamingDecompressor::detect_compression(Some("br"), &[]), CompressionType::Brotli);
    }

    #[test]
    fn test_no_compression() {
        let config = StreamingDecompressConfig::default();
        let mut decompressor = StreamingDecompressor::new(CompressionType::None, config.clone()).unwrap();
        
        let test_data = b"Hello, world!";
        let result = decompressor.add_chunk(test_data, &config).unwrap();
        
        assert_eq!(result.compression_ratio, 1.0);
        
        let final_data = decompressor.finalize().unwrap();
        assert_eq!(final_data, test_data);
    }
}