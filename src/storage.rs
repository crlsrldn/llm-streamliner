//! Persistent storage for compressed memory modules
//!
//! Provides asynchronous file-based storage for serialized memory modules.
//! Uses Tokio's async I/O for efficient operations.
//!
//! # Examples
//! ```
//! use llm_streamliner::storage;
//! use std::path::Path;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), storage::StorageError> {
//!     let path = Path::new("module.dat");
//!     let data = b"test data";
//!     
//!     // Store compressed module
//!     storage::store_module(path, data).await?;
//!     
//!     // Retrieve compressed module
//!     let retrieved = storage::retrieve_module(path).await?;
//!     assert_eq!(data, retrieved.as_slice());
//!     Ok(())
//! }
//! ```

use std::path::Path;
use thiserror::Error;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::MemoryModule;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Stores compressed memory modules to disk
///
/// # Arguments
/// * `path` - Filesystem path where the module should be stored
/// * `data` - Binary data to store (typically compressed memory module)
///
/// # Errors
/// Returns `StorageError` if:
/// - The file cannot be created
/// - Writing fails
///
/// # Example
/// ```no_run
/// use llm_streamliner::storage;
/// use std::path::Path;
///
/// #[tokio::main]
/// async fn main() -> Result<(), storage::StorageError> {
///     storage::store_module(Path::new("module.dat"), b"data").await
/// }
/// ```
pub async fn store_module(path: &Path, data: &[u8]) -> Result<(), StorageError> {
    let mut file = fs::File::create(path).await?;
    file.write_all(data).await?;
    Ok(())
}

/// Retrieves compressed memory modules from disk
///
/// # Arguments
/// * `path` - Filesystem path where the module is stored
///
/// # Returns
/// `Vec<u8>` containing the stored binary data
///
/// # Errors
/// Returns `StorageError` if:
/// - The file cannot be opened
/// - Reading fails
///
/// # Example
/// ```no_run
/// use llm_streamliner::storage;
/// use std::path::Path;
///
/// #[tokio::main]
/// async fn main() -> Result<(), storage::StorageError> {
///     let data = storage::retrieve_module(Path::new("module.dat")).await?;
///     println!("Retrieved {} bytes", data.len());
///     Ok(())
/// }
/// ```
pub async fn retrieve_module(path: &Path) -> Result<Vec<u8>, StorageError> {
    let mut file = fs::File::open(path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;
    Ok(buffer)
}

/// Stores a `MemoryModule` as JSON on disk
///
/// # Arguments
/// * `path` - Filesystem path where the module should be stored
/// * `module` - MemoryModule instance to serialize and store
///
/// # Errors
/// Returns `StorageError` if the file cannot be created or serialization/writing fails.
pub async fn store_memory_module(path: &Path, module: &MemoryModule) -> Result<(), StorageError> {
    let json = module.to_json()?;
    let mut file = fs::File::create(path).await?;
    file.write_all(json.as_bytes()).await?;
    Ok(())
}

/// Loads a `MemoryModule` from JSON stored on disk
///
/// # Arguments
/// * `path` - Filesystem path where the module is stored
///
/// # Errors
/// Returns `StorageError` if the file cannot be opened, read, or deserialized.
pub async fn load_memory_module(path: &Path) -> Result<MemoryModule, StorageError> {
    let mut file = fs::File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    MemoryModule::from_json(&contents).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_storage_roundtrip() {
        let test_data = b"test memory module data";
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        store_module(path, test_data).await.unwrap();
        let retrieved = retrieve_module(path).await.unwrap();

        assert_eq!(test_data.as_slice(), retrieved);
    }

    #[tokio::test]
    async fn test_memory_module_storage_roundtrip() {
        let compressor = crate::ZlibCompressor;
        let expander = crate::ZlibExpander;
        let original = "disk storage module test";

        let module = MemoryModule::new(original, &compressor).await.unwrap();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        store_memory_module(path, &module).await.unwrap();
        let loaded = load_memory_module(path).await.unwrap();

        let expanded = loaded.expand(&expander).await.unwrap();
        assert_eq!(original, expanded);
    }
}