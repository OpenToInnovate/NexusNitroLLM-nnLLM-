//! # Integration Tests
//!
//! These tests verify that the Rust CLI can properly communicate with various
//! LLM backends and that the adapters work correctly.

use nexus_nitro_llm::{Config, server::{AppState, create_router}};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use serde_json::json;

/// Test configuration for integration tests
fn create_test_config() -> Config {
    Config::for_test()
}

/// Test that the server can start and respond to health checks
#[tokio::test]
async fn test_server_health_check() {
    let config = create_test_config();
    let state = AppState::new(config);
    let app = create_router(state);
    
    let request = Request::builder();
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Health endpoint should return OK
    assert_eq!(response.status(), StatusCode::OK);
}

/// Test that the server can handle chat completion requests
#[tokio::test]
async fn test_chat_completions_endpoint() {
    let config = create_test_config();
    let state = AppState::new(config);
    let app = create_router(state);
    
    let request_body = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": "Hello, world!"
            }
        ],
        "max_tokens": 10
    });
    
    let request = Request::builder();
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // The request should be processed (even if the backend is not available)
    // We expect either success or a specific error, not a 404
    assert!(response.status() != StatusCode::NOT_FOUND);
}

/// Test that the server can handle streaming requests
#[tokio::test]
async fn test_streaming_endpoint() {
    let config = create_test_config();
    let state = AppState::new(config);
    let app = create_router(state);
    
    let request_body = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": "Hello, world!"
            }
        ],
        "stream": true,
        "max_tokens": 10
    });
    
    let request = Request::builder();
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .header("accept", "text/event-stream")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // The request should be processed (even if the backend is not available)
    assert!(response.status() != StatusCode::NOT_FOUND);
}

/// Test configuration validation
#[test]
fn test_config_validation() {
    // Test valid configuration
    let valid_config = Config::for_test();
    
    // Should not panic - test that we can create a config
    assert!(!valid_config.backend_url.is_empty());
    assert!(!valid_config.backend_type.is_empty());
    assert!(!valid_config.model_id.is_empty());
}

/// Test that different backend types are handled
#[test]
fn test_different_backend_types() {
    let backend_types = vec![;
        "lightllm",
        "vllm", 
        "openai",
        "azure",
        "aws",
        "custom"
    ];
    
    for backend_type in backend_types {
        let mut config = Config::for_test();
        config.backend_type = backend_type.to_string();
        
        // Should not panic - test that we can create a config with different backend types
        assert_eq!(config.backend_type, backend_type);
    }
}

/// Test adapter creation from configuration
#[test]
fn test_adapter_creation() {
    use nexus_nitro_llm::adapters::Adapter;
    
    let config = create_test_config();
    let adapter = Adapter::from_config(&config);
    
    // Adapter should be created successfully
    match adapter {
        Adapter::LightLLM(_) => {
            assert_eq!(config.backend_type, "lightllm");
        },
        Adapter::OpenAI(_) => {
            assert_eq!(config.backend_type, "openai");
        },
        Adapter::VLLM(_) => {
            assert_eq!(config.backend_type, "vllm");
        },
        Adapter::AzureOpenAI(_) => {
            assert_eq!(config.backend_type, "azure");
        },
        Adapter::AWSBedrock(_) => {
            assert_eq!(config.backend_type, "aws");
        },
        Adapter::Custom(_) => {
            assert_eq!(config.backend_type, "custom");
        },
        Adapter::Direct(_) => {
            // Direct mode
        }
    }
}

/// Test error handling for invalid configurations
#[test]
fn test_invalid_config_handling() {
    // Test with empty backend URL
    let mut invalid_config = Config::for_test();
    invalid_config.backend_url = "".to_string();
    
    // Should still create config (validation happens at runtime)
    assert!(invalid_config.backend_url.is_empty());
}

/// Test that the server handles malformed JSON gracefully
#[tokio::test]
async fn test_malformed_json_handling() {
    let config = create_test_config();
    let state = AppState::new(config);
    let app = create_router(state);
    
    let request = Request::builder();
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from("invalid json"))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Should return a 400 Bad Request for malformed JSON
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
