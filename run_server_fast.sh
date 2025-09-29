#!/bin/bash

# Fast NexusNitroLLM Server Runner - STREAMING ONLY MODE
# This script ensures the server always runs in fast streaming mode
# with optimized settings for maximum performance.

set -e

echo "🚀 Starting NexusNitroLLM in FAST STREAMING MODE"
echo "   - All requests will use streaming for optimal performance"
echo "   - Non-streaming mode is disabled to prevent slow responses"
echo "   - Optimized for real-time Claude interactions"
echo ""

# Configuration for maximum performance
export RUST_LOG=info
export ENABLE_STREAMING=true
export FORCE_STREAMING_MODE=true
export HTTP_CLIENT_TIMEOUT=60
export HTTP_CLIENT_MAX_CONNECTIONS=50
export HTTP_CLIENT_MAX_CONNECTIONS_PER_HOST=20
export STREAMING_CHUNK_SIZE=512
export STREAMING_TIMEOUT=120
export STREAMING_KEEP_ALIVE_INTERVAL=30

# Build if needed
if [ ! -f "./target/release/nnllm" ]; then
    echo "🔧 Building optimized release binary..."
    cargo build --release
fi

# Kill any existing server
pkill nnllm 2>/dev/null || true
sleep 1

# Start the server with performance monitoring
echo "🚀 Starting server on port 8080..."
echo "📊 Performance mode: STREAMING ONLY"
echo "🔗 Backend: Auto-detected from URL"
echo "⚡ Optimized for sub-second response times"
echo ""

# Run with performance monitoring
exec ./target/release/nnllm --port 8080