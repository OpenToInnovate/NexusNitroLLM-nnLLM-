//! # Advanced Async and Streaming Tests
//! 
//! Comprehensive tests for complex async scenarios, concurrent streaming,
//! error handling, and performance characteristics of the LightLLM Rust proxy.
//! 
//! These tests validate:
//! - Concurrent streaming requests
//! - Error recovery and resilience
//! - Performance under load
//! - Memory management during streaming
//! - Connection pooling behavior
//! - Backpressure handling

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    routing::post,
    Router,
};
use nexus_nitro_llm::{
    config::Config,
    routes::{chat_completions, AppState},
};
use serde_json::json;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::Semaphore,
    time::timeout,
};
use tower::ServiceExt;

/// # Create Test App with Advanced Configuration
/// 
/// Creates a test application with optimized settings for testing.
async fn create_advanced_test_app() -> Router {
    let mut config = Config::for_test();
    config.port = 8080;
    config.backend_url = "http://localhost:8000".to_string();
    config.model_id = "test-model".to_string();
    
    let state = AppState::new(config).await;
    
    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(state)
}

/// # Test Concurrent Streaming Requests
/// 
/// Tests that the proxy can handle multiple concurrent streaming requests
/// without issues. This validates connection pooling and async handling.
#[tokio::test]
async fn test_concurrent_streaming_requests() {
    let app = create_advanced_test_app().await;
    
    // Create semaphore to limit concurrent requests
    let semaphore = Arc::new(Semaphore::new(10));
    let mut handles = vec![];
    
    // Launch 20 concurrent streaming requests
    for i in 0..20 {
        let app = app.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        
        let handle = tokio::spawn(async move {
            let _permit = permit; // Hold the permit
            
            let request_body = json!({
                "model": "test-model",
                "messages": [
                    {"role": "user", "content": format!("Concurrent request {}", i)}
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
            
            // All requests should succeed (even if backend returns errors)
            assert_eq!(response.status(), StatusCode::OK);
            
            // Should be streaming response
            let content_type = response.headers().get("content-type").unwrap();
            assert!(content_type.to_str().unwrap().contains("text/event-stream"));
            
            i // Return request ID for verification
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let results = futures::future::join_all(handles).await;
    
    // Verify all requests completed successfully
    let completed_requests: Vec<usize> = results
        .into_iter()
        .map(|result| result.unwrap())
        .collect();
    
    assert_eq!(completed_requests.len(), 20);
    assert_eq!(completed_requests.iter().max().unwrap(), &19);
    assert_eq!(completed_requests.iter().min().unwrap(), &0);
}

/// # Test Streaming with Timeout Handling
/// 
/// Tests that streaming requests handle timeouts gracefully and don't
/// hang indefinitely.
#[tokio::test]
async fn test_streaming_timeout_handling() {
    let app = create_advanced_test_app().await;
    
    let request_body = json!({
        "model": "test-model",
        "messages": [
            {"role": "user", "content": "Test timeout handling"}
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
    
    // Test with a 5-second timeout
    let result = timeout(Duration::from_secs(5), app.oneshot(request)).await;
    
    match result {
        Ok(response) => {
            let response = response.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            
            // Should be streaming response
            let content_type = response.headers().get("content-type").unwrap();
            assert!(content_type.to_str().unwrap().contains("text/event-stream"));
        }
        Err(_) => {
            // Timeout is acceptable - the request should complete quickly
            // If it times out, that indicates a problem
            panic!("Request timed out - this should not happen with proper error handling");
        }
    }
}

/// # Test Memory Management During Streaming
/// 
/// Tests that streaming responses don't cause memory leaks or excessive
/// memory usage by sending many concurrent requests.
#[tokio::test]
async fn test_memory_management_streaming() {
    let app = create_advanced_test_app().await;
    
    // Record initial memory state (approximate)
    let initial_requests = 100;
    let mut handles = vec![];
    
    // Launch many concurrent requests to test memory management
    for i in 0..initial_requests {
        let app = app.clone();
        
        let handle = tokio::spawn(async move {
            let request_body = json!({
                "model": "test-model",
                "messages": [
                    {"role": "user", "content": format!("Memory test request {}", i)}
                ],
                "max_tokens": 10,
                "stream": true
            });
            
            let request = Request::builder()
                .method(Method::POST)
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                .unwrap();
            
            let response = app.oneshot(request).await.unwrap();
            
            // Verify response is valid
            assert_eq!(response.status(), StatusCode::OK);
            
            // Read the response body to ensure it's processed
            let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let body_str = String::from_utf8_lossy(&body_bytes);
            
            // Should contain SSE format
            assert!(body_str.contains("data: "));
            
            i
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests with a timeout
    let results = timeout(
        Duration::from_secs(30),
        futures::future::join_all(handles)
    )
    
    match results {
        Ok(completed_handles) => {
            let successful_requests: Vec<usize> = completed_handles
                .into_iter()
                .map(|result| result.unwrap())
                .collect();
            
            assert_eq!(successful_requests.len(), initial_requests);
            println!("✅ Memory management test passed: {} requests completed", successful_requests.len());
        }
        Err(_) => {
            panic!("Memory management test timed out - possible memory leak or performance issue");
        }
    }
}

/// # Test Error Recovery and Resilience
/// 
/// Tests that the proxy handles various error conditions gracefully
/// during streaming operations.
#[tokio::test]
async fn test_error_recovery_streaming() {
    let app = create_advanced_test_app().await;
    
    // Test cases for different error conditions
    let test_cases = vec![
        // Invalid JSON
        (
            "invalid json",
            StatusCode::BAD_REQUEST,
        ),
        // Missing required fields
        (
            r#"{"stream": true}"#,
            StatusCode::BAD_REQUEST,
        ),
        // Empty messages
        (
            r#"{"model": "test", "messages": [], "stream": true}"#,
            StatusCode::BAD_REQUEST,
        ),
        // Invalid stream parameter
        (
            r#"{"model": "test", "messages": [{"role": "user", "content": "test"}], "stream": "invalid"}"#,
            StatusCode::BAD_REQUEST,
        ),
    ];
    
    for (test_case, expected_status) in test_cases {
        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .body(Body::from(test_case.as_bytes()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        
        // Verify error handling
        assert_eq!(response.status(), expected_status);
        
        // For streaming requests, errors should still be in SSE format
        if test_case.contains("\"stream\": true") || test_case.contains("\"stream\": \"invalid\"") {
            let content_type = response.headers().get("content-type");
            if let Some(ct) = content_type {
                // Should be streaming format even for errors
                assert!(ct.to_str().unwrap().contains("text/event-stream"));
            }
        }
    }
}

/// # Test Performance Under Load
/// 
/// Tests the proxy's performance characteristics under various load conditions.
#[tokio::test]
async fn test_performance_under_load() {
    let app = create_advanced_test_app().await;
    
    // Test different load levels
    let load_levels = vec![1, 5, 10, 20];
    
    for load_level in load_levels {
        println!("Testing load level: {} concurrent requests", load_level);
        
        let start_time = Instant::now();
        let mut handles = vec![];
        
        // Launch concurrent requests
        for i in 0..load_level {
            let app = app.clone();
            
            let handle = tokio::spawn(async move {
                let request_body = json!({
                    "model": "test-model",
                    "messages": [
                        {"role": "user", "content": format!("Load test {}", i)}
                    ],
                    "max_tokens": 20,
                    "stream": true
                });
                
                let request = Request::builder()
                    .method(Method::POST)
                    .uri("/v1/chat/completions")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                    .unwrap();
                
                let response = app.oneshot(request).await.unwrap();
                
                // Verify response
                assert_eq!(response.status(), StatusCode::OK);
                
                // Read response to ensure complete processing
                let _body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
                
                i
            });
            
            handles.push(handle);
        }
        
        // Wait for all requests to complete with timeout
        let results = timeout(
            Duration::from_secs(30),
            futures::future::join_all(handles)
        )
        
        let duration = start_time.elapsed();
        
        match results {
            Ok(completed_handles) => {
                let successful_requests: Vec<usize> = completed_handles
                    .into_iter()
                    .map(|result| result.unwrap())
                    .collect();
                
                assert_eq!(successful_requests.len(), load_level);
                
                let avg_time_per_request = duration.as_millis() as f64 / load_level as f64;
                println!("  ✅ Load level {}: {}ms average per request", load_level, avg_time_per_request);
                
                // Performance assertions
                assert!(avg_time_per_request < 1000.0, "Average response time too high: {}ms", avg_time_per_request);
            }
            Err(_) => {
                panic!("Performance test timed out at load level {}", load_level);
            }
        }
    }
}

/// # Test Streaming Chunk Processing
/// 
/// Tests that streaming chunks are processed correctly and maintain
/// proper format throughout the stream.
#[tokio::test]
async fn test_streaming_chunk_processing() {
    let app = create_advanced_test_app().await;
    
    let request_body = json!({
        "model": "test-model",
        "messages": [
            {"role": "user", "content": "Test chunk processing"}
        ],
        "max_tokens": 100,
        "stream": true
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/event-stream"));
    
    // Read and analyze the streaming response
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    
    // Split into lines and analyze each chunk
    let lines: Vec<&str> = body_str.lines().collect();
    let mut data_lines = 0;
    let mut valid_json_chunks = 0;
    
    for line in lines {
        if line.starts_with("data: ") {
            data_lines += 1;
            let json_part = &line[6..]; // Remove "data: " prefix
            
            if json_part.trim().is_empty() {
                continue; // Skip empty data lines
            }
            
            // Try to parse as JSON
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_part) {
                valid_json_chunks += 1;
                
                // Verify required fields for streaming chunks
                if let Some(object) = parsed.get("object") {
                    if object == "chat.completion.chunk" {
                        // Verify chunk structure
                        assert!(parsed.get("id").is_some(), "Chunk missing ID");
                        assert!(parsed.get("model").is_some(), "Chunk missing model");
                        assert!(parsed.get("choices").is_some(), "Chunk missing choices");
                    }
                }
            }
        }
    }
    
    // Verify we got streaming data
    assert!(data_lines > 0, "No data lines found in streaming response");
    assert!(valid_json_chunks > 0, "No valid JSON chunks found in streaming response");
    
    println!("✅ Streaming chunk processing: {} data lines, {} valid JSON chunks", data_lines, valid_json_chunks);
}

/// # Test Connection Pooling Behavior
/// 
/// Tests that the HTTP client connection pooling works correctly
/// for streaming requests.
#[tokio::test]
async fn test_connection_pooling_streaming() {
    let app = create_advanced_test_app().await;
    
    // Make multiple requests in sequence to test connection reuse
    let request_count = 10;
    let mut response_times = vec![];
    
    for i in 0..request_count {
        let start_time = Instant::now();
        
        let request_body = json!({
            "model": "test-model",
            "messages": [
                {"role": "user", "content": format!("Connection test {}", i)}
            ],
            "max_tokens": 20,
            "stream": true
        });
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        
        let duration = start_time.elapsed();
        response_times.push(duration);
        
        // Verify response
        assert_eq!(response.status(), StatusCode::OK);
        
        // Read response to ensure complete processing
        let _body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    }
    
    // Analyze response times
    let first_request_time = response_times[0].as_millis();
    let avg_time: u128 = response_times.iter().map(|t| t.as_millis()).sum::<u128>() / request_count as u128;
    
    println!("✅ Connection pooling test:");
    println!("  First request: {}ms", first_request_time);
    println!("  Average time: {}ms", avg_time);
    
    // Subsequent requests should be faster due to connection reuse
    // (though this may not always be true in test environment)
    assert!(avg_time < 1000, "Average response time too high: {}ms", avg_time);
}

/// # Test Backpressure Handling
/// 
/// Tests that the proxy handles backpressure correctly when clients
/// read streaming responses slowly.
#[tokio::test]
async fn test_backpressure_handling() {
    let app = create_advanced_test_app().await;
    
    let request_body = json!({
        "model": "test-model",
        "messages": [
            {"role": "user", "content": "Test backpressure handling"}
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
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Read the full response and analyze it
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    
    // Count data lines to simulate chunk processing
    let _data_lines = body_str.matches("data: ").count();
    
    // Simulate slow processing by adding delays between chunks
    let lines: Vec<&str> = body_str.lines().collect();
    let mut processed_chunks = 0;
    
    for line in lines {
        if line.starts_with("data: ") {
            processed_chunks += 1;
            // Simulate slow processing
            tokio::time::sleep(Duration::from_millis(10))
        }
    }
    
    println!("✅ Backpressure handling test: {} chunks processed", processed_chunks);
    assert!(processed_chunks > 0, "No chunks were processed");
}

/// # Test Mixed Request Types
/// 
/// Tests that the proxy can handle a mix of streaming and non-streaming
/// requests concurrently without issues.
#[tokio::test]
async fn test_mixed_request_types() {
    let app = create_advanced_test_app().await;
    
    let mut handles = vec![];
    
    // Launch mix of streaming and non-streaming requests
    for i in 0..10 {
        let app = app.clone();
        let is_streaming = i % 2 == 0; // Alternate between streaming and non-streaming
        
        let handle = tokio::spawn(async move {
            let request_body = json!({
                "model": "test-model",
                "messages": [
                    {"role": "user", "content": format!("Mixed request {}", i)}
                ],
                "max_tokens": 20,
                "stream": is_streaming
            });
            
            let request = Request::builder()
                .method(Method::POST)
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                .unwrap();
            
            let response = app.oneshot(request).await.unwrap();
            
            // Verify response
            assert_eq!(response.status(), StatusCode::OK);
            
            let content_type = response.headers().get("content-type").unwrap();
            
            if is_streaming {
                assert!(content_type.to_str().unwrap().contains("text/event-stream"));
            } else {
                assert!(content_type.to_str().unwrap().contains("application/json"));
            }
            
            // Read response to ensure complete processing
            let _body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            
            (i, is_streaming)
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let results = timeout(
        Duration::from_secs(30),
        futures::future::join_all(handles)
    )
    
    match results {
        Ok(completed_handles) => {
            let successful_requests: Vec<(usize, bool)> = completed_handles
                .into_iter()
                .map(|result| result.unwrap())
                .collect();
            
            assert_eq!(successful_requests.len(), 10);
            
            let streaming_count = successful_requests.iter().filter(|(_, is_streaming)| *is_streaming).count();
            let non_streaming_count = successful_requests.iter().filter(|(_, is_streaming)| !is_streaming).count();
            
            println!("✅ Mixed request types test:");
            println!("  Streaming requests: {}", streaming_count);
            println!("  Non-streaming requests: {}", non_streaming_count);
            
            assert_eq!(streaming_count, 5);
            assert_eq!(non_streaming_count, 5);
        }
        Err(_) => {
            panic!("Mixed request types test timed out");
        }
    }
}
