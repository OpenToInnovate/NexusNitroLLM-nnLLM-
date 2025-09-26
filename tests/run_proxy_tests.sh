#!/bin/bash

# Proxy Functionality Test Runner
# This script runs comprehensive proxy tests using Mockoon CLI

set -e

echo "🚀 Starting NexusNitroLLM Proxy Functionality Tests"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Mockoon CLI is installed
check_mockoon() {
    print_status "Checking Mockoon CLI installation..."
    if ! command -v mockoon-cli &> /dev/null; then
        print_error "Mockoon CLI is not installed. Installing now..."
        npm install -g @mockoon/cli
        if [ $? -ne 0 ]; then
            print_error "Failed to install Mockoon CLI"
            exit 1
        fi
    fi
    print_success "Mockoon CLI is available"
}

# Check if test environment file exists
check_test_env() {
    print_status "Checking test environment file..."
    if [ ! -f "tests/mockoon-env.json" ]; then
        print_error "Test environment file not found: tests/mockoon-env.json"
        exit 1
    fi
    print_success "Test environment file found"
}

# Start Mockoon server
start_mockoon() {
    print_status "Starting Mockoon server..."
    
    # Kill any existing Mockoon processes
    pkill -f "mockoon-cli" || true
    sleep 2
    
    # Start Mockoon server in background
    mockoon-cli start --data tests/mockoon-env.json --port 3000 --hostname 127.0.0.1 &
    MOCKOON_PID=$!
    
    # Wait for server to be ready
    print_status "Waiting for Mockoon server to be ready..."
    for i in {1..30}; do
        if curl -s http://127.0.0.1:3000/health > /dev/null 2>&1; then
            print_success "Mockoon server is ready"
            return 0
        fi
        sleep 1
    done
    
    print_error "Mockoon server failed to start within 30 seconds"
    kill $MOCKOON_PID 2>/dev/null || true
    exit 1
}

# Stop Mockoon server
stop_mockoon() {
    print_status "Stopping Mockoon server..."
    pkill -f "mockoon-cli" || true
    print_success "Mockoon server stopped"
}

# Run Rust tests
run_rust_tests() {
    print_status "Running Rust proxy functionality tests..."
    
    # Run the specific test suite
    cargo test --test proxy_functionality_tests --features server,streaming
    
    if [ $? -eq 0 ]; then
        print_success "Rust proxy tests passed"
    else
        print_error "Rust proxy tests failed"
        return 1
    fi
}

# Run basic integration tests
run_integration_tests() {
    print_status "Running basic integration tests..."
    
    cargo test --test integration_tests --features server,streaming
    
    if [ $? -eq 0 ]; then
        print_success "Integration tests passed"
    else
        print_error "Integration tests failed"
        return 1
    fi
}

# Test direct API calls to Mockoon
test_mockoon_directly() {
    print_status "Testing Mockoon server directly..."
    
    # Test health endpoint
    if curl -s http://127.0.0.1:3000/health | grep -q "ok"; then
        print_success "Health endpoint working"
    else
        print_error "Health endpoint failed"
        return 1
    fi
    
    # Test models endpoint
    if curl -s http://127.0.0.1:3000/v1/models | grep -q "gpt-3.5-turbo"; then
        print_success "Models endpoint working"
    else
        print_error "Models endpoint failed"
        return 1
    fi
    
    # Test chat completions endpoint
    CHAT_RESPONSE=$(curl -s -X POST http://127.0.0.1:3000/v1/chat/completions \
        -H "Content-Type: application/json" \
        -d '{"model":"gpt-3.5-turbo","messages":[{"role":"user","content":"Hello"}],"max_tokens":10}')
    
    if echo "$CHAT_RESPONSE" | grep -q "chatcmpl-"; then
        print_success "Chat completions endpoint working"
    else
        print_error "Chat completions endpoint failed"
        return 1
    fi
}

# Run performance tests
run_performance_tests() {
    print_status "Running performance tests..."
    
    # Test concurrent requests
    print_status "Testing concurrent requests..."
    
    # Start 10 concurrent requests
    for i in {1..10}; do
        (
            curl -s -X POST http://127.0.0.1:3000/v1/chat/completions \
                -H "Content-Type: application/json" \
                -d '{"model":"gpt-3.5-turbo","messages":[{"role":"user","content":"Test"}],"max_tokens":10}' \
                > /dev/null 2>&1
        ) &
    done
    
    # Wait for all requests to complete
    wait
    
    print_success "Concurrent requests completed"
}

# Cleanup function
cleanup() {
    print_status "Cleaning up..."
    stop_mockoon
    exit 0
}

# Set up signal handlers
trap cleanup EXIT INT TERM

# Main execution
main() {
    echo "Starting comprehensive proxy functionality tests..."
    echo ""
    
    # Pre-flight checks
    check_mockoon
    check_test_env
    
    # Start Mockoon server
    start_mockoon
    
    # Test Mockoon directly first
    test_mockoon_directly
    
    # Run performance tests
    run_performance_tests
    
    # Run Rust tests
    run_rust_tests
    
    # Run integration tests
    run_integration_tests
    
    echo ""
    echo "=================================================="
    print_success "All proxy functionality tests completed successfully!"
    echo "=================================================="
}

# Run main function
main "$@"