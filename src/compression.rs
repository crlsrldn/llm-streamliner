use super::{Compressor, Expander, StreamlinerError};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::io::{Read, Write};

#[cfg(feature = "gzip")]
use flate2::{read::GzDecoder, write::GzEncoder};

#[cfg(feature = "lz4")]
use lz4_flex::frame::{FrameDecoder, FrameEncoder};

#[cfg(feature = "lz4")]
use std::io::Cursor;

/// Zlib-based compression implementation
pub struct ZlibCompressor;

#[async_trait::async_trait]
impl Compressor for ZlibCompressor {
    async fn compress(
        &'async_trait self,
        context: &'async_trait str,
    ) -> Result<Vec<u8>, StreamlinerError> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(context.as_bytes())?;
        Ok(encoder.finish()?)
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
        let mut decoder = ZlibDecoder::new(compressed);
        let mut output = String::new();
        decoder
            .read_to_string(&mut output)
            .map_err(|e| StreamlinerError::ExpansionError(e.to_string()))?;
        Ok(output)
    }
}

/// Gzip-based compression implementation
#[cfg(feature = "gzip")]
pub struct GzipCompressor;

#[cfg(feature = "gzip")]
#[async_trait::async_trait]
impl Compressor for GzipCompressor {
    async fn compress(
        &'async_trait self,
        context: &'async_trait str,
    ) -> Result<Vec<u8>, StreamlinerError> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(context.as_bytes())?;
        Ok(encoder.finish()?)
    }
}

/// Gzip-based expansion implementation
#[cfg(feature = "gzip")]
pub struct GzipExpander;

#[cfg(feature = "gzip")]
#[async_trait::async_trait]
impl Expander for GzipExpander {
    async fn expand(
        &'async_trait self,
        compressed: &'async_trait [u8],
    ) -> Result<String, StreamlinerError> {
        let mut decoder = GzDecoder::new(compressed);
        let mut output = String::new();
        decoder
            .read_to_string(&mut output)
            .map_err(|e| StreamlinerError::ExpansionError(e.to_string()))?;
        Ok(output)
    }
}

/// LZ4-based compression implementation
#[cfg(feature = "lz4")]
pub struct Lz4Compressor;

#[cfg(feature = "lz4")]
#[async_trait::async_trait]
impl Compressor for Lz4Compressor {
    async fn compress(
        &'async_trait self,
        context: &'async_trait str,
    ) -> Result<Vec<u8>, StreamlinerError> {
        let mut encoder = FrameEncoder::new(Vec::new());
        encoder.write_all(context.as_bytes())?;
        encoder.finish().map_err(|e| {
            StreamlinerError::CompressionError(std::io::Error::new(std::io::ErrorKind::Other, e))
        })
    }
}

/// LZ4-based expansion implementation
#[cfg(feature = "lz4")]
pub struct Lz4Expander;

#[cfg(feature = "lz4")]
#[async_trait::async_trait]
impl Expander for Lz4Expander {
    async fn expand(
        &'async_trait self,
        compressed: &'async_trait [u8],
    ) -> Result<String, StreamlinerError> {
        let mut decoder = FrameDecoder::new(Cursor::new(compressed));
        let mut output = String::new();
        decoder
            .read_to_string(&mut output)
            .map_err(|e| StreamlinerError::ExpansionError(e.to_string()))?;
        Ok(output)
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

    #[cfg(feature = "gzip")]
    #[tokio::test]
    async fn test_gzip_roundtrip() {
        let compressor = GzipCompressor;
        let expander = GzipExpander;
        let original = "test context";

        let compressed = compressor.compress(original).await.unwrap();
        let expanded = expander.expand(&compressed).await.unwrap();

        assert_eq!(original, expanded);
    }

    #[cfg(feature = "lz4")]
    #[tokio::test]
    async fn test_lz4_roundtrip() {
        let compressor = Lz4Compressor;
        let expander = Lz4Expander;
        let original = "test context";

        let compressed = compressor.compress(original).await.unwrap();
        let expanded = expander.expand(&compressed).await.unwrap();

        assert_eq!(original, expanded);
    }
}
