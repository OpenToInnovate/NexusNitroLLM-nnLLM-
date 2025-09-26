# NexusNitroLLM Library Usage

This document provides comprehensive guidance on using `nexus_nitro_llm` as a library in your Rust projects.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
nexus_nitro_llm = "0.1.0"

# Or with specific features
nexus_nitro_llm = { version = "0.1.0", features = ["server", "metrics", "caching"] }
```

## Features

The library supports modular features to keep your dependencies minimal:

- **`default`**: Includes `server`, `enhanced`, `caching`, `rate-limiting`, `metrics`
- **`server`**: HTTP server components (axum, tower, tokio)
- **`enhanced`**: Advanced features like streaming, batching, routing
- **`caching`**: Response caching system
- **`rate-limiting`**: Rate limiting and throttling
- **`distributed-rate-limiting`**: Distributed rate limiting (requires `rate-limiting`)
- **`metrics`**: Performance metrics collection
- **`cli`**: Command-line interface support
- **`streaming-enhanced`**: Advanced streaming features
- **`batching`**: Request batching for efficiency
- **`routing`**: Intelligent routing to multiple backends

## Quick Start

### Basic Server

```rust
use nexus_nitro_llm::{Config, AppState, create_router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = Config::for_test(); // or Config::parse_args() for CLI

    // Create application state
    let state = AppState::new(config).await;

    // Create router
    let app = create_router(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app.into_make_service()
    ).await?;

    Ok(())
}
```

### Using Adapters Directly

```rust
use lightllm_rust::{Config, Adapter, ChatCompletionRequest, Message, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::for_test();
    let adapter = Adapter::from_config(&config);

    let request = ChatCompletionRequest {
        model: "llama".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: "Hello!".to_string(),
            name: None,
        }],
        max_tokens: Some(100),
        temperature: Some(0.7),
        // ... other fields
    };

    let response = adapter.send_request(&request).await?;
    println!("Response: {:#?}", response);

    Ok(())
}
```

## Configuration

The `Config` struct provides comprehensive configuration options:

```rust
use lightllm_rust::Config;

let mut config = Config::for_test();
config.port = 8080;
config.lightllm_url = "http://localhost:8000".to_string();
config.model_id = "llama".to_string();
config.enable_metrics = true;
config.enable_caching = true;
config.rate_limit_requests_per_minute = 100;
```

### Environment Variables

The library supports configuration via environment variables:

```bash
export nnLLM_URL="http://localhost:8000"
export nnLLM_MODEL="llama"
export nnLLM_TOKEN="your-token-here"
export PORT="8080"
export ENABLE_METRICS="true"
```

## Feature Examples

### With Metrics (requires `metrics` feature)

```rust
use lightllm_rust::{Config, AppState, LLMMetrics};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::for_test();
    config.enable_metrics = true;

    let state = AppState::new(config).await;
    let metrics = Arc::new(LLMMetrics::new());

    // Monitor metrics
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            let stats = metrics.get_stats().await;
            println!("Requests: {}, Avg time: {:.2}ms",
                     stats.total_requests, stats.average_response_time);
        }
    });

    // ... rest of server setup
    Ok(())
}
```

### With Caching (requires `caching` feature)

```rust
use lightllm_rust::{Config, CacheManager, CacheConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cache_config = CacheConfig {
        max_size: 1000,
        ttl_seconds: 300,
        enable_compression: true,
    };

    let cache_manager = CacheManager::new(cache_config).await;

    // Cache manager is automatically used in AppState when caching is enabled
    let mut config = Config::for_test();
    config.enable_caching = true;
    config.cache_max_size = 1000;
    config.cache_ttl_seconds = 300;

    // ... rest of setup
    Ok(())
}
```

### With Rate Limiting (requires `rate-limiting` feature)

```rust
use lightllm_rust::{Config, AdvancedRateLimiter, RateLimitRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rate_limiter = AdvancedRateLimiter::new(60, 10).await; // 60 RPM, 10 burst

    let limit_request = RateLimitRequest {
        identifier: "user123".to_string(),
        tokens_requested: 1,
        priority: lightllm_rust::rate_limiting::Priority::Normal,
    };

    match rate_limiter.check_rate_limit(&limit_request).await {
        Ok(result) => {
            if result.allowed {
                println!("Request allowed");
            } else {
                println!("Rate limited. Retry after: {:?}", result.retry_after);
            }
        }
        Err(e) => println!("Rate limiting error: {}", e),
    }

    Ok(())
}
```

## Custom Integration

### Embedding in Existing Applications

You can embed the proxy functionality into existing applications:

```rust
use lightllm_rust::{Config, AppState, chat_completions};
use axum::{Router, routing::post};

async fn create_custom_app() -> Router {
    let config = Config::for_test();
    let state = AppState::new(config).await;

    Router::new()
        .route("/api/llm/chat", post(chat_completions))
        .route("/api/health", get(health_check))
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}
```

### Custom Adapters

You can extend the adapter system:

```rust
use lightllm_rust::{Adapter, ChatCompletionRequest, ProxyError};

// Create custom logic around adapters
async fn custom_request_handler(
    adapter: &Adapter,
    mut request: ChatCompletionRequest,
) -> Result<serde_json::Value, ProxyError> {
    // Add custom preprocessing
    request.messages.insert(0, system_message());

    // Send request
    let response = adapter.send_request(&request).await?;

    // Add custom postprocessing
    Ok(response)
}

fn system_message() -> lightllm_rust::Message {
    lightllm_rust::Message {
        role: lightllm_rust::Role::System,
        content: "You are a helpful assistant.".to_string(),
        name: None,
    }
}
```

## Error Handling

The library uses a custom `ProxyError` type:

```rust
use lightllm_rust::{ProxyError, Result};

async fn handle_request() -> Result<String> {
    // Library functions return Result<T, ProxyError>
    match some_library_function().await {
        Ok(result) => Ok(result),
        Err(ProxyError::BackendError(msg)) => {
            eprintln!("Backend error: {}", msg);
            Err(ProxyError::BackendError(msg))
        }
        Err(ProxyError::ConfigurationError(msg)) => {
            eprintln!("Config error: {}", msg);
            Err(ProxyError::ConfigurationError(msg))
        }
        Err(e) => {
            eprintln!("Other error: {}", e);
            Err(e)
        }
    }
}
```

## Testing

The library provides test utilities:

```rust
#[cfg(test)]
mod tests {
    use lightllm_rust::{Config, AppState};

    #[tokio::test]
    async fn test_configuration() {
        let config = Config::for_test();
        assert_eq!(config.port, 8080);
        assert_eq!(config.model_id, "llama");
    }

    #[tokio::test]
    async fn test_app_state() {
        let config = Config::for_test();
        let state = AppState::new(config).await;
        // Test your application logic
    }
}
```

## Performance Tips

1. **Enable HTTP/2**: The library automatically uses HTTP/2 when available
2. **Connection Pooling**: Configure `http_client_max_connections` appropriately
3. **Caching**: Enable caching for repeated requests
4. **Batching**: Use batching for high-throughput scenarios
5. **Rate Limiting**: Prevent backend overload with appropriate limits

## Minimal Feature Set

For the smallest possible dependency footprint:

```toml
[dependencies]
lightllm_rust = { version = "0.1.0", default-features = false }
```

This gives you just the core types and adapter functionality without server dependencies.