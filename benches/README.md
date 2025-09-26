# Performance Benchmarking Framework

This directory contains a comprehensive performance benchmarking suite for NexusNitroLLM implementations across Rust, Node.js, and Python. The framework provides fair, apples-to-apples comparisons against Mockoon server endpoints.

## Overview

The benchmarking framework measures:
- **Latency**: p50, p95, p99 request round-trip times
- **Throughput**: requests per second at various concurrency levels
- **Memory**: peak RSS usage of client processes
- **Error rates**: non-2xx responses, timeouts, retries
- **Streaming**: SSE bytes/sec and time-to-first-byte

## Architecture

### Ground Rules (Fairness First)
- Same workload: identical endpoints, payloads, concurrency, duration
- Same machine: single host, performance mode, stable network loopback
- Warmup: 30-60s warmup per test; discard warmup results
- Multiple runs: ≥5 independent runs; report median + MAD or p50/p95/p99
- Isolation: measure client process only (server is Mockoon with fixed latency)
- Observability: JSON output for aggregation

### Test Matrix

**Routes:**
- `POST /v1/chat/completions` - 200 JSON happy path
- `POST /v1/chat/completions:stream` - Streaming SSE
- `POST /v1/chat/completions:rate` - 429 + Retry-After
- `POST /v1/chat/completions:error` - 500 error
- `POST /v1/chat/completions:malformed` - Malformed JSON

**Payload Sizes:**
- S: 1KB (~50 repeated "Hello, world!")
- M: 32KB (~1600 repeated "Hello, world!")
- L: 256KB (~12800 repeated "Hello, world!")

**Concurrency Levels:**
- 1, 8, 32, 128 concurrent connections

**Durations:**
- Warmup: 30s
- Benchmark: 60s per test point

## Quick Start

### Prerequisites

1. **Mockoon Server**: Start Mockoon with the test environment
```bash
# Install Mockoon CLI
npm install -g @mockoon/cli

# Start the test server
mockoon-cli start --data ../tests/mockoon-env.json --port 3000 --hostname 127.0.0.1
```

2. **Rust Dependencies**: The benchmark binaries are included in the main Cargo.toml
```bash
cargo build --release --bin rust_benchmark
cargo build --release --bin rust_streaming_benchmark
```

3. **Node.js Dependencies**:
```bash
cd benches
npm install
```

4. **Python Dependencies**:
```bash
pip install -r requirements.txt
```

### Running Benchmarks

#### Option 1: Full Suite (Recommended)
```bash
cd benches
node benchmark_controller.js
```

This runs the complete test matrix and generates a comprehensive report.

#### Option 2: Individual Benchmarks

**Rust:**
```bash
# Basic benchmark
BASE=http://localhost:3000 C=32 T=60 cargo run --release --bin rust_benchmark -- /v1/chat/completions

# Streaming benchmark
BASE=http://localhost:3000 C=32 T=60 cargo run --release --bin rust_streaming_benchmark -- /v1/chat/completions:stream
```

**Node.js:**
```bash
# Basic benchmark
BASE=http://localhost:3000 C=32 T=60 node node_benchmark.js /v1/chat/completions

# With different payload sizes
SIZE=M BASE=http://localhost:3000 C=32 T=60 node node_benchmark.js /v1/chat/completions
```

**Python:**
```bash
# Basic benchmark
BASE=http://localhost:3000 C=32 T=60 python3 python_benchmark.py

# With different routes
ROUTE=/v1/chat/completions:stream BASE=http://localhost:3000 C=32 T=60 python3 python_benchmark.py
```

## Output Format

Each benchmark outputs a JSON summary:

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
  "timestamp": "2024-01-01T12:00:00.000Z",
  "test_config": {
    "route": "/v1/chat/completions",
    "concurrency": 32,
    "payload_size": "M",
    "duration_seconds": 60,
    "mockoon_base": "http://localhost:3000"
  }
}
```

For streaming benchmarks, additional fields:
```json
{
  "sse_bytes_per_sec": 150000.0,
  "time_to_first_ms": {
    "p50": 1.2,
    "p95": 2.1,
    "p99": 3.5
  },
  "total_bytes": 9000000
}
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `BASE` | `http://localhost:3000` | Mockoon server base URL |
| `ROUTE` | `/v1/chat/completions` | Endpoint to benchmark |
| `C` | `32` | Concurrency level |
| `T` | `60` | Test duration in seconds |
| `SIZE` | `S` | Payload size (S/M/L) |
| `MOCKOON_BASE` | `http://localhost:3000` | Controller: Mockoon URL |
| `WARMUP_SECONDS` | `30` | Controller: Warmup duration |
| `BENCHMARK_SECONDS` | `60` | Controller: Benchmark duration |
| `RESULTS_FILE` | `benchmark_results.json` | Controller: Output file |

## Performance Expectations

### Typical Results (localhost, modern hardware)

**Low Concurrency (c=1-8):**
- All implementations should be similar
- Latency dominated by network I/O
- Throughput: 500-2000 RPS

**Medium Concurrency (c=32):**
- Rust typically wins throughput and p99
- Node.js competitive with slightly higher p99 due to GC
- Python close but may fall behind without uvloop

**High Concurrency (c=128):**
- Rust maintains best performance
- Node.js may show GC pauses
- Python performance degrades significantly

### Streaming Performance
- Time-to-first-byte: 1-5ms
- SSE throughput: 50-200 KB/s per connection
- Memory usage increases with concurrent streams

## Profiling and Optimization

### Rust
```bash
# Generate flamegraph
cargo flamegraph --bin rust_benchmark

# Profile with perf
perf record -g ./target/release/rust_benchmark
```

### Node.js
```bash
# Clinic.js profiling
npx clinic flame -- node node_benchmark.js

# 0x flamegraph
npx 0x node node_benchmark.js
```

### Python
```bash
# py-spy profiling
py-spy record -- python3 python_benchmark.py

# yappi for asyncio
python3 -m yappi python_benchmark.py
```

## Troubleshooting

### Common Issues

1. **Mockoon not responding**
   - Check if Mockoon is running: `curl http://localhost:3000/health`
   - Verify port 3000 is not in use
   - Check Mockoon environment file exists

2. **Benchmark timeouts**
   - Increase timeout values in client configurations
   - Check system resources (CPU, memory)
   - Verify network connectivity

3. **High error rates**
   - Check Mockoon server logs
   - Verify endpoint configurations in mockoon-env.json
   - Monitor system resources during benchmark

4. **Inconsistent results**
   - Ensure system is in performance mode
   - Close unnecessary applications
   - Run multiple iterations and take median

### Performance Tips

1. **System Optimization**
   ```bash
   # Linux: Set CPU governor to performance
   echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
   
   # macOS: Disable App Nap for terminal
   # Windows: Set power plan to High Performance
   ```

2. **Network Optimization**
   - Use localhost/loopback interface
   - Disable network monitoring tools
   - Ensure adequate TCP buffer sizes

3. **Client Optimization**
   - Pre-build HTTP clients (connection pooling)
   - Use appropriate timeout values
   - Monitor memory usage patterns

## Integration with CI/CD

The benchmark framework can be integrated into CI/CD pipelines:

```yaml
# .github/workflows/performance.yml
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
          node benchmark_controller.js
      
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: benches/benchmark_results.json
```

## Contributing

When adding new benchmark scenarios:

1. Update the test matrix in `benchmark_controller.js`
2. Add corresponding routes to `mockoon-env.json`
3. Update this documentation
4. Ensure all implementations support the new scenario

## License

This benchmarking framework is part of the NexusNitroLLM project and follows the same license terms.




