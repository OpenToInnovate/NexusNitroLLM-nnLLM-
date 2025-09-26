//! # Streaming Tests
//! 
//! Comprehensive tests for the streaming functionality of the LightLLM Rust proxy.
//! Tests both the SSE format and the streaming response handling.

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    Router,
};
use nexus_nitro_llm::{config::Config, routes::AppState, adapters::Adapter};
use serde_json::json;
use tower::ServiceExt;
use wiremock::{
    matchers::{body_json, method, path},
    Mock, MockServer, ResponseTemplate,
};

/// # Create Test App with Streaming Support
///
/// Creates a test application with streaming support enabled.
async fn create_streaming_test_app(lightllm_url: String) -> Router {
    let mut config = Config::for_test();
    config.port = 8080;
    config.backend_url = lightllm_url;
    config.model_id = "test-model".to_string();

    let state = AppState::new(config);
    nexus_nitro_llm::routes::create_router(state)
}

/// # Test Streaming Request Format
/// 
/// Tests that the proxy correctly handles streaming requests and returns proper SSE format.
#[tokio::test]
async fn test_streaming_request_format() {
    let mock_server = MockServer::start();
    let app = create_streaming_test_app(mock_server.uri());
    
    let request_body = json!({;
        "model": "test-model",
        "messages": [
            {"role": "user", "content": "What is the capital of France?"}
        ],
        "max_tokens": 50,
        "stream": true
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Check that we get a streaming response
    assert_eq!(response.status(), StatusCode::OK);
    
    // Check that the content type is correct for SSE
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/event-stream"));
    
    // Check that we get SSE format (even if it's an error)
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    
    // Should contain SSE format with data: prefix
    assert!(body_str.contains("data: "));
    
    // Should be valid JSON in the data field
    let lines: Vec<&str> = body_str.lines().collect();
    for line in lines {
        if line.starts_with("data: ") {
            let json_part = &line[6..]; // Remove "data: " prefix
            if json_part.trim() != "" {
                // Should be valid JSON
                let _: serde_json::Value = serde_json::from_str(json_part).unwrap();
            }
        }
    }
}

/// # Test Non-Streaming Request Still Works
/// 
/// Ensures that regular (non-streaming) requests still work correctly.
#[tokio::test]
async fn test_non_streaming_request() {
    let mock_server = MockServer::start();

    // Set up mock for non-streaming request
    Mock::given(method("POST"))
        .and(path("/generate"))
        .and(body_json(json!({
            "prompt": "<|user|>\nWhat is the capital of France?\n<|assistant|> ",
            "max_new_tokens": 50,
            "temperature": 1.0,
            "top_p": 1.0
        })))
        .respond_with(ResponseTemplate::new(200)
            .insert_header("content-type", "application/json")
            .set_body_json(json!({
                "text": "The capital of France is Paris."
            })))
        .mount(&mock_server);
        

    let app = create_streaming_test_app(mock_server.uri());
    
    let request_body = json!({;
        "model": "test-model",
        "messages": [
            {"role": "user", "content": "What is the capital of France?"}
        ],
        "max_tokens": 50,
        "stream": false
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Should get a regular JSON response
    assert_eq!(response.status(), StatusCode::OK);
    
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
}

/// # Test Streaming with Missing Stream Parameter
/// 
/// Tests that requests without the stream parameter default to non-streaming.
#[tokio::test]
async fn test_missing_stream_parameter() {
    let mock_server = MockServer::start();

    // Set up mock for non-streaming request
    Mock::given(method("POST"))
        .and(path("/generate"))
        .and(body_json(json!({
            "prompt": "<|user|>\nWhat is the capital of France?\n<|assistant|> ",
            "max_new_tokens": 50,
            "temperature": 1.0,
            "top_p": 1.0
        })))
        .respond_with(ResponseTemplate::new(200)
            .insert_header("content-type", "application/json")
            .set_body_json(json!({
                "text": "The capital of France is Paris."
            })))
        .mount(&mock_server);
        

    let app = create_streaming_test_app(mock_server.uri());
    
    let request_body = json!({;
        "model": "test-model",
        "messages": [
            {"role": "user", "content": "What is the capital of France?"}
        ],
        "max_tokens": 50
        // No stream parameter
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Should get a regular JSON response (not streaming)
    assert_eq!(response.status(), StatusCode::OK);
    
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
}

/// # Test Adapter Streaming Support
/// 
/// Tests that adapters correctly report their streaming support capabilities.
#[tokio::test]
async fn test_adapter_streaming_support() {
    let mock_server = MockServer::start().await;
    let mut config = Config::for_test();
    config.port = 8080;
    config.backend_url = mock_server.uri();
    config.model_id = "test-model".to_string();

    let adapter = Adapter::from_config(&config);
    
    // Both adapters should support streaming now
    assert!(adapter.supports_streaming());
}

/// # Test SSE Event Format
/// 
/// Tests that SSE events are properly formatted according to OpenAI specification.
#[tokio::test]
async fn test_sse_event_format() {
    use nexus_nitro_llm::schemas::{ChatCompletionChunk, StreamChoice, StreamDelta};
    
    // Create a sample chunk
    let chunk = ChatCompletionChunk {
        id: "chatcmpl-test".to_string(),
        object: "chat.completion.chunk".to_string(),
        created: 1234567890,
        model: "test-model".to_string(),
        choices: vec![StreamChoice {
            index: 0,
            delta: StreamDelta {
                role: Some("assistant".to_string()),
                content: Some("Hello".to_string()),
                tool_calls: None,
                function_call: None,
            },
            finish_reason: None,
        }],
        usage: None,
    };
    
    // Serialize to JSON
    let json_str = serde_json::to_string(&chunk).unwrap();
    
    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    
    // Check required fields
    assert_eq!(parsed["id"], "chatcmpl-test");
    assert_eq!(parsed["object"], "chat.completion.chunk");
    assert_eq!(parsed["model"], "test-model");
    assert!(parsed["choices"].is_array());
    assert_eq!(parsed["choices"][0]["delta"]["content"], "Hello");
}

/// # Test Error Handling in Streaming
/// 
/// Tests that errors are properly formatted in streaming responses.
#[tokio::test]
async fn test_streaming_error_handling() {
    use nexus_nitro_llm::schemas::{StreamingError, ErrorDetails};
    
    let error = StreamingError {
        error: ErrorDetails {
            message: "Test error".to_string(),
            r#type: "test_error".to_string(),
            code: Some("TEST_ERROR".to_string()),
        },
    };
    
    let json_str = serde_json::to_string(&error).unwrap();
    
    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    
    // Check error structure
    assert_eq!(parsed["error"]["message"], "Test error");
    assert_eq!(parsed["error"]["type"], "test_error");
    assert_eq!(parsed["error"]["code"], "TEST_ERROR");
}
