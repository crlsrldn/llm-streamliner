# LLM-Streamliner Performance Benchmarks

## Compression Performance (Zlib Implementation)
- **Test Data**: 1MB string
- **Average Time**: 1.67ms - 1.77ms
- **Outliers**: 14% (14/100 samples)
- **Recommendation**: Target 8.3s duration for more stable measurements

## Test Environment
- Date: 2025-05-18
- Rust Version: 1.87.0
- Criterion Version: 0.5

## How to Run
```bash
cargo bench --features=benchmarks
```