#!/usr/bin/env bash
set -euo pipefail

echo "🚀 LightLLM Rust Proxy Performance Benchmark"
echo "============================================="
echo ""

# Configuration - Set these via environment variables or command line
PROXY_URL=${PROXY_URL:-"http://localhost:8080"}
TOKEN=${TOKEN:-""}
MODEL=${MODEL:-"llama"}
REQUESTS=${REQUESTS:-10}

echo "📊 Benchmark Configuration:"
echo "  - Proxy URL: $PROXY_URL"
echo "  - Model: $MODEL"
echo "  - Requests: $REQUESTS"
if [ -n "$TOKEN" ]; then
    echo "  - Token: ${TOKEN:0:20}..."
else
    echo "  - Token: (not set - will use no authentication)"
fi
echo ""

# Test payload
PAYLOAD='{
  "model": "'$MODEL'",
  "messages": [
    {
      "role": "user",
      "content": "What is the capital of Japan? Please be concise."
    }
  ],
  "max_tokens": 30,
  "temperature": 0.7
}'

echo "🧪 Running performance benchmark..."
echo ""

# Function to make a single request and measure time
make_request() {
    local start_time=$(date +%s.%N)
    local curl_cmd="curl -s -w \"\n%{time_total}\" -X POST -H \"Content-Type: application/json\""
    
    if [ -n "$TOKEN" ]; then
        curl_cmd="$curl_cmd -H \"Authorization: Bearer $TOKEN\""
    fi
    
    local response=$(eval "$curl_cmd \"$PROXY_URL/v1/chat/completions\" -d \"$PAYLOAD\"")
    local end_time=$(date +%s.%N)
    
    local http_time=$(echo "$response" | tail -n1)
    local response_body=$(echo "$response" | head -n -1)
    
    # Extract response time from curl
    echo "$http_time"
}

# Run benchmark
echo "Making $REQUESTS requests..."
times=()
for i in $(seq 1 $REQUESTS); do
    echo -n "Request $i/$REQUESTS... "
    time_taken=$(make_request)
    times+=($time_taken)
    echo "${time_taken}s"
done

echo ""
echo "📈 Performance Results:"
echo "======================"

# Calculate statistics
total_time=0
for time in "${times[@]}"; do
    total_time=$(echo "$total_time + $time" | bc -l)
done

avg_time=$(echo "scale=3; $total_time / $REQUESTS" | bc -l)
min_time=$(printf '%s\n' "${times[@]}" | sort -n | head -n1)
max_time=$(printf '%s\n' "${times[@]}" | sort -n | tail -n1)

echo "  Total requests: $REQUESTS"
echo "  Average response time: ${avg_time}s"
echo "  Fastest response: ${min_time}s"
echo "  Slowest response: ${max_time}s"
echo "  Total time: ${total_time}s"
echo ""

# Calculate throughput
throughput=$(echo "scale=2; $REQUESTS / $total_time" | bc -l)
echo "  Throughput: ${throughput} requests/second"
echo ""

echo "✅ Benchmark completed!"
echo ""
echo "🔧 Optimizations Applied:"
echo "  - HTTP/2 connection pooling"
echo "  - Gzip/Brotli compression"
echo "  - Pre-allocated string buffers"
echo "  - Zero-copy JSON parsing"
echo "  - Shared HTTP client instances"
echo "  - Response compression middleware"
echo ""
echo "📝 Performance Tips:"
echo "  - Use HTTP/2 for better multiplexing"
echo "  - Enable compression for large responses"
echo "  - Connection pooling reduces latency"
echo "  - Pre-allocation reduces GC pressure"
