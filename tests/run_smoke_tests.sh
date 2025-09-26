#!/bin/bash
# Smoke Test Runner
# 
# Runs deadline-driven, cancellation-aware smoke tests across all implementations
# to catch big problems quickly and in a focused manner.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
MOCKOON_URL="http://localhost:3000"
MOCKOON_PID=""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if Mockoon is running
check_mockoon() {
    if curl -s -f "$MOCKOON_URL/health" > /dev/null 2>&1; then
        log_success "Mockoon is running at $MOCKOON_URL"
        return 0
    else
        log_warning "Mockoon not responding at $MOCKOON_URL"
        return 1
    fi
}

# Start Mockoon if not running
start_mockoon() {
    log_info "Starting Mockoon server..."
    
    # Check if Mockoon CLI is installed
    if ! command -v mockoon-cli &> /dev/null; then
        log_error "Mockoon CLI not found. Install with: npm install -g @mockoon/cli"
        exit 1
    fi
    
    # Start Mockoon in background
    mockoon-cli start \
        --data "$PROJECT_ROOT/tests/mockoon-env.json" \
        --port 3000 \
        --hostname 127.0.0.1 \
        --daemon-off &
    
    MOCKOON_PID=$!
    
    # Wait for Mockoon to start
    log_info "Waiting for Mockoon to start..."
    for i in {1..30}; do
        if check_mockoon; then
            log_success "Mockoon started successfully (PID: $MOCKOON_PID)"
            return 0
        fi
        sleep 1
    done
    
    log_error "Failed to start Mockoon after 30 seconds"
    exit 1
}

# Stop Mockoon
stop_mockoon() {
    if [ -n "$MOCKOON_PID" ]; then
        log_info "Stopping Mockoon (PID: $MOCKOON_PID)..."
        kill "$MOCKOON_PID" 2>/dev/null || true
        wait "$MOCKOON_PID" 2>/dev/null || true
        log_success "Mockoon stopped"
    fi
}

# Run Rust smoke tests
run_rust_smoke_tests() {
    log_info "Running Rust smoke tests..."
    
    cd "$PROJECT_ROOT"
    
    if ! cargo check --test smoke_tests 2>/dev/null; then
        log_warning "Rust smoke tests not available (compilation failed)"
        return 1
    fi
    
    if cargo test --test smoke_tests -- --nocapture 2>/dev/null; then
        log_success "Rust smoke tests passed"
        return 0
    else
        log_error "Rust smoke tests failed"
        return 1
    fi
}

# Run Node.js smoke tests
run_node_smoke_tests() {
    log_info "Running Node.js smoke tests..."
    
    cd "$SCRIPT_DIR"
    
    if ! command -v node &> /dev/null; then
        log_warning "Node.js not found, skipping Node.js smoke tests"
        return 1
    fi
    
    if ! [ -f "smoke_tests_node.js" ]; then
        log_warning "Node.js smoke tests not found"
        return 1
    fi
    
    if node smoke_tests_node.js 2>/dev/null; then
        log_success "Node.js smoke tests passed"
        return 0
    else
        log_error "Node.js smoke tests failed"
        return 1
    fi
}

# Run Python smoke tests
run_python_smoke_tests() {
    log_info "Running Python smoke tests..."
    
    cd "$SCRIPT_DIR"
    
    if ! command -v python3 &> /dev/null; then
        log_warning "Python3 not found, skipping Python smoke tests"
        return 1
    fi
    
    if ! [ -f "smoke_tests_python.py" ]; then
        log_warning "Python smoke tests not found"
        return 1
    fi
    
    # Check if required packages are installed
    if ! python3 -c "import httpx, asyncio" 2>/dev/null; then
        log_warning "Required Python packages not installed (httpx, asyncio)"
        return 1
    fi
    
    if python3 smoke_tests_python.py 2>/dev/null; then
        log_success "Python smoke tests passed"
        return 0
    else
        log_error "Python smoke tests failed"
        return 1
    fi
}

# Run quick connectivity test
run_connectivity_test() {
    log_info "Running connectivity test..."
    
    local endpoints=(
        "/health"
        "/v1/chat/completions"
        "/v1/chat/completions:stream"
        "/v1/chat/completions:rate"
        "/v1/chat/completions:error"
        "/v1/chat/completions:malformed"
    )
    
    local failed=0
    
    for endpoint in "${endpoints[@]}"; do
        if curl -s -f "$MOCKOON_URL$endpoint" > /dev/null 2>&1; then
            log_success "Endpoint $endpoint is responding"
        else
            log_error "Endpoint $endpoint is not responding"
            failed=1
        fi
    done
    
    if [ $failed -eq 0 ]; then
        log_success "All endpoints are responding"
        return 0
    else
        log_error "Some endpoints are not responding"
        return 1
    fi
}

# Generate smoke test report
generate_report() {
    local results_file="$PROJECT_ROOT/tests/smoke_test_results.json"
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    log_info "Generating smoke test report..."
    
    cat > "$results_file" << EOF
{
  "timestamp": "$timestamp",
  "mockoon_url": "$MOCKOON_URL",
  "test_results": {
    "connectivity": $([ $1 -eq 0 ] && echo "true" || echo "false"),
    "rust": $([ $2 -eq 0 ] && echo "true" || echo "false"),
    "nodejs": $([ $3 -eq 0 ] && echo "true" || echo "false"),
    "python": $([ $4 -eq 0 ] && echo "true" || echo "false")
  },
  "summary": {
    "total_tests": 4,
    "passed": $(( $1 == 0 ? 1 : 0 + $2 == 0 ? 1 : 0 + $3 == 0 ? 1 : 0 + $4 == 0 ? 1 : 0 )),
    "failed": $(( $1 != 0 ? 1 : 0 + $2 != 0 ? 1 : 0 + $3 != 0 ? 1 : 0 + $4 != 0 ? 1 : 0 ))
  }
}
EOF
    
    log_success "Report saved to $results_file"
}

# Main execution
main() {
    log_info "🚀 Starting smoke test suite..."
    log_info "Project root: $PROJECT_ROOT"
    
    # Check if Mockoon is running, start if needed
    if ! check_mockoon; then
        start_mockoon
    fi
    
    # Set up cleanup trap
    trap 'stop_mockoon' EXIT
    
    # Run tests
    local connectivity_result=0
    local rust_result=1
    local node_result=1
    local python_result=1
    
    # Always run connectivity test first
    run_connectivity_test
    connectivity_result=$?
    
    # Run implementation tests
    run_rust_smoke_tests
    rust_result=$?
    
    run_node_smoke_tests
    node_result=$?
    
    run_python_smoke_tests
    python_result=$?
    
    # Generate report
    generate_report $connectivity_result $rust_result $node_result $python_result
    
    # Summary
    echo
    log_info "📊 Smoke Test Summary:"
    echo "  Connectivity: $([ $connectivity_result -eq 0 ] && log_success "PASS" || log_error "FAIL")"
    echo "  Rust:         $([ $rust_result -eq 0 ] && log_success "PASS" || log_error "FAIL")"
    echo "  Node.js:      $([ $node_result -eq 0 ] && log_success "PASS" || log_error "FAIL")"
    echo "  Python:       $([ $python_result -eq 0 ] && log_success "PASS" || log_error "FAIL")"
    
    # Exit with error if any critical tests failed
    local total_failed=$(( $connectivity_result + $rust_result + $node_result + $python_result ))
    
    if [ $total_failed -eq 0 ]; then
        log_success "🎉 All smoke tests passed!"
        exit 0
    else
        log_error "💥 Some smoke tests failed ($total_failed failed)"
        exit 1
    fi
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [--help|--connectivity-only|--rust-only|--node-only|--python-only]"
        echo
        echo "Options:"
        echo "  --help              Show this help message"
        echo "  --connectivity-only Run only connectivity tests"
        echo "  --rust-only         Run only Rust smoke tests"
        echo "  --node-only         Run only Node.js smoke tests"
        echo "  --python-only       Run only Python smoke tests"
        echo
        echo "Examples:"
        echo "  $0                  # Run all smoke tests"
        echo "  $0 --rust-only      # Run only Rust tests"
        echo "  $0 --connectivity-only  # Just check if Mockoon is responding"
        exit 0
        ;;
    --connectivity-only)
        check_mockoon && run_connectivity_test
        exit $?
        ;;
    --rust-only)
        check_mockoon && run_rust_smoke_tests
        exit $?
        ;;
    --node-only)
        check_mockoon && run_node_smoke_tests
        exit $?
        ;;
    --python-only)
        check_mockoon && run_python_smoke_tests
        exit $?
        ;;
    *)
        main
        ;;
esac

