//! # Native Rust Tests with Mockoon
//!
//! Tests the native Rust library directly against a mock OpenAI-compatible API
//! using Mockoon CLI for comprehensive functionality testing.

use nexus_nitro_llm::{Config, server::{AppState, create_router}};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use reqwest::Client;

/// Test configuration for native Rust tests with Mockoon
fn create_native_test_config() -> Config {
    let mut config = Config::for_test();
    config.backend_url = "http://127.0.0.1:3000".to_string();
    config.backend_type = "openai".to_string();
    config.port = 8085; // Different port to avoid conflicts
    config
}

/// Wait for Mockoon server to be ready
async fn wait_for_mockoon_server() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut attempts = 0;
    let max_attempts = 30;

    while attempts < max_attempts {
        match client.get("http://127.0.0.1:3000/health").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    println!("✅ Mockoon server is ready for native Rust tests");
                    return Ok(());
                }
            }
            Err(_) => {
                // Server not ready yet
            }
        }

        sleep(Duration::from_millis(500))
        attempts += 1;
    }

    Err("Mockoon server failed to become ready within timeout".into())
}

/// Test native Rust server health check
#[tokio::test]
async fn test_native_rust_health_check() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping native Rust test - Mockoon server not running");
        return;
    }

    let config = create_native_test_config();
    let state = AppState::new(config);
    let app = create_router(state);

    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

/// Test native Rust chat completion with Mockoon backend
#[tokio::test]
async fn test_native_rust_chat_completion() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping native Rust test - Mockoon server not running");
        return;
    }

    let config = create_native_test_config();
    let state = AppState::new(config);
    let app = create_router(state);

    let request_body = json!({;
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": "Hello from native Rust!"
            }
        ],
        "max_tokens": 50
    });

    let request = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify response headers
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
}

/// Test native Rust streaming with Mockoon backend
#[tokio::test]
async fn test_native_rust_streaming() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping native Rust test - Mockoon server not running");
        return;
    }

    let config = create_native_test_config();
    let state = AppState::new(config);
    let app = create_router(state);

    let request_body = json!({;
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": "Stream this message!"
            }
        ],
        "stream": true,
        "max_tokens": 50
    });

    let request = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .header("accept", "text/event-stream")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify streaming headers
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/event-stream"));
}

/// Test native Rust models endpoint
#[tokio::test]
async fn test_native_rust_models_endpoint() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping native Rust test - Mockoon server not running");
        return;
    }

    let config = create_native_test_config();
    let state = AppState::new(config);
    let app = create_router(state);

    let request = Request::builder()
        .uri("/v1/models")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
}

/// Test native Rust error handling
#[tokio::test]
async fn test_native_rust_error_handling() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping native Rust test - Mockoon server not running");
        return;
    }

    let config = create_native_test_config();
    let state = AppState::new(config);
    let app = create_router(state);

    // Test malformed JSON
    let request = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from("invalid json"))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test empty messages array (should trigger 400 from Mockoon)
    let request_body = json!({;
        "model": "gpt-3.5-turbo",
        "messages": [],
        "max_tokens": 50
    });

    let request = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// Test native Rust with different models
#[tokio::test]
async fn test_native_rust_different_models() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping native Rust test - Mockoon server not running");
        return;
    }

    let config = create_native_test_config();
    let state = AppState::new(config);
    let app = create_router(state);

    let models = vec!["gpt-3.5-turbo", "gpt-4", "gpt-4-turbo-preview"];

    for model in models {
        let request_body = json!({;
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": format!("Test message for {} from native Rust", model)
                }
            ],
            "max_tokens": 50
        });

        let request = Request::builder()
            .uri("/v1/chat/completions")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

/// Test native Rust concurrent requests
#[tokio::test]
async fn test_native_rust_concurrent_requests() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping native Rust test - Mockoon server not running");
        return;
    }

    let config = create_native_test_config();
    let state = AppState::new(config);
    let app = create_router(state);

    let request_body = json!({;
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": "Concurrent test from native Rust"
            }
        ],
        "max_tokens": 50
    });

    let body_data = serde_json::to_string(&request_body).unwrap();

    // Send 5 concurrent requests
    let mut handles = vec![];
    for i in 0..5 {
        let app_clone = app.clone();
        let body_data_clone = body_data.clone();
        
        let handle = tokio::spawn(async move {;
            let request = Request::builder()
                .uri("/v1/chat/completions")
                .method("POST")
                .header("content-type", "application/json")
                .header("x-request-id", format!("native-rust-test-{}", i))
                .body(Body::from(body_data_clone))
                .unwrap();

            app_clone.oneshot(request).await.unwrap()
        });
        
        handles.push(handle);
    }

    // Wait for all requests to complete
    let mut success_count = 0;
    for handle in handles {
        let response = handle.await.unwrap();
        if response.status().is_success() {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 5);
}

/// Test native Rust large request handling
#[tokio::test]
async fn test_native_rust_large_request() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping native Rust test - Mockoon server not running");
        return;
    }

    let config = create_native_test_config();
    let state = AppState::new(config);
    let app = create_router(state);

    // Create a large message
    let large_content = "A".repeat(10000); // 10KB message
    
    let request_body = json!({;
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": large_content
            }
        ],
        "max_tokens": 100
    });

    let request = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

/// Test native Rust configuration validation
#[tokio::test]
async fn test_native_rust_config_validation() {
    // Test valid configuration
    let config = create_native_test_config();
    assert_eq!(config.backend_url, "http://127.0.0.1:3000");
    assert_eq!(config.backend_type, "openai");
    assert_eq!(config.port, 8085);

    // Test different backend types
    let backend_types = vec!["openai", "azure", "vllm", "lightllm"];
    for backend_type in backend_types {
        let mut config = create_native_test_config();
        config.backend_type = backend_type.to_string();
        assert_eq!(config.backend_type, backend_type);
    }
}

/// Test native Rust adapter creation
#[tokio::test]
async fn test_native_rust_adapter_creation() {
    use nexus_nitro_llm::adapters::Adapter;
    
    let backend_types = vec!["openai", "azure", "vllm", "lightllm"];
    
    for backend_type in backend_types {
        let mut config = create_native_test_config();
        config.backend_type = backend_type.to_string();
        
        let adapter = Adapter::from_config(&config);
        // The adapter name might be different from the backend type
        // Just verify that we can create an adapter successfully
        assert!(adapter.name().len() > 0);
        println!("Created adapter for {} backend: {}", backend_type, adapter.name());
    }
}

/// Comprehensive native Rust integration test
#[tokio::test]
async fn test_native_rust_integration_comprehensive() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping native Rust test - Mockoon server not running");
        return;
    }

    let config = create_native_test_config();
    let state = AppState::new(config);
    let app = create_router(state);

    println!("🧪 Running comprehensive native Rust integration tests...");

    // Test 1: Health check
    let health_request = Request::builder();
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let health_response = app.clone().oneshot(health_request).await.unwrap();
    assert_eq!(health_response.status(), StatusCode::OK);
    println!("✅ Health check passed");

    // Test 2: Models list
    let models_request = Request::builder();
        .uri("/v1/models")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let models_response = app.clone().oneshot(models_request).await.unwrap();
    assert_eq!(models_response.status(), StatusCode::OK);
    println!("✅ Models endpoint passed");

    // Test 3: Chat completion
    let chat_request_body = json!({;
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": "Hello from native Rust integration test!"
            }
        ],
        "max_tokens": 50
    });

    let chat_request = Request::builder();
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&chat_request_body).unwrap()))
        .unwrap();
    
    let chat_response = app.clone().oneshot(chat_request).await.unwrap();
    assert_eq!(chat_response.status(), StatusCode::OK);
    println!("✅ Chat completion passed");

    // Test 4: Streaming chat completion
    let streaming_request_body = json!({;
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": "Stream this from native Rust!"
            }
        ],
        "stream": true,
        "max_tokens": 50
    });

    let streaming_request = Request::builder();
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .header("accept", "text/event-stream")
        .body(Body::from(serde_json::to_string(&streaming_request_body).unwrap()))
        .unwrap();
    
    let streaming_response = app.oneshot(streaming_request).await.unwrap();
    assert_eq!(streaming_response.status(), StatusCode::OK);
    println!("✅ Streaming chat completion passed");

    println!("🎉 All native Rust integration tests passed!");
}

/// Test native Rust performance characteristics
#[tokio::test]
async fn test_native_rust_performance() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping native Rust test - Mockoon server not running");
        return;
    }

    let config = create_native_test_config();
    let state = AppState::new(config);
    let app = create_router(state);

    let request_body = json!({;
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": "Performance test message"
            }
        ],
        "max_tokens": 10
    });

    let body_data = serde_json::to_string(&request_body).unwrap();

    // Measure response time for multiple requests
    let start_time = std::time::Instant::now();
    let request_count = 10;

    for i in 0..request_count {
        let request = Request::builder()
            .uri("/v1/chat/completions")
            .method("POST")
            .header("content-type", "application/json")
            .header("x-request-id", format!("perf-test-{}", i))
            .body(Body::from(body_data.clone()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    let elapsed = start_time.elapsed();
    let avg_time = elapsed / request_count;
    
    println!("🚀 Native Rust Performance Results:");
    println!("   Total requests: {}", request_count);
    println!("   Total time: {:?}", elapsed);
    println!("   Average time per request: {:?}", avg_time);
    
    // Assert reasonable performance (should be under 1 second per request with Mockoon)
    assert!(avg_time.as_millis() < 1000, "Average response time too slow: {:?}", avg_time);
}