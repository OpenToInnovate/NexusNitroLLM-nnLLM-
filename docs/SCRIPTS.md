# Scripts Directory

This directory contains essential scripts for running and testing NexusNitroLLM.

## Available Scripts

### 🚀 `run.sh`
Main script to run the NexusNitroLLM server.
- Loads environment variables from `.env` file
- Runs the server in release mode
- Usage: `./run.sh`

### 🔧 `setup_env.sh`
Interactive environment setup script for secure configuration.
- Helps users set up environment variables securely
- Validates configuration values
- Creates `.env` file with proper settings
- Usage: `./setup_env.sh`

### 📊 `benchmark.sh`
Performance benchmarking script for testing proxy performance.
- Configurable via environment variables
- Measures response times and throughput
- Supports authentication tokens
- Usage: `./benchmark.sh` or `TOKEN=your-token MODEL=llama ./benchmark.sh`

### 🌊 `demo_streaming.sh`
Demonstration script for streaming functionality.
- Shows Server-Sent Events (SSE) support
- Tests OpenAI-compatible streaming format
- Uses mock endpoints for safe testing
- Usage: `./demo_streaming.sh`

## Security Notes

- **No hardcoded credentials**: All scripts use environment variables or command-line arguments
- **Safe defaults**: Scripts use localhost and mock endpoints by default
- **Token handling**: Sensitive tokens should be passed via environment variables
- **Environment files**: Use `.env` files for local development (not committed to git)

## Usage Examples

### Running the Server
```bash
# Using environment variables
export nnLLM_URL="https://your-backend.com"
export nnLLM_TOKEN="your-token"
./run.sh

# Or using setup script first
./setup_env.sh
./run.sh
```

### Benchmarking Performance
```bash
# Basic benchmark
./benchmark.sh

# With custom settings
PROXY_URL="http://localhost:8080" \
TOKEN="your-token" \
MODEL="llama" \
REQUESTS=20 \
./benchmark.sh
```

### Testing Streaming
```bash
# Run streaming demo
./demo_streaming.sh
```

## Environment Variables

Common environment variables used by the scripts:

- `nnLLM_URL`: Backend LLM server URL
- `nnLLM_TOKEN`: Authentication token
- `nnLLM_MODEL`: Default model ID
- `PROXY_URL`: Proxy server URL (for testing)
- `RUST_LOG`: Logging level (debug, info, warn, error)

## Best Practices

1. **Never commit credentials**: Use `.env` files or environment variables
2. **Use secure tokens**: Generate proper API keys for production
3. **Test locally first**: Use mock endpoints for initial testing
4. **Monitor performance**: Use benchmark script to measure improvements
5. **Check logs**: Use appropriate log levels for debugging
