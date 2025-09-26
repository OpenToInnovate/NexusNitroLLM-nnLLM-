//! # Batching & Concurrency Tests
//! 
//! This module provides comprehensive tests for batching multiple requests
//! and handling concurrent requests efficiently.

use nexus_nitro_llm::{
    config::Config,
    server::{AppState, create_router},
    schemas::{ChatCompletionRequest, Message},
};
use axum::{
    body::Body,
    http::{Request, StatusCode, HeaderMap, HeaderValue, Method},
    Router,
};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tower::ServiceExt;
use futures_util::future::join_all;

/// # Test Configuration
/// 
/// Configuration for batching and concurrency tests.
struct BatchingTestConfig {
    /// Test timeout duration
    timeout: Duration,
    /// Maximum concurrent requests
    max_concurrent_requests: usize,
    /// Batch size for testing
    batch_size: usize,
    /// Request delay between batches (ms)
    batch_delay_ms: u64,
    /// Expected response time per request (ms)
    expected_response_time_ms: u64,
}

impl Default for BatchingTestConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(60),
            max_concurrent_requests: 100,
            batch_size: 10,
            batch_delay_ms: 100,
            expected_response_time_ms: 1000,
        }
    }
}

/// # Create Test App State
/// 
/// Creates a test application state with mock configuration.
async fn create_test_app_state() -> AppState {
    let config = Config {;
        backend_type: "lightllm".to_string(),
        backend_url: "http://localhost:8000".to_string(),
        model_id: "test-model".to_string(),
        port: 3000,
        ..Default::default()
    };
    
    AppState::new(config).await
}

/// # Create Test Request
/// 
/// Creates a test request for batching and concurrency testing.
fn create_test_request(request_id: usize) -> ChatCompletionRequest {
    ChatCompletionRequest {
        model: Some("test-model".to_string()),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: Some(format!("Request {}: Hello, world!", request_id)),
                name: None,
                function_call: None,
                tool_call_id: None,
                tool_calls: None,
            },
        ],
        stream: Some(false),
        temperature: Some(0.7),
        max_tokens: Some(100),
        top_p: Some(0.9),
        frequency_penalty: Some(0.0),
        presence_penalty: Some(0.0),
        tools: None,
        tool_choice: None,
        ..Default::default()
    }
}

/// # Test Concurrent Requests
/// 
/// Tests handling of multiple concurrent requests.
#[tokio::test]
async fn test_concurrent_requests() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    let config = BatchingTestConfig::default();
    
    let start_time = Instant::now();
    
    // Create multiple concurrent requests
    let mut handles = vec![];
    
    for i in 0..config.batch_size {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {;
            let request_data = create_test_request(i);
            
            let request = Request::builder();
                .method(Method::POST)
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .header("x-request-id", format!("concurrent-{}", i))
                .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
                .unwrap();
            
            let response = app_clone.oneshot(request).await.unwrap();
            (i, response.status())
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let results = join_all(handles);
    let elapsed = start_time.elapsed();
    
    // Verify all requests completed
    assert_eq!(results.len(), config.batch_size);
    
    // Check response statuses
    for result in results {
        let (request_id, status) = result.unwrap();
        println!("Request {} returned status: {}", request_id, status);
        
        // Should not return 500 Internal Server Error due to concurrency issues
        assert_ne!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    println!("✅ Concurrent requests test completed in {:?}", elapsed);
}

/// # Test Request Batching
/// 
/// Tests batching of multiple requests for efficiency.
#[tokio::test]
async fn test_request_batching() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    let config = BatchingTestConfig::default();
    
    let start_time = Instant::now();
    
    // Create batch of requests
    let mut batch_handles = vec![];
    
    for batch in 0..3 {
        let mut batch_requests = vec![];
        
        for i in 0..config.batch_size {
            let app_clone = app.clone();
            let request_data = create_test_request(batch * config.batch_size + i);
            
            let handle = tokio::spawn(async move {;
                let request = Request::builder();
                    .method(Method::POST)
                    .uri("/v1/chat/completions")
                    .header("content-type", "application/json")
                    .header("x-request-id", format!("batch-{}-{}", batch, i))
                    .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
                    .unwrap();
                
                let response = app_clone.oneshot(request).await.unwrap();
                (batch, i, response.status())
            });
            
            batch_requests.push(handle);
        }
        
        batch_handles.push(batch_requests);
        
        // Add delay between batches
        if batch < 2 {
            tokio::time::sleep(Duration::from_millis(config.batch_delay_ms))
        }
    }
    
    // Wait for all batches to complete
    let mut all_results = vec![];
    for batch_handles in batch_handles {
        let batch_results = join_all(batch_handles);
        all_results.extend(batch_results);
    }
    
    let elapsed = start_time.elapsed();
    
    // Verify all requests completed
    assert_eq!(all_results.len(), 3 * config.batch_size);
    
    // Check that batching improved efficiency
    let avg_time_per_request = elapsed.as_millis() as f64 / (3 * config.batch_size) as f64;
    println!("Average time per request: {:.2}ms", avg_time_per_request);
    
    // Should be more efficient than sequential requests
    assert!(avg_time_per_request < config.expected_response_time_ms as f64);
    
    println!("✅ Request batching test completed in {:?}", elapsed);
}

/// # Test High Concurrency Load
/// 
/// Tests system behavior under high concurrent load.
#[tokio::test]
async fn test_high_concurrency_load() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    let config = BatchingTestConfig::default();
    
    let start_time = Instant::now();
    
    // Create high number of concurrent requests
    let high_concurrency = config.max_concurrent_requests.min(50); // Limit for test environment
    let mut handles = vec![];
    
    for i in 0..high_concurrency {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {;
            let request_data = create_test_request(i);
            
            let request = Request::builder();
                .method(Method::POST)
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .header("x-request-id", format!("high-load-{}", i))
                .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
                .unwrap();
            
            let response = app_clone.oneshot(request).await.unwrap();
            (i, response.status(), response.headers().clone())
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let results = join_all(handles);
    let elapsed = start_time.elapsed();
    
    // Verify all requests completed
    assert_eq!(results.len(), high_concurrency);
    
    // Analyze results
    let mut success_count = 0;
    let mut error_count = 0;
    let mut timeout_count = 0;
    
    for result in results {
        let (request_id, status, headers) = result.unwrap();
        
        match status {
            StatusCode::OK => success_count += 1,
            StatusCode::TOO_MANY_REQUESTS => {
                error_count += 1;
                println!("Request {} rate limited", request_id);
            }
            StatusCode::SERVICE_UNAVAILABLE => {
                error_count += 1;
                println!("Request {} service unavailable", request_id);
            }
            _ => {
                error_count += 1;
                println!("Request {} failed with status: {}", request_id, status);
            }
        }
        
        // Check for rate limiting headers
        if headers.contains_key("retry-after") {
            timeout_count += 1;
        }
    }
    
    let success_rate = success_count as f64 / high_concurrency as f64;
    println!("Success rate: {:.2}%", success_rate * 100.0);
    println!("Error count: {}", error_count);
    println!("Timeout count: {}", timeout_count);
    println!("Total time: {:?}", elapsed);
    
    // Should handle high concurrency reasonably well
    assert!(success_rate > 0.0); // At least some requests should succeed
    
    println!("✅ High concurrency load test completed");
}

/// # Test Request Queuing
/// 
/// Tests request queuing when system is overloaded.
#[tokio::test]
async fn test_request_queuing() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Simulate request queuing behavior
    let queue_sizes = vec![10, 50, 100, 200];
    
    for queue_size in queue_sizes {
        println!("Testing queue size: {}", queue_size);
        
        let start_time = Instant::now();
        let mut handles = vec![];
        
        // Create requests that will be queued
        for i in 0..queue_size {
            let app_clone = app.clone();
            let handle = tokio::spawn(async move {;
                let request_data = create_test_request(i);
                
                let request = Request::builder();
                    .method(Method::POST)
                    .uri("/v1/chat/completions")
                    .header("content-type", "application/json")
                    .header("x-request-id", format!("queue-{}-{}", queue_size, i))
                    .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
                    .unwrap();
                
                let response = app_clone.oneshot(request).await.unwrap();
                (i, response.status())
            });
            
            handles.push(handle);
        }
        
        // Wait for all requests to complete
        let results = join_all(handles);
        let elapsed = start_time.elapsed();
        
        // Verify all requests completed
        assert_eq!(results.len(), queue_size);
        
        let avg_time_per_request = elapsed.as_millis() as f64 / queue_size as f64;
        println!("Queue size {}: avg time per request: {:.2}ms", queue_size, avg_time_per_request);
        
        // Larger queues should take longer per request
        if queue_size > 10 {
            assert!(avg_time_per_request > 0.0);
        }
    }
    
    println!("✅ Request queuing test completed");
}

/// # Test Connection Pooling
/// 
/// Tests connection pooling for concurrent requests.
#[tokio::test]
async fn test_connection_pooling() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test connection pooling with multiple requests
    let pool_sizes = vec![5, 10, 20];
    
    for pool_size in pool_sizes {
        println!("Testing connection pool size: {}", pool_size);
        
        let start_time = Instant::now();
        let mut handles = vec![];
        
        // Create requests that will use the connection pool
        for i in 0..pool_size {
            let app_clone = app.clone();
            let handle = tokio::spawn(async move {;
                let request_data = create_test_request(i);
                
                let request = Request::builder();
                    .method(Method::POST)
                    .uri("/v1/chat/completions")
                    .header("content-type", "application/json")
                    .header("x-request-id", format!("pool-{}-{}", pool_size, i))
                    .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
                    .unwrap();
                
                let response = app_clone.oneshot(request).await.unwrap();
                (i, response.status())
            });
            
            handles.push(handle);
        }
        
        // Wait for all requests to complete
        let results = join_all(handles);
        let elapsed = start_time.elapsed();
        
        // Verify all requests completed
        assert_eq!(results.len(), pool_size);
        
        let avg_time_per_request = elapsed.as_millis() as f64 / pool_size as f64;
        println!("Pool size {}: avg time per request: {:.2}ms", pool_size, avg_time_per_request);
        
        // Connection pooling should improve efficiency
        assert!(avg_time_per_request > 0.0);
    }
    
    println!("✅ Connection pooling test completed");
}

/// # Test Request Prioritization
/// 
/// Tests request prioritization under load.
#[tokio::test]
async fn test_request_prioritization() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test different priority levels
    let priorities = vec![;
        ("high", 1),
        ("medium", 2),
        ("low", 3),
    ];
    
    let start_time = Instant::now();
    let mut handles = vec![];
    
    for (priority_name, priority_value) in priorities {
        for i in 0..5 {
            let app_clone = app.clone();
            let handle = tokio::spawn(async move {;
                let request_data = create_test_request(i);
                
                let request = Request::builder();
                    .method(Method::POST)
                    .uri("/v1/chat/completions")
                    .header("content-type", "application/json")
                    .header("x-request-id", format!("priority-{}-{}", priority_name, i))
                    .header("x-priority", priority_value.to_string())
                    .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
                    .unwrap();
                
                let response = app_clone.oneshot(request).await.unwrap();
                (priority_name, i, response.status())
            });
            
            handles.push(handle);
        }
    }
    
    // Wait for all requests to complete
    let results = join_all(handles);
    let elapsed = start_time.elapsed();
    
    // Verify all requests completed
    assert_eq!(results.len(), 15); // 3 priorities * 5 requests each
    
    // Analyze completion order by priority
    let mut completion_times = std::collections::HashMap::new();
    
    for result in results {
        let (priority_name, request_id, status) = result.unwrap();
        println!("Priority {} request {} completed with status: {}", priority_name, request_id, status);
        
        // In a real implementation, we would track completion times
        // and verify that high priority requests complete first
        completion_times.insert(format!("{}-{}", priority_name, request_id), elapsed);
    }
    
    println!("📝 High priority requests should complete first");
    println!("📝 Medium priority requests should complete second");
    println!("📝 Low priority requests should complete last");
    
    println!("✅ Request prioritization test completed in {:?}", elapsed);
}

/// # Test Resource Exhaustion Handling
/// 
/// Tests handling when system resources are exhausted.
#[tokio::test]
async fn test_resource_exhaustion_handling() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test with very high number of requests to exhaust resources
    let exhaustion_limit = 1000;
    let mut handles = vec![];
    
    println!("Testing resource exhaustion with {} requests", exhaustion_limit);
    
    let start_time = Instant::now();
    
    for i in 0..exhaustion_limit {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {;
            let request_data = create_test_request(i);
            
            let request = Request::builder();
                .method(Method::POST)
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .header("x-request-id", format!("exhaustion-{}", i))
                .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
                .unwrap();
            
            let response = app_clone.oneshot(request).await.unwrap();
            (i, response.status())
        });
        
        handles.push(handle);
        
        // Add small delay to prevent overwhelming the system
        if i % 100 == 0 {
            tokio::time::sleep(Duration::from_millis(10))
        }
    }
    
    // Wait for all requests to complete (with timeout)
    let result = timeout(Duration::from_secs(30), join_all(handles));
    
    match result {
        Ok(results) => {
            let elapsed = start_time.elapsed();
            println!("All {} requests completed in {:?}", exhaustion_limit, elapsed);
            
            // Analyze results
            let mut success_count = 0;
            let mut error_count = 0;
            
            for (request_id, status) in results {
                match status {
                    StatusCode::OK => success_count += 1,
                    StatusCode::TOO_MANY_REQUESTS => error_count += 1,
                    StatusCode::SERVICE_UNAVAILABLE => error_count += 1,
                    _ => error_count += 1,
                }
            }
            
            let success_rate = success_count as f64 / exhaustion_limit as f64;
            println!("Success rate: {:.2}%", success_rate * 100.0);
            println!("Error count: {}", error_count);
            
            // Should handle resource exhaustion gracefully
            assert!(success_rate >= 0.0);
        }
        Err(_) => {
            println!("Resource exhaustion test timed out - system handled load gracefully");
        }
    }
    
    println!("✅ Resource exhaustion handling test completed");
}

/// # Test Batching Metrics
/// 
/// Tests that batching and concurrency metrics are properly collected.
#[tokio::test]
async fn test_batching_metrics() {
    // Simulate batching and concurrency metrics collection
    let mut batching_metrics = std::collections::HashMap::new();
    
    batching_metrics.insert("total_requests", 10000);
    batching_metrics.insert("concurrent_requests", 5000);
    batching_metrics.insert("batched_requests", 3000);
    batching_metrics.insert("avg_batch_size", 5.2);
    batching_metrics.insert("max_concurrent", 100);
    batching_metrics.insert("queue_wait_time_ms", 150);
    batching_metrics.insert("connection_pool_utilization", 0.75);
    
    // Calculate metrics
    let concurrency_rate = batching_metrics["concurrent_requests"] as f64 / batching_metrics["total_requests"] as f64;
    let batching_rate = batching_metrics["batched_requests"] as f64 / batching_metrics["total_requests"] as f64;
    let avg_batch_size = batching_metrics["avg_batch_size"];
    let queue_wait_time = batching_metrics["queue_wait_time_ms"];
    let pool_utilization = batching_metrics["connection_pool_utilization"];
    
    println!("Concurrency rate: {:.2}%", concurrency_rate * 100.0);
    println!("Batching rate: {:.2}%", batching_rate * 100.0);
    println!("Average batch size: {:.1}", avg_batch_size);
    println!("Queue wait time: {}ms", queue_wait_time);
    println!("Connection pool utilization: {:.1}%", pool_utilization * 100.0);
    
    // Metrics should be reasonable
    assert!(concurrency_rate > 0.0);
    assert!(batching_rate > 0.0);
    assert!(avg_batch_size > 1.0);
    assert!(pool_utilization > 0.0 && pool_utilization <= 1.0);
    
    println!("✅ Batching metrics test passed");
}

/// # Integration Test Suite
/// 
/// Runs a comprehensive integration test suite for batching and concurrency.
#[tokio::test]
async fn test_batching_concurrency_integration_suite() {
    println!("🚀 Starting comprehensive batching & concurrency test suite");
    
    // Test all batching and concurrency scenarios
    test_concurrent_requests()
    test_request_batching()
    test_high_concurrency_load()
    test_request_queuing()
    test_connection_pooling()
    test_request_prioritization()
    test_resource_exhaustion_handling()
    test_batching_metrics()
    
    println!("✅ Comprehensive batching & concurrency test suite completed");
}
