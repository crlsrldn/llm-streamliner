use super::{Compressor, Expander, StreamlinerError};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::io::{Error, ErrorKind, Read, Write};

/// Zlib-based compression implementation
pub struct ZlibCompressor;

#[async_trait::async_trait]
impl Compressor for ZlibCompressor {
    async fn compress(
        &'async_trait self,
        context: &'async_trait str,
    ) -> Result<Vec<u8>, StreamlinerError> {
        let context = context.to_owned();

        tokio::task::spawn_blocking(move || -> Result<Vec<u8>, StreamlinerError> {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(context.as_bytes())?;
            Ok(encoder.finish()?)
        })
        .await
        .map_err(|e| StreamlinerError::CompressionError(Error::new(ErrorKind::Other, e)))?
    }
}

/// Zlib-based expansion implementation
pub struct ZlibExpander;

#[async_trait::async_trait]
impl Expander for ZlibExpander {
    async fn expand(
        &'async_trait self,
        compressed: &'async_trait [u8],
    ) -> Result<String, StreamlinerError> {
        let compressed = compressed.to_vec();

        tokio::task::spawn_blocking(move || -> Result<String, StreamlinerError> {
            let mut decoder = ZlibDecoder::new(compressed.as_slice());
            let mut output = String::new();
            decoder
                .read_to_string(&mut output)
                .map_err(|e| StreamlinerError::ExpansionError(e.to_string()))?;
            Ok(output)
        })
        .await
        .map_err(|e| StreamlinerError::ExpansionError(e.to_string()))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_zlib_roundtrip() {
        let compressor = ZlibCompressor;
        let expander = ZlibExpander;
        let original = "test context";

        let compressed = compressor.compress(original).await.unwrap();
        let expanded = expander.expand(&compressed).await.unwrap();

        assert_eq!(original, expanded);
    }

    #[tokio::test]
    async fn test_zlib_invalid_data_expansion_error() {
        let expander = ZlibExpander;
        let invalid_data = b"not valid zlib";

        let result = expander.expand(invalid_data).await;

        assert!(matches!(result, Err(StreamlinerError::ExpansionError(_))));
    }
}
