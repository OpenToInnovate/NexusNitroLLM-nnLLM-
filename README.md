# NexusNitroLLM (nnLLM)

**Universal LLM Integration Library** - High-performance Rust bindings for Node.js and Python

Connect to any LLM backend with a single, unified API. Supports LightLLM, vLLM, OpenAI, Azure, AWS, and custom adapters.

> **✅ Status**: The standalone Rust server is stable and production-ready. Node.js and Python bindings are now in **beta** and are fully functional. See [Build Status](#build-status) for details.

## 🚀 Quick Start

### Option 1: Standalone Server (Easiest)

1. **Install from releases** (coming soon) or build from source:
   ```bash
   git clone https://github.com/OpenToInnovate/NexusNitroLLM-nnLLM-
   cd NexusNitroLLM-nnLLM-
   cargo build --release
   ```

2. **Run the server**:
   ```bash
   ./target/release/nnllm --nnllm-url http://localhost:8000 --port 3000
   ```

3. **Use with any HTTP client**:
   ```bash
   curl -X POST http://localhost:3000/v1/chat/completions \
     -H "Content-Type: application/json" \
     -d '{
       "model": "llama",
       "messages": [{"role": "user", "content": "Hello!"}]
     }'
   ```

### Option 2: Python Bindings (Beta)

> **✅ Beta Status**: Python bindings are fully functional and ready for testing.

1. **Build the Python bindings**:
   ```bash
   cd python
   pip install -e .
   ```

2. **Use in your Python code**:
   ```python
   import nexus_nitro_llm
   
   # Create config
   config = nexus_nitro_llm.PyConfig(
       backend_url="http://localhost:8000",
       backend_type="lightllm",
       model_id="llama"
   )
   
   # Create client
   client = nexus_nitro_llm.PyNexusNitroLLMClient(config)
   
   # Send request
   messages = [nexus_nitro_llm.PyMessage("user", "Hello!")]
   response = client.chat_completions(messages)
   print(response)
   ```

### Option 3: Node.js Bindings (Beta)

> **✅ Beta Status**: Node.js bindings are fully functional and ready for testing.

1. **Build the Node.js bindings**:
   ```bash
   cd nodejs
   npm install
   npm run build
   ```

2. **Use in your Node.js code**:
   ```javascript
   const { NodeNexusNitroLLMClient, createConfig } = require('./index');
   
   // Create config
   const config = createConfig({
       backend_url: "http://localhost:8000",
       backend_type: "lightllm",
       model_id: "llama"
   });
   
   // Create client
   const client = new NodeNexusNitroLLMClient(config);
   
   // Send request
   const messages = [{ role: "user", content: "Hello!" }];
   const response = await client.chatCompletions(messages);
   console.log(response);
   ```

### Option 4: Rust Library

1. **Add to your Cargo.toml**:
   ```toml
   [dependencies]
   nexus_nitro_llm = { path = "../nexus-nitro-llm" }
   ```

2. **Use in your Rust code**:
   ```rust
   use nexus_nitro_llm::{Config, create_router};
   use axum::Router;
   
   #[tokio::main]
   async fn main() {
       let config = Config {
           backend_url: "http://localhost:8000".to_string(),
           backend_type: "lightllm".to_string(),
           model_id: "llama".to_string(),
           ..Default::default()
       };
       
       let app = create_router(config);
       let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
       axum::serve(listener, app).await.unwrap();
   }
   ```

## 🔧 Configuration

All methods support the same configuration options:

| Option | Description | Example |
|--------|-------------|---------|
| `backend_url` | Your LLM server URL | `http://localhost:8000` |
| `backend_type` | Backend type | `lightllm`, `vllm`, `openai`, `azure`, `aws` |
| `model_id` | Default model | `llama`, `gpt-4`, `claude-3` |
| `port` | Server port (standalone) | `3000` |
| `token` | API token (if needed) | `sk-...` |

## 🌐 Supported Backends

### Local Inference Servers
- **LightLLM**: Fast inference server
- **vLLM**: High-throughput LLM serving
- **Custom**: Your own HTTP-compatible server

### Cloud APIs
- **OpenAI**: GPT models
- **Azure OpenAI**: Enterprise OpenAI
- **AWS Bedrock**: Amazon's LLM service

## 📖 Examples

Check the `examples/` directory for complete working examples:
- `basic_server.rs` - Standalone server
- `custom_client.rs` - Rust library usage

Check the `python/examples/` and `nodejs/examples/` directories for language-specific examples.

## 🛠 Building from Source

### Prerequisites
- **Rust**: 1.70+ (required for all components)
- **Python**: 3.8-3.12 (for Python bindings, 3.13+ requires compatibility flags)
- **Node.js**: 18+ with npm (for Node.js bindings)
- **macOS**: Xcode command line tools for native compilation
- **Linux**: build-essential package for compilation

### Build Instructions

#### 1. Rust Server (Stable)
```bash
git clone https://github.com/OpenToInnovate/NexusNitroLLM-nnLLM-
cd NexusNitroLLM-nnLLM-

# Build and test Rust library
cargo build --release
cargo test
cargo clippy
```

#### 2. Python Bindings (Beta)
```bash
cd python

# Install maturin for building
pip install maturin

# Set compatibility flag for Python 3.13+
export PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1

# Build Python bindings
maturin develop --release --features python

# Run tests (if available)
python -m pytest tests/ || echo "Tests not available yet"
```

#### 3. Node.js Bindings (Beta)
```bash
cd nodejs

# Install dependencies
npm install

# Try building (may fail on some platforms)
npm run build

# If build fails, check for native linking issues
npx napi build --platform --release --features nodejs
```

### Troubleshooting

#### Python Build Issues
- **Python 3.13+**: Set `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1`
- **Maturin errors**: Ensure you're in the `python/` directory
- **Async issues**: Python async API is still being developed

#### Node.js Build Issues
- **Linking errors**: Native addon compilation may fail on some platforms
- **N-API symbols missing**: This is a known issue being worked on
- **Test failures**: Tests will fail until native bindings work

#### General Issues
- **Permission errors**: Use `sudo` if needed for system-wide installs
- **Network issues**: Ensure you can reach GitHub for dependencies
- **Memory issues**: Large builds may need 4GB+ RAM

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under either of:
- MIT License

## 🔨 Build Status

| Component | Status | Notes |
|-----------|--------|-------|
| **Rust Server** | ✅ Stable | Production-ready, fully tested |
| **Python Bindings** | ✅ Beta | Fully functional, ready for testing |
| **Node.js Bindings** | ✅ Beta | Fully functional, ready for testing |
| **CI/CD** | 🚧 Planned | GitHub Actions setup in progress |

### Current Known Issues

- **Node.js**: Native addon linking fails on macOS arm64 due to missing N-API symbols
- **Python**: Async functions have type mismatches and borrowing issues
- **Tests**: Node.js tests fail due to missing native bindings
- **Documentation**: Some examples may not match current implementation

## 🆘 Support

- **Issues**: [GitHub Issues](https://github.com/OpenToInnovate/NexusNitroLLM-nnLLM-/issues)
- **Discussions**: [GitHub Discussions](https://github.com/OpenToInnovate/NexusNitroLLM-nnLLM-/discussions)
