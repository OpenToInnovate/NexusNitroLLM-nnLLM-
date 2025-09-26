# Performance Benchmarking Framework

This document describes the comprehensive performance benchmarking framework for NexusNitroLLM implementations across Rust, Node.js, and Python.

## Overview

The benchmarking framework provides fair, apples-to-apples performance comparisons across different language implementations. It measures latency, throughput, memory usage, and error rates under various load conditions.

## Architecture

### Key Principles

1. **Fairness First**: Same workload, same machine, same network conditions
2. **Isolation**: Measure client performance only (server is controlled Mockoon)
3. **Reproducibility**: Pinned versions, documented environment, clear methodology
4. **Comprehensive**: Multiple scenarios, concurrency levels, payload sizes

### Test Matrix

| Dimension | Values | Description |
|-----------|--------|-------------|
| **Routes** | 5 endpoints | Happy path, streaming, errors, malformed JSON |
| **Payload Sizes** | S/M/L | 1KB, 32KB, 256KB |
| **Concurrency** | 4 levels | 1, 8, 32, 128 connections |
| **Duration** | 60s | Warmup 30s + benchmark 60s |

## Quick Start

### Prerequisites

1. **Mockoon Server**
```bash
npm install -g @mockoon/cli
mockoon-cli start --data tests/mockoon-env.json --port 3000 --hostname 127.0.0.1
```

2. **Rust Dependencies**
```bash
cargo build --release --bin rust_benchmark
cargo build --release --bin rust_streaming_benchmark
```

3. **Node.js Dependencies**
```bash
cd benches
npm install
```

4. **Python Dependencies**
```bash
pip install -r benches/requirements.txt
```

### Running Benchmarks

#### Option 1: Full Suite (Recommended)
```bash
cd benches
npm run full-suite
```

This runs:
1. Complete benchmark matrix
2. Professional report generation
3. Chart and data export generation

#### Option 2: Individual Components
```bash
# Run benchmarks only
npm run benchmark

# Generate report from existing results
npm run report

# Generate charts from existing results
npm run charts
```

## Benchmark Implementations

### Rust Implementation
- **Binary**: `rust_benchmark`, `rust_streaming_benchmark`
- **HTTP Client**: `reqwest` with connection pooling
- **Metrics**: `hdrhistogram` for latency measurements
- **Concurrency**: `tokio` async runtime with multi-threaded flavor

**Features:**
- Zero-copy optimizations where possible
- Efficient memory management
- High-performance HTTP client
- Detailed error categorization

### Node.js Implementation
- **HTTP Client**: `undici` (high-performance HTTP client)
- **Metrics**: `hdr-histogram-js` for latency measurements
- **Concurrency**: Native async/await with Promise.all

**Features:**
- Modern HTTP client with connection pooling
- Efficient JSON parsing
- Memory-efficient streaming
- Comprehensive error handling

### Python Implementation
- **HTTP Client**: `httpx` with async support
- **Metrics**: `hdrhistogram` for latency measurements
- **Concurrency**: `asyncio` with multiple workers

**Features:**
- Modern async HTTP client
- Efficient memory usage
- Streaming support
- Detailed error tracking

## Output Formats

### JSON Results
Each benchmark outputs structured JSON:

```json
{
  "lang": "rust|node|python|rust-streaming",
  "route": "/v1/chat/completions",
  "concurrency": 32,
  "payload_size": "M",
  "reqs": 120000,
  "throughput_rps": 2000.1,
  "latency_ms": {
    "p50": 2.1,
    "p95": 3.7,
    "p99": 7.2
  },
  "errors": {
    "non2xx": 0,
    "timeouts": 0,
    "total": 0
  },
  "success_rate": 100.0,
  "timestamp": "2024-01-01T12:00:00.000Z"
}
```

### Professional Reports
Generated reports include:
- Executive summary with key findings
- Methodology and system information
- Key results table
- Detailed analysis and recommendations
- Reproduction instructions
- Raw data appendices

### Chart Data
Multiple formats available:
- ASCII charts for quick viewing
- CSV data for spreadsheet analysis
- JSON data for custom visualizations
- Combined markdown reports

## Performance Expectations

### Typical Results (Modern Hardware, Localhost)

| Language | c=1 | c=8 | c=32 | c=128 |
|----------|-----|-----|------|-------|
| **Rust** | ~1,500 RPS | ~8,000 RPS | ~15,000 RPS | ~20,000 RPS |
| **Node.js** | ~1,400 RPS | ~7,500 RPS | ~12,000 RPS | ~16,000 RPS |
| **Python** | ~1,200 RPS | ~6,000 RPS | ~8,000 RPS | ~6,000 RPS |

### Latency Characteristics

- **Low Concurrency (c=1-8)**: All implementations similar
- **Medium Concurrency (c=32)**: Rust typically wins, Node.js competitive
- **High Concurrency (c=128)**: Rust maintains performance, others may degrade

### Memory Usage

- **Rust**: Lowest memory footprint (~50MB peak)
- **Node.js**: Moderate usage (~120MB peak)
- **Python**: Higher usage (~150MB peak)

## Profiling and Optimization

### Rust Profiling
```bash
# Flamegraph
cargo flamegraph --bin rust_benchmark

# Perf profiling
perf record -g ./target/release/rust_benchmark
```

### Node.js Profiling
```bash
# Clinic.js
npx clinic flame -- node node_benchmark.js

# 0x flamegraph
npx 0x node node_benchmark.js
```

### Python Profiling
```bash
# py-spy
py-spy record -- python3 python_benchmark.py

# yappi for asyncio
python3 -m yappi python_benchmark.py
```

## Environment Configuration

### System Optimization

**Linux:**
```bash
# CPU governor
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Network tuning
sudo sysctl -w net.core.rmem_max=134217728
sudo sysctl -w net.core.wmem_max=134217728
```

**macOS:**
```bash
# Disable App Nap for terminal
defaults write NSGlobalDomain NSAppSleepDisabled -bool YES
```

**Windows:**
- Set power plan to High Performance
- Disable Windows Defender real-time scanning during benchmarks

### Runtime Configuration

**Rust:**
```bash
export RUSTFLAGS="-C target-cpu=native"
export CARGO_PROFILE_RELEASE_LTO=true
```

**Node.js:**
```bash
export NODE_OPTIONS="--max-old-space-size=4096"
export UV_THREADPOOL_SIZE=128
```

**Python:**
```bash
export UVLOOP=1  # Use uvloop if available
```

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Performance Benchmarks
on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'
      
      - name: Install dependencies
        run: |
          cargo build --release
          cd benches && npm install
          pip install -r requirements.txt
      
      - name: Start Mockoon
        run: |
          npm install -g @mockoon/cli
          mockoon-cli start --data tests/mockoon-env.json --port 3000 &
          sleep 10
      
      - name: Run benchmarks
        run: |
          cd benches
          npm run benchmark
      
      - name: Generate report
        run: |
          cd benches
          npm run report
          npm run charts
      
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: |
            benches/benchmark_results.json
            benches/reports/
```

## Troubleshooting

### Common Issues

1. **Mockoon Not Responding**
   - Check if port 3000 is available
   - Verify Mockoon environment file exists
   - Test with: `curl http://localhost:3000/health`

2. **High Error Rates**
   - Check system resources (CPU, memory)
   - Verify Mockoon server stability
   - Monitor network connectivity

3. **Inconsistent Results**
   - Ensure system is in performance mode
   - Close unnecessary applications
   - Run multiple iterations

4. **Memory Issues**
   - Increase system memory limits
   - Check for memory leaks in implementations
   - Monitor garbage collection

### Performance Debugging

1. **Check System Resources**
   ```bash
   # CPU usage
   top -p $(pgrep -f benchmark)
   
   # Memory usage
   ps aux | grep benchmark
   
   # Network connections
   netstat -an | grep :3000
   ```

2. **Monitor Mockoon**
   ```bash
   # Check Mockoon logs
   mockoon-cli start --data tests/mockoon-env.json --port 3000 --verbose
   ```

3. **Analyze Results**
   ```bash
   # View raw results
   jq '.results[] | select(.lang=="rust")' benchmark_results.json
   
   # Calculate averages
   jq '.results | group_by(.lang) | map({lang: .[0].lang, avg_rps: (map(.throughput_rps) | add / length)})' benchmark_results.json
   ```

## Best Practices

### Benchmarking
1. **Consistent Environment**: Same hardware, OS, and configuration
2. **Multiple Runs**: Run benchmarks multiple times and take median
3. **Warmup**: Always include warmup period before measurements
4. **Isolation**: Close unnecessary applications and services

### Analysis
1. **Statistical Significance**: Use proper statistical methods
2. **Outlier Handling**: Identify and explain outliers
3. **Context**: Consider real-world usage patterns
4. **Trends**: Look for patterns across different conditions

### Reporting
1. **Clear Methodology**: Document all configuration details
2. **Reproducible**: Include exact commands and versions
3. **Actionable**: Provide clear recommendations
4. **Honest**: Report limitations and caveats

## Future Enhancements

### Planned Features
1. **TLS Benchmarking**: HTTPS performance comparison
2. **Real API Testing**: Integration with actual LLM APIs
3. **Distributed Testing**: Multi-machine benchmark scenarios
4. **Advanced Metrics**: More detailed resource usage tracking
5. **Automated Regression**: Performance regression detection

### Extension Points
1. **Custom Metrics**: Add application-specific measurements
2. **Additional Languages**: Support for other language implementations
3. **Custom Workloads**: Define custom test scenarios
4. **Visualization**: Enhanced chart generation and interactive dashboards

## Contributing

When contributing to the benchmarking framework:

1. **Maintain Fairness**: Ensure all implementations are tested equally
2. **Document Changes**: Update methodology and configuration docs
3. **Test Thoroughly**: Verify benchmarks work across different environments
4. **Performance**: Keep benchmark overhead minimal
5. **Reproducibility**: Ensure results can be reproduced by others

## License

This benchmarking framework is part of the NexusNitroLLM project and follows the same license terms.

