//! LLM-Streamliner: Incremental compression/expansion pipelines for LLM contexts
//! 
//! Provides traits and implementations for compressing LLM context into memory modules
//! that can be efficiently stored and expanded when needed.

use thiserror::Error;
use serde::{Serialize, Deserialize};

pub mod compression;

pub use compression::{ZlibCompressor, ZlibExpander};

/// Error type for compression/expansion operations
#[derive(Error, Debug)]
pub enum StreamlinerError {
    #[error("Compression failed: {0}")]
    CompressionError(#[from] std::io::Error),
    #[error("Expansion failed: {0}")]
    ExpansionError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Trait for compressing text into binary representations
#[async_trait::async_trait]
pub trait Compressor {
    /// Compresses text into a binary format
    /// # Arguments
    /// * `context` - The text context to compress
    /// # Returns
    /// Binary representation of the compressed text or error
    async fn compress(&'async_trait self, context: &'async_trait str) -> Result<Vec<u8>, StreamlinerError>;
}

/// Trait for expanding binary representations back into text
#[async_trait::async_trait]
pub trait Expander {
    /// Expands binary data back into text
    /// # Arguments
    /// * `compressed` - The compressed binary data
    /// # Returns
    /// The original text or error
    async fn expand(&'async_trait self, compressed: &'async_trait [u8]) -> Result<String, StreamlinerError>;
}

/// Compressed memory module containing context and metadata
#[derive(Serialize, Deserialize)]
pub struct MemoryModule {
    /// The compressed data bytes
    compressed_data: Vec<u8>,
    /// Metadata about the compression (algorithm, version, etc.)
    metadata: String,
}

impl MemoryModule {
    /// Creates a new MemoryModule by compressing the given context
    pub async fn new(context: &str, compressor: &impl Compressor) -> Result<Self, StreamlinerError> {
        let compressed_data = compressor.compress(context).await?;
        Ok(Self {
            compressed_data,
            metadata: String::new(),
        })
    }

    /// Expands the compressed data back into text
    pub async fn expand(&self, expander: &impl Expander) -> Result<String, StreamlinerError> {
        expander.expand(&self.compressed_data).await
    }

    /// Serializes the module to a JSON string
    pub fn to_json(&self) -> Result<String, StreamlinerError> {
        serde_json::to_string(self).map_err(Into::into)
    }

    /// Deserializes a module from a JSON string
    pub fn from_json(json: &str) -> Result<Self, StreamlinerError> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Gets metadata about the compression
    pub fn metadata(&self) -> &str {
        &self.metadata
    }

    /// Updates the metadata
    pub fn set_metadata(&mut self, metadata: String) {
        self.metadata = metadata;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    struct TestCompressor;
    struct TestExpander;

    #[test]
    async fn test_zlib_implementation() {
        let compressor = ZlibCompressor;
        let expander = ZlibExpander;
        let original = "This is a longer test string to verify zlib compression works properly";
        
        let module = MemoryModule::new(original, &compressor).await.unwrap();
        let expanded = module.expand(&expander).await.unwrap();
        
        assert_eq!(original, expanded);
        
        // Verify serialization roundtrip
        let json = module.to_json().unwrap();
        let deserialized = MemoryModule::from_json(&json).unwrap();
        let reexpanded = deserialized.expand(&expander).await.unwrap();
        assert_eq!(original, reexpanded);
    }

    #[async_trait::async_trait]
    impl Compressor for TestCompressor {
        async fn compress(&'async_trait self, context: &'async_trait str) -> Result<Vec<u8>, StreamlinerError> {
            Ok(context.as_bytes().to_vec())
        }
    }

    #[async_trait::async_trait]
    impl Expander for TestExpander {
        async fn expand(&'async_trait self, compressed: &'async_trait [u8]) -> Result<String, StreamlinerError> {
            String::from_utf8(compressed.to_vec())
                .map_err(|e| StreamlinerError::ExpansionError(format!("UTF-8 conversion failed: {}", e)))
        }
    }

    #[test]
    async fn test_memory_module_roundtrip() {
        let compressor = TestCompressor;
        let expander = TestExpander;
        let original = "test context";
        
        let module = MemoryModule::new(original, &compressor).await.unwrap();
        let expanded = module.expand(&expander).await.unwrap();
        
        assert_eq!(original, expanded);
    }
}
