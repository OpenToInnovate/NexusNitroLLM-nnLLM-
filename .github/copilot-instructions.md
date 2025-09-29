# NexusNitroLLM (nnLLM) - AI Assistant Instructions

## Architecture Overview

This is a **multi-target Rust library** providing universal LLM integration with OpenAI-compatible APIs. The project builds 4 artifacts:
- **Standalone server** (`nnllm` binary) - Production-ready
- **Rust library** (`nexus_nitro_llm` crate) - Core functionality  
- **Python bindings** (via PyO3/Maturin) - Alpha status
- **Node.js bindings** (via NAPI-RS) - Alpha status

### Core Components

- `src/adapters/` - Universal backend adapters (LightLLM, vLLM, OpenAI, Azure, AWS)
- `src/server/` - Axum-based HTTP server with handlers, state management, middleware
- `src/config.rs` - Comprehensive configuration via CLI args, env vars, and .env files
- `src/streaming/` - Server-Sent Events (SSE) streaming support
- `examples/` - Working examples for each use case

## Key Patterns

### Feature-Gated Architecture
Uses Cargo features extensively for conditional compilation:
```rust
#[cfg(feature = "streaming")]
pub mod streaming;

#[cfg(feature = "server")]
pub use server::{AppState, create_router};
```
Default features: `["server", "streaming", "tools", "caching", "metrics", "cli"]`

### Adapter Pattern
Central `Adapter` enum in `src/adapters/mod.rs` with auto-detection:
```rust
pub enum Adapter {
    LightLLM(LightLLMAdapter),
    VLLM(VLLMAdapter), 
    OpenAI(OpenAIAdapter),
    // ... others
}
```
Adapters auto-select based on `backend_type` config or URL pattern matching.

### State Management
`AppState` in `src/server/state.rs` holds shared state (config, adapter, HTTP client):
```rust
pub struct AppState {
    pub config: Config,
    pub adapter: Adapter,
    pub streaming_handler: StreamingHandler,
    pub http_client: reqwest::Client,
}
```

## Development Workflows

### Building & Testing
```bash
# Core Rust library (stable)
cargo build --release
cargo test
cargo run --bin nnllm -- --help

# Python bindings (alpha - build issues)
cd python && maturin develop --release --features python

# Node.js bindings (alpha - linking issues) 
cd nodejs && npm install && npm run build

# Comprehensive test runner
cargo test --test comprehensive_test_runner
```

### Running the Server
```bash
# Using binary
./target/release/nnllm --nnllm-url http://localhost:8000 --port 3000

# Using scripts
./scripts/setup_env.sh  # Interactive .env setup
./run_server_persistent.sh  # Background server with logging
```

### Configuration Precedence
1. CLI arguments (`--nnllm-url`, `--port`, etc.)
2. Environment variables (`nnLLM_URL`, `PORT`, etc.) 
3. .env file (created from `env.example`)
4. Default values in `src/config.rs`

## Testing Strategy

### Test Structure
- `tests/` - Integration tests with Mockoon mock servers
- `benches/` - Performance benchmarks across all language bindings
- `examples/` - Working examples that serve as integration tests

### Key Test Commands
```bash
# Run all Rust tests
cargo test

# Benchmark controller (orchestrates Rust/Node.js/Python)
node benches/benchmark_controller.js

# Comprehensive test runner
cargo test --test comprehensive_test_runner --features "server,streaming"
```

## Critical Implementation Details

### HTTP Client Factory
Centralized in `src/core/http_client.rs` with connection pooling:
```rust
HttpClientBuilder::from_config(&config)
    .build()
    .unwrap()
```

### Error Handling
Custom `ProxyError` enum in `src/error.rs` with structured error responses.

### Streaming Implementation
SSE streaming in `src/streaming/` using `create_streaming_response()` function.

### Multi-Language Bindings
- Python: PyO3 bindings in `src/python.rs` (alpha - async issues)
- Node.js: NAPI-RS bindings in `src/nodejs.rs` (alpha - linking issues)

## Current Issues & Status

- **Rust Server**: ✅ Production-ready, fully functional
- **Python Bindings**: ⚠️ Alpha - build/async issues  
- **Node.js Bindings**: ⚠️ Alpha - N-API linking problems
- See `README.md` "Build Status" section for current limitations

When modifying the codebase, ensure feature gates are properly used and consider impact on all 4 build targets.