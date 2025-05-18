# LLM-Streamliner

[![Crates.io](https://img.shields.io/crates/v/llm-streamliner)](https://crates.io/crates/llm-streamliner)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Rust SDK for compressing LLM context into expandable memory modules, solving context window limitations.

## Features

- **Incremental Compression**: Compress large contexts into compact memory modules
- **On-Demand Expansion**: Reconstruct original content when needed
- **Multiple Algorithms**: Support for various compression backends (currently Zlib)
- **Serialization**: Save/load compressed modules
- **Async Ready**: Designed for async workflows

## Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
llm-streamliner = "0.1"
```

## Usage

```rust
use llm_streamliner::{MemoryModule, ZlibCompressor, ZlibExpander};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compress context
    let context = "Your large LLM context here...";
    let compressor = ZlibCompressor;
    let module = MemoryModule::new(context, &compressor).await?;
    
    // Serialize
    let json = module.to_json()?;
    
    // Later... deserialize and expand
    let loaded_module = MemoryModule::from_json(&json)?;
    let expander = ZlibExpander;
    let expanded = loaded_module.expand(&expander).await?;
    
    println!("Original length: {}", context.len());
    println!("Compressed size: {}", json.len());
    println!("Expanded matches original: {}", expanded == context);
    
    Ok(())
}
```

## Compression Algorithms

| Algorithm | Feature Flag | Description |
|-----------|-------------|-------------|
| Zlib      | -           | Default compression (good balance of speed/ratio) |
| Gzip      | gzip        | Higher compression ratio |
| LZ4       | lz4         | Faster compression |

Enable features in Cargo.toml:
```toml
llm-streamliner = { version = "0.1", features = ["gzip"] }
```

## Benchmarks

Coming soon! We'll provide performance metrics for:
- Compression ratios
- Compression/decompression speeds
- Memory overhead

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT - See [LICENSE](LICENSE) for details.