#!/usr/bin/env bash
set -euo pipefail

# LightLLM Rust Proxy Environment Setup Script
# This script helps users set up their environment variables securely

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}[$(date '+%H:%M:%S')] ${message}${NC}"
}

# Function to check if .env file exists
check_env_file() {
    if [ -f ".env" ]; then
        print_status $YELLOW "⚠️  .env file already exists"
        read -p "Do you want to overwrite it? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_status $BLUE "Keeping existing .env file"
            return 1
        fi
    fi
    return 0
}

# Function to create .env file from template
create_env_file() {
    print_status $BLUE "Creating .env file from template..."
    
    if [ -f "env.example" ]; then
        cp env.example .env
        print_status $GREEN "✅ .env file created from env.example"
    else
        print_status $RED "❌ env.example file not found"
        return 1
    fi
}

# Function to prompt for configuration values
prompt_for_config() {
    print_status $BLUE "🔧 Setting up configuration..."
    echo
    
    # Core Configuration
    echo "=== CORE CONFIGURATION ==="
    
    # Port
    read -p "Server port (default: 8080): " port
    port=${port:-8080}
    sed -i.bak "s/PORT=8080/PORT=$port/" .env
    
    # Host
    read -p "Server host (default: 0.0.0.0): " host
    host=${host:-0.0.0.0}
    sed -i.bak "s/HOST=0.0.0.0/HOST=$host/" .env
    
    # LightLLM URL
    read -p "LightLLM backend URL (default: http://localhost:8000): " lightllm_url
    lightllm_url=${lightllm_url:-http://localhost:8000}
    sed -i.bak "s|nnLLM_URL=http://localhost:8000|nnLLM_URL=$lightllm_url|" .env
    
    # Model ID
    read -p "Default model ID (default: llama): " model_id
    model_id=${model_id:-llama}
    sed -i.bak "s/nnLLM_MODEL=llama/nnLLM_MODEL=$model_id/" .env
    
    echo
    echo "=== AUTHENTICATION ==="
    
    # Check if using LiteLLM proxy
    if [[ "$lightllm_url" == *"/v1/"* ]]; then
        print_status $YELLOW "Detected LiteLLM proxy URL. You'll need a virtual key (sk-...)"
        echo "Options:"
        echo "1. Generate a virtual key using the admin token"
        echo "2. Use an existing virtual key"
        echo "3. Skip for now (you can set nnLLM_TOKEN later)"
        read -p "Choose option (1/2/3): " auth_option
        
        case $auth_option in
            1)
                echo "You'll need to run ./scripts/generate_virtual_key.sh after setup"
                ;;
            2)
                read -p "Enter your virtual key (sk-...): " virtual_key
                if [[ "$virtual_key" == sk-* ]]; then
                    sed -i.bak "s/nnLLM_TOKEN=/nnLLM_TOKEN=$virtual_key/" .env
                    print_status $GREEN "✅ Virtual key configured"
                else
                    print_status $RED "❌ Invalid virtual key format (should start with sk-)"
                fi
                ;;
            3)
                print_status $YELLOW "⚠️  Skipping authentication setup"
                ;;
        esac
    else
        read -p "LightLLM authentication token (optional): " token
        if [ -n "$token" ]; then
            sed -i.bak "s/nnLLM_TOKEN=/nnLLM_TOKEN=$token/" .env
            print_status $GREEN "✅ Token configured"
        fi
    fi
    
    echo
    echo "=== ENVIRONMENT ==="
    
    # Environment
    read -p "Environment (development/staging/production) [default: development]: " environment
    environment=${environment:-development}
    sed -i.bak "s/ENVIRONMENT=development/ENVIRONMENT=$environment/" .env
    
    # Log level
    read -p "Log level (error/warn/info/debug/trace) [default: info]: " log_level
    log_level=${log_level:-info}
    sed -i.bak "s/RUST_LOG=info/RUST_LOG=$log_level/" .env
    
    echo
    echo "=== FEATURES ==="
    
    # Feature flags
    read -p "Enable streaming? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        sed -i.bak "s/ENABLE_STREAMING=true/ENABLE_STREAMING=true/" .env
    else
        sed -i.bak "s/ENABLE_STREAMING=true/ENABLE_STREAMING=false/" .env
    fi
    
    read -p "Enable rate limiting? (Y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Nn]$ ]]; then
        sed -i.bak "s/ENABLE_RATE_LIMITING=true/ENABLE_RATE_LIMITING=false/" .env
    fi
    
    read -p "Enable metrics? (Y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Nn]$ ]]; then
        sed -i.bak "s/ENABLE_METRICS=true/ENABLE_METRICS=false/" .env
    fi
    
    # Clean up backup files
    rm -f .env.bak
}

# Function to validate configuration
validate_config() {
    print_status $BLUE "🔍 Validating configuration..."
    
    # Check if .env file exists
    if [ ! -f ".env" ]; then
        print_status $RED "❌ .env file not found"
        return 1
    fi
    
    # Source the .env file
    set -a
    source .env
    set +a
    
    # Validate required fields
    if [ -z "${nnLLM_URL:-}" ]; then
        print_status $RED "❌ nnLLM_URL is not set"
        return 1
    fi
    
    if [ -z "${nnLLM_MODEL:-}" ]; then
        print_status $RED "❌ nnLLM_MODEL is not set"
        return 1
    fi
    
    # Validate URL format
    if [[ ! "$nnLLM_URL" =~ ^https?:// ]]; then
        print_status $RED "❌ nnLLM_URL must start with http:// or https://"
        return 1
    fi
    
    # Validate port
    if ! [[ "$PORT" =~ ^[0-9]+$ ]] || [ "$PORT" -lt 1 ] || [ "$PORT" -gt 65535 ]; then
        print_status $RED "❌ PORT must be a number between 1 and 65535"
        return 1
    fi
    
    # Validate environment
    if [[ ! "$ENVIRONMENT" =~ ^(development|staging|production)$ ]]; then
        print_status $RED "❌ ENVIRONMENT must be one of: development, staging, production"
        return 1
    fi
    
    # Validate log level
    if [[ ! "$RUST_LOG" =~ ^(error|warn|info|debug|trace)$ ]]; then
        print_status $RED "❌ RUST_LOG must be one of: error, warn, info, debug, trace"
        return 1
    fi
    
    print_status $GREEN "✅ Configuration validation passed"
}

# Function to show configuration summary
show_summary() {
    print_status $BLUE "📋 Configuration Summary"
    echo "========================"
    
    # Source the .env file
    set -a
    source .env
    set +a
    
    echo "Server: $HOST:$PORT"
    echo "Backend: $nnLLM_URL"
    echo "Model: $nnLLM_MODEL"
    echo "Environment: $ENVIRONMENT"
    echo "Log Level: $RUST_LOG"
    echo "Streaming: $ENABLE_STREAMING"
    echo "Rate Limiting: $ENABLE_RATE_LIMITING"
    echo "Metrics: $ENABLE_METRICS"
    
    if [ -n "${nnLLM_TOKEN:-}" ]; then
        echo "Authentication: *** (configured)"
    else
        echo "Authentication: none"
    fi
    
    echo "========================"
}

# Function to provide next steps
show_next_steps() {
    print_status $BLUE "🚀 Next Steps"
    echo "============="
    echo
    echo "1. Test your configuration:"
    echo "   cargo run"
    echo
    echo "2. If using LiteLLM proxy and need a virtual key:"
    echo "   ./scripts/generate_virtual_key.sh"
    echo
    echo "3. Test the proxy:"
    echo "   curl -X POST http://localhost:$PORT/v1/chat/completions \\"
    echo "     -H \"Content-Type: application/json\" \\"
    echo "     -d '{\"model\": \"$nnLLM_MODEL\", \"messages\": [{\"role\": \"user\", \"content\": \"Hello!\"}]}'"
    echo
    echo "4. Run tests:"
    echo "   cargo test"
    echo
    echo "5. For production deployment:"
    echo "   - Set ENVIRONMENT=production"
    echo "   - Configure specific CORS_ORIGIN"
    echo "   - Use appropriate log level"
    echo "   - Set up monitoring and health checks"
    echo
}

# Main function
main() {
    print_status $BLUE "🔧 LightLLM Rust Proxy Environment Setup"
    print_status $BLUE "=========================================="
    echo
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        print_status $RED "❌ Error: Must be run from the project root directory"
        exit 1
    fi
    
    # Check if env.example exists
    if [ ! -f "env.example" ]; then
        print_status $RED "❌ Error: env.example file not found"
        exit 1
    fi
    
    # Create .env file
    if check_env_file; then
        create_env_file
    fi
    
    # Prompt for configuration
    prompt_for_config
    
    # Validate configuration
    if validate_config; then
        print_status $GREEN "✅ Environment setup completed successfully!"
        echo
        
        show_summary
        echo
        show_next_steps
        
        print_status $GREEN "🎉 You're ready to run the LightLLM Rust proxy!"
    else
        print_status $RED "❌ Environment setup failed. Please check your configuration."
        exit 1
    fi
}

# Run main function
main "$@"
