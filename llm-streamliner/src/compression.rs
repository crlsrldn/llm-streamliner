use super::{Compressor, Expander, StreamlinerError};
use flate2::{Compression, write::ZlibEncoder, read::ZlibDecoder};
use std::io::{Write, Read};

/// Zlib-based compression implementation
pub struct ZlibCompressor;

#[async_trait::async_trait]
impl Compressor for ZlibCompressor {
    async fn compress(&'async_trait self, context: &'async_trait str) -> Result<Vec<u8>, StreamlinerError> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(context.as_bytes())?;
        Ok(encoder.finish()?)
    }
}

/// Zlib-based expansion implementation
pub struct ZlibExpander;

#[async_trait::async_trait]
impl Expander for ZlibExpander {
    async fn expand(&'async_trait self, compressed: &'async_trait [u8]) -> Result<String, StreamlinerError> {
        let mut decoder = ZlibDecoder::new(compressed);
        let mut output = String::new();
        decoder.read_to_string(&mut output)
            .map_err(|e| StreamlinerError::ExpansionError(e.to_string()))?;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_zlib_roundtrip() {
        let compressor = ZlibCompressor;
        let expander = ZlibExpander;
        let original = "test context";
        
        let compressed = compressor.compress(original).await.unwrap();
        let expanded = expander.expand(&compressed).await.unwrap();
        
        assert_eq!(original, expanded);
    }
}