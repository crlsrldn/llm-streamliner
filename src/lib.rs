//! LLM-Streamliner: Incremental compression/expansion pipelines for LLM contexts
//!
//! Provides traits and implementations for compressing LLM context into memory modules
//! that can be efficiently stored and expanded when needed.

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    #[error("Unsupported schema version: {0}")]
    UnsupportedSchemaVersion(u32),
}

/// Trait for compressing text into binary representations
#[async_trait::async_trait]
pub trait Compressor {
    /// Compresses text into a binary format
    /// # Arguments
    /// * `context` - The text context to compress
    /// # Returns
    /// Binary representation of the compressed text or error
    async fn compress(
        &'async_trait self,
        context: &'async_trait str,
    ) -> Result<Vec<u8>, StreamlinerError>;

    /// Identifies the compression algorithm used by this compressor
    fn algorithm(&self) -> &'static str;
}

/// Trait for expanding binary representations back into text
#[async_trait::async_trait]
pub trait Expander {
    /// Expands binary data back into text
    /// # Arguments
    /// * `compressed` - The compressed binary data
    /// # Returns
    /// The original text or error
    async fn expand(
        &'async_trait self,
        compressed: &'async_trait [u8],
    ) -> Result<String, StreamlinerError>;

    /// Identifies the compression algorithm supported by this expander
    fn algorithm(&self) -> &'static str;
}

/// Metadata describing a compressed memory module
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MemoryMetadata {
    /// Compression algorithm name (e.g., "zlib")
    algorithm: String,
    /// Schema version to support future evolution of the format
    #[serde(default = "MemoryMetadata::current_schema_version")]
    schema_version: u32,
    /// Optional user-provided tags for discoverability
    #[serde(default, skip_serializing_if = "Option::is_none")]
    user_tags: Option<Vec<String>>,
}

impl MemoryMetadata {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    const fn current_schema_version() -> u32 {
        Self::CURRENT_SCHEMA_VERSION
    }

    pub fn new(algorithm: String, user_tags: Option<Vec<String>>) -> Self {
        Self {
            algorithm,
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            user_tags,
        }
    }

    pub fn algorithm(&self) -> &str {
        &self.algorithm
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn user_tags(&self) -> Option<&[String]> {
        self.user_tags.as_deref()
    }
}

/// Compressed memory module containing context and metadata
#[derive(Serialize, Deserialize, Debug)]
pub struct MemoryModule {
    /// The compressed data bytes
    compressed_data: Vec<u8>,
    /// Metadata about the compression (algorithm, version, etc.)
    metadata: MemoryMetadata,
}

impl MemoryModule {
    /// Creates a new MemoryModule by compressing the given context
    pub async fn new(
        context: &str,
        compressor: &impl Compressor,
        user_tags: Option<Vec<String>>,
    ) -> Result<Self, StreamlinerError> {
        let compressed_data = compressor.compress(context).await?;
        Ok(Self {
            compressed_data,
            metadata: MemoryMetadata::new(compressor.algorithm().to_string(), user_tags),
        })
    }

    /// Expands the compressed data back into text
    pub async fn expand(&self, expander: &impl Expander) -> Result<String, StreamlinerError> {
        if expander.algorithm() != self.metadata.algorithm {
            return Err(StreamlinerError::ExpansionError(format!(
                "Expander for '{}' cannot expand algorithm '{}'",
                expander.algorithm(),
                self.metadata.algorithm
            )));
        }
        expander.expand(&self.compressed_data).await
    }

    /// Serializes the module to a JSON string
    pub fn to_json(&self) -> Result<String, StreamlinerError> {
        serde_json::to_string(self).map_err(Into::into)
    }

    /// Deserializes a module from a JSON string
    pub fn from_json(json: &str) -> Result<Self, StreamlinerError> {
        let module: Self = serde_json::from_str(json)?;
        if module.metadata.schema_version != MemoryMetadata::CURRENT_SCHEMA_VERSION {
            return Err(StreamlinerError::UnsupportedSchemaVersion(
                module.metadata.schema_version,
            ));
        }
        Ok(module)
    }

    /// Gets metadata about the compression
    pub fn metadata(&self) -> &MemoryMetadata {
        &self.metadata
    }

    /// Gets the compression algorithm name
    pub fn algorithm(&self) -> &str {
        self.metadata.algorithm()
    }

    /// Gets the schema version number
    pub fn schema_version(&self) -> u32 {
        self.metadata.schema_version()
    }

    /// Gets the optional user tags
    pub fn user_tags(&self) -> Option<&[String]> {
        self.metadata.user_tags()
    }

    /// Updates the metadata
    pub fn set_metadata(&mut self, metadata: MemoryMetadata) {
        self.metadata = metadata;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    struct TestCompressor;
    struct TestExpander;

    #[tokio::test]
    async fn test_zlib_implementation() {
        let compressor = ZlibCompressor;
        let expander = ZlibExpander;
        let original = "This is a longer test string to verify zlib compression works properly";

        let module = MemoryModule::new(original, &compressor, None)
            .await
            .unwrap();
        let expanded = module.expand(&expander).await.unwrap();

        assert_eq!(original, expanded);

        assert_eq!(module.algorithm(), "zlib");
        assert_eq!(
            module.schema_version(),
            MemoryMetadata::CURRENT_SCHEMA_VERSION
        );
        assert!(module.user_tags().is_none());

        // Verify serialization roundtrip
        let json = module.to_json().unwrap();
        let deserialized = MemoryModule::from_json(&json).unwrap();
        assert_eq!(deserialized.algorithm(), "zlib");
        assert_eq!(
            deserialized.schema_version(),
            MemoryMetadata::CURRENT_SCHEMA_VERSION
        );
        let reexpanded = deserialized.expand(&expander).await.unwrap();
        assert_eq!(original, reexpanded);
    }

    #[async_trait::async_trait]
    impl Compressor for TestCompressor {
        async fn compress(
            &'async_trait self,
            context: &'async_trait str,
        ) -> Result<Vec<u8>, StreamlinerError> {
            Ok(context.as_bytes().to_vec())
        }

        fn algorithm(&self) -> &'static str {
            "identity"
        }
    }

    #[async_trait::async_trait]
    impl Expander for TestExpander {
        async fn expand(
            &'async_trait self,
            compressed: &'async_trait [u8],
        ) -> Result<String, StreamlinerError> {
            String::from_utf8(compressed.to_vec()).map_err(|e| {
                StreamlinerError::ExpansionError(format!("UTF-8 conversion failed: {}", e))
            })
        }

        fn algorithm(&self) -> &'static str {
            "identity"
        }
    }

    #[tokio::test]
    async fn test_memory_module_roundtrip() {
        let compressor = TestCompressor;
        let expander = TestExpander;
        let original = "test context";

        let module = MemoryModule::new(original, &compressor, Some(vec!["demo".into()]))
            .await
            .unwrap();
        let expanded = module.expand(&expander).await.unwrap();

        assert_eq!(original, expanded);
        assert_eq!(module.algorithm(), "identity");
        assert_eq!(
            module.schema_version(),
            MemoryMetadata::CURRENT_SCHEMA_VERSION
        );
        assert_eq!(module.user_tags(), Some(["demo".to_string()].as_slice()));

        // Ensure mismatched expander is rejected based on metadata
        let zlib_expander = ZlibExpander;
        let err = module.expand(&zlib_expander).await.unwrap_err();
        assert!(format!("{}", err).contains("cannot expand algorithm"));
    }

    #[tokio::test]
    async fn test_schema_version_defaults_and_validation() {
        let compressor = TestCompressor;
        let module = MemoryModule::new("context", &compressor, None)
            .await
            .unwrap();

        // Simulate a legacy payload with no schema_version and ensure defaulting works
        let mut value = serde_json::to_value(&module).unwrap();
        let metadata = value
            .get_mut("metadata")
            .and_then(Value::as_object_mut)
            .expect("metadata object present");
        metadata.remove("schema_version");

        let legacy_json = serde_json::to_string(&value).unwrap();
        let deserialized = MemoryModule::from_json(&legacy_json).unwrap();
        assert_eq!(
            deserialized.schema_version(),
            MemoryMetadata::CURRENT_SCHEMA_VERSION
        );

        // Reject future schema versions
        let mut future_value = serde_json::to_value(&module).unwrap();
        let future_metadata = future_value
            .get_mut("metadata")
            .and_then(Value::as_object_mut)
            .expect("metadata object present");
        future_metadata.insert(
            "schema_version".to_string(),
            (MemoryMetadata::CURRENT_SCHEMA_VERSION + 1).into(),
        );

        let future_json = serde_json::to_string(&future_value).unwrap();
        let err = MemoryModule::from_json(&future_json).unwrap_err();
        assert!(matches!(err, StreamlinerError::UnsupportedSchemaVersion(_)));
    }
}
