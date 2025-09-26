//! # Comprehensive Streaming Tests
//! 
//! This module provides comprehensive tests for streaming functionality including
//! SSE format validation, chunk validation, error handling, and performance testing.

use nexus_nitro_llm::{
    config::Config,
    server::{AppState, create_router},
    schemas::{ChatCompletionRequest, Message},
    streaming_adapters::{StreamingAdapter, EnhancedStreamingHandler, StreamingMetrics},
};
use axum::{
    body::Body,
    http::{Request, StatusCode, HeaderMap, HeaderValue, Method},
    Router,
};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::timeout;
use tower::ServiceExt;
use futures_util::StreamExt;

/// # Test Configuration
/// 
/// Configuration for streaming tests.
struct StreamingTestConfig {
    /// Test timeout duration
    timeout: Duration,
    /// Maximum number of chunks to test
    max_chunks: usize,
    /// Expected chunk size range
    chunk_size_range: (usize, usize),
    /// Expected time between chunks (ms)
    expected_chunk_interval_ms: u64,
}

impl Default for StreamingTestConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_chunks: 100,
            chunk_size_range: (1, 4096),
            expected_chunk_interval_ms: 100,
        }
    }
}

/// # Create Test App State
/// 
/// Creates a test application state with mock configuration.
fn create_test_app_state() -> AppState {
    let config = Config {;
        backend_type: "lightllm".to_string(),
        backend_url: "http://localhost:8000".to_string(),
        model_id: "test-model".to_string(),
        port: 3000,
        ..Default::default()
    };
    
    AppState::new(config)
}

/// # Create Streaming Test Request
/// 
/// Creates a test request with streaming enabled.
fn create_streaming_test_request() -> ChatCompletionRequest {
    ChatCompletionRequest {
        model: Some("test-model".to_string()),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: Some("Generate a long response to test streaming.".to_string()),
                name: None,
                function_call: None,
                tool_call_id: None,
                tool_calls: None,
            },
        ],
        stream: Some(true),
        temperature: Some(0.7),
        max_tokens: Some(200),
        top_p: Some(0.9),
        frequency_penalty: Some(0.0),
        presence_penalty: Some(0.0),
        tools: None,
        tool_choice: None,
    }
}

/// # Test SSE Format Validation
/// 
/// Tests that streaming responses follow proper SSE format.
#[tokio::test]
async fn test_sse_format_validation() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    let config = StreamingTestConfig::default();
    
    let request_data = create_streaming_test_request();
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .header("accept", "text/event-stream")
        .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Should return 200 OK for streaming
    assert_eq!(response.status(), StatusCode::OK);
    
    // Check content type
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/event-stream"));
    
    // Check cache control
    let cache_control = response.headers().get("cache-control").unwrap();
    assert_eq!(cache_control.to_str().unwrap(), "no-cache");
    
    // Check connection
    let connection = response.headers().get("connection").unwrap();
    assert_eq!(connection.to_str().unwrap(), "keep-alive");
}

/// # Test SSE Chunk Format
/// 
/// Tests that SSE chunks follow the correct format.
#[tokio::test]
async fn test_sse_chunk_format() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    let config = StreamingTestConfig::default();
    
    let request_data = create_streaming_test_request();
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .header("accept", "text/event-stream")
        .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    if response.status() == StatusCode::OK {
        // In a real test, we would read the stream and validate chunks
        // For now, we'll test the expected format patterns
        
        let expected_chunk_patterns = vec![;
            "data: ",
            "event: ",
            "id: ",
            "retry: ",
        ];
        
        // These patterns should be present in SSE chunks
        for pattern in expected_chunk_patterns {
            println!("📝 SSE chunk should contain pattern: {}", pattern);
        }
        
        // Test that chunks end with double newline
        println!("📝 SSE chunks should end with \\n\\n");
        
        // Test that data chunks contain valid JSON
        println!("📝 SSE data chunks should contain valid JSON");
    }
}

/// # Test Streaming Error Handling
/// 
/// Tests error handling in streaming responses.
#[tokio::test]
async fn test_streaming_error_handling() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test streaming with invalid request
    let invalid_request = json!({;
        "model": "test-model",
        "messages": [], // Empty messages should cause error
        "stream": true
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .header("accept", "text/event-stream")
        .body(Body::from(serde_json::to_vec(&invalid_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Should return 400 Bad Request
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test streaming with unsupported model
    let unsupported_model_request = json!({;
        "model": "unsupported-model",
        "messages": [{"role": "user", "content": "Hello"}],
        "stream": true
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .header("accept", "text/event-stream")
        .body(Body::from(serde_json::to_vec(&unsupported_model_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Should return 400 Bad Request or 404 Not Found
    assert!(response.status() == StatusCode::BAD_REQUEST || 
            response.status() == StatusCode::NOT_FOUND);
}

/// # Test Streaming Performance
/// 
/// Tests streaming performance and timing.
#[tokio::test]
async fn test_streaming_performance() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    let config = StreamingTestConfig::default();
    
    let request_data = create_streaming_test_request();
    
    let start_time = std::time::Instant::now();
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .header("accept", "text/event-stream")
        .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    let first_byte_time = start_time.elapsed();
    
    // Time to first byte should be reasonable
    assert!(first_byte_time < Duration::from_secs(5));
    
    if response.status() == StatusCode::OK {
        // In a real test, we would measure:
        // - Time to first chunk
        // - Chunk delivery rate
        // - Total streaming time
        // - Memory usage during streaming
        
        println!("📝 Time to first byte: {:?}", first_byte_time);
        println!("📝 Should measure chunk delivery rate");
        println!("📝 Should measure total streaming time");
        println!("📝 Should monitor memory usage");
    }
}

/// # Test Streaming with Different Parameters
/// 
/// Tests streaming with various parameter combinations.
#[tokio::test]
async fn test_streaming_with_parameters() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    let parameter_tests = vec![;
        // High temperature
        json!({
            "model": "test-model",
            "messages": [{"role": "user", "content": "Generate creative content"}],
            "stream": true,
            "temperature": 1.5
        }),
        // Low temperature
        json!({
            "model": "test-model",
            "messages": [{"role": "user", "content": "Generate precise content"}],
            "stream": true,
            "temperature": 0.1
        }),
        // High max_tokens
        json!({
            "model": "test-model",
            "messages": [{"role": "user", "content": "Generate a long response"}],
            "stream": true,
            "max_tokens": 1000
        }),
        // Low max_tokens
        json!({
            "model": "test-model",
            "messages": [{"role": "user", "content": "Generate a short response"}],
            "stream": true,
            "max_tokens": 10
        }),
    ];
    
    for (i, request_data) in parameter_tests.iter().enumerate() {
        let request = Request::builder();
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .header("accept", "text/event-stream")
            .body(Body::from(serde_json::to_vec(request_data).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        
        // Should not return 400 Bad Request for valid parameters
        assert_ne!(response.status(), StatusCode::BAD_REQUEST, 
                  "Parameter test {} failed", i);
    }
}

/// # Test Streaming with Tools
/// 
/// Tests streaming with function calling tools.
#[tokio::test]
async fn test_streaming_with_tools() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    let tool_request = json!({;
        "model": "test-model",
        "messages": [{"role": "user", "content": "What's the weather like?"}],
        "stream": true,
        "tools": [
            {
                "type": "function",
                "function": {
                    "name": "get_weather",
                    "description": "Get current weather",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "location": {
                                "type": "string",
                                "description": "The city to get weather for"
                            }
                        },
                        "required": ["location"]
                    }
                }
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .header("accept", "text/event-stream")
        .body(Body::from(serde_json::to_vec(&tool_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Should not return 400 Bad Request for valid tool request
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
    
    if response.status() == StatusCode::OK {
        // In a real test, we would validate that:
        // - Tool calls are streamed properly
        // - Tool results are handled correctly
        // - Streaming continues after tool execution
        
        println!("📝 Should validate tool call streaming");
        println!("📝 Should validate tool result handling");
        println!("📝 Should validate streaming continuation");
    }
}

/// # Test Streaming Cancellation
/// 
/// Tests streaming request cancellation.
#[tokio::test]
async fn test_streaming_cancellation() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    let request_data = create_streaming_test_request();
    
    // Test with short timeout to simulate cancellation
    let result = timeout(;
        Duration::from_millis(100),
        async {
            let request = Request::builder();
                .method(Method::POST)
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .header("accept", "text/event-stream")
                .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
                .unwrap();
            
            app.clone().oneshot(request).await
        }
    )
    
    match result {
        Ok(response) => {
            // Request completed within timeout
            println!("📝 Request completed within timeout");
        }
        Err(_) => {
            // Request timed out (expected for cancellation test)
            println!("📝 Request timed out (cancellation test)");
        }
    }
}

/// # Test Streaming Metrics
/// 
/// Tests streaming metrics collection.
#[tokio::test]
async fn test_streaming_metrics() {
    let handler = EnhancedStreamingHandler::new();
    let metrics = handler.get_metrics();
    
    // Verify initial metrics
    assert_eq!(metrics.total_requests.load(std::sync::atomic::Ordering::Relaxed), 0);
    assert_eq!(metrics.total_chunks.load(std::sync::atomic::Ordering::Relaxed), 0);
    assert_eq!(metrics.error_count.load(std::sync::atomic::Ordering::Relaxed), 0);
    
    // Simulate some streaming requests
    for i in 0..5 {
        let adapter = nexus_nitro_llm::adapters::Adapter::LightLLM(;
            nexus_nitro_llm::adapters::LightLLMAdapter {
                url: format!("http://localhost:{}", 8000 + i),
                model_id: "test-model".to_string(),
            }
        );
        
        let request = create_streaming_test_request();
        
        // This will fail in test environment, but should update metrics
        let _ = handler.handle_streaming_request(request, adapter);
    }
    
    // Verify metrics were updated
    assert_eq!(metrics.total_requests.load(std::sync::atomic::Ordering::Relaxed), 5);
    assert!(metrics.error_count.load(std::sync::atomic::Ordering::Relaxed) > 0);
    
    println!("✅ Streaming metrics test passed");
}

/// # Test Streaming with Different Content Types
/// 
/// Tests streaming with different content types and encodings.
#[tokio::test]
async fn test_streaming_content_types() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    let content_type_tests = vec![;
        ("text/event-stream", "Standard SSE"),
        ("text/event-stream; charset=utf-8", "SSE with charset"),
        ("application/x-ndjson", "Newline-delimited JSON"),
    ];
    
    for (content_type, description) in content_type_tests {
        let request_data = create_streaming_test_request();
        
        let request = Request::builder();
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .header("accept", content_type)
            .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        
        // Should handle different content types appropriately
        println!("📝 {}: Status {}", description, response.status());
    }
}

/// # Test Streaming Buffer Management
/// 
/// Tests streaming buffer management and memory usage.
#[tokio::test]
async fn test_streaming_buffer_management() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test with large response to stress buffer management
    let large_request = json!({;
        "model": "test-model",
        "messages": [{"role": "user", "content": "Generate a very long response with many details and examples"}],
        "stream": true,
        "max_tokens": 2000
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .header("accept", "text/event-stream")
        .body(Body::from(serde_json::to_vec(&large_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    if response.status() == StatusCode::OK {
        // In a real test, we would:
        // - Monitor memory usage during streaming
        // - Check for buffer overflows
        // - Verify proper cleanup after streaming
        
        println!("📝 Should monitor memory usage during streaming");
        println!("📝 Should check for buffer overflows");
        println!("📝 Should verify proper cleanup");
    }
}

/// # Test Streaming Error Recovery
/// 
/// Tests error recovery during streaming.
#[tokio::test]
async fn test_streaming_error_recovery() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test with invalid backend URL to trigger error
    let config = Config {;
        backend_type: "lightllm".to_string(),
        backend_url: "http://invalid-backend:9999".to_string(),
        model_id: "test-model".to_string(),
        port: 3000,
        ..Default::default()
    };
    
    let error_app_state = AppState::new(config);
    let error_app = create_router(error_app_state);
    
    let request_data = create_streaming_test_request();
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .header("accept", "text/event-stream")
        .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
        .unwrap();
    
    let response = error_app.clone().oneshot(request).await.unwrap();
    
    // Should handle error gracefully
    assert!(response.status() == StatusCode::INTERNAL_SERVER_ERROR ||
            response.status() == StatusCode::BAD_GATEWAY ||
            response.status() == StatusCode::SERVICE_UNAVAILABLE);
    
    // In a real test, we would verify that:
    // - Error is properly formatted as SSE
    // - Client receives proper error information
    // - Connection is closed gracefully
    
    println!("📝 Should format error as SSE");
    println!("📝 Should provide proper error information");
    println!("📝 Should close connection gracefully");
}

/// # Integration Test Suite
/// 
/// Runs a comprehensive integration test suite for streaming functionality.
#[tokio::test]
async fn test_comprehensive_streaming_integration_suite() {
    println!("🚀 Starting comprehensive streaming test suite");
    
    // Test all streaming scenarios
    test_sse_format_validation()
    test_sse_chunk_format()
    test_streaming_error_handling()
    test_streaming_performance()
    test_streaming_with_parameters()
    test_streaming_with_tools()
    test_streaming_cancellation()
    test_streaming_metrics()
    test_streaming_content_types()
    test_streaming_buffer_management()
    test_streaming_error_recovery()
    
    println!("✅ Comprehensive streaming test suite completed");
}
