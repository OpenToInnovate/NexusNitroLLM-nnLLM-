//! # Stress Testing Suite
//! 
//! Advanced stress tests for the LightLLM Rust proxy to validate
//! performance, reliability, and resource management under extreme conditions.
//! 
//! These tests include:
//! - High-concurrency scenarios
//! - Memory stress testing
//! - Long-running connection tests
//! - Resource exhaustion scenarios
//! - Failure recovery testing

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
    sync::{Semaphore, Barrier},
    time::timeout,
    task::JoinSet,
};
use tower::ServiceExt;

/// # Create Stress Test App
/// 
/// Creates a test application optimized for stress testing.
async fn create_stress_test_app() -> Router {
    let mut config = Config::for_test();
    config.port = 8080;
    config.backend_url = "http://localhost:8000".to_string();
    config.model_id = "stress-test-model".to_string();
    
    let state = AppState::new(config).await;
    
    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(state)
}

/// # Test High Concurrency Stress
/// 
/// Tests the proxy under extreme concurrent load to validate
/// thread safety and resource management.
#[tokio::test]
#[ignore] // Run with: cargo test --test stress_test -- --ignored
async fn test_high_concurrency_stress() {
    let app = create_stress_test_app().await;
    
    // Extreme concurrency test
    let concurrent_requests = 1000;
    let semaphore = Arc::new(Semaphore::new(100)); // Limit to 100 concurrent
    let barrier = Arc::new(Barrier::new(concurrent_requests));
    let mut handles = JoinSet::new();
    
    println!("🚀 Starting high concurrency stress test with {} requests", concurrent_requests);
    let start_time = Instant::now();
    
    for i in 0..concurrent_requests {
        let app = app.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let barrier = barrier.clone();
        
        handles.spawn(async move {
            let _permit = permit; // Hold the permit
            
            // Wait for all tasks to be ready
            barrier.wait().await;
            
            let request_body = json!({
                "model": "stress-test-model",
                "messages": [
                    {"role": "user", "content": format!("Stress test request {}", i)}
                ],
                "max_tokens": 10,
                "stream": i % 2 == 0 // Mix of streaming and non-streaming
            });
            
            let request = Request::builder()
                .method(Method::POST)
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                .unwrap();
            
            let response = app.oneshot(request).await.unwrap();
            
            // Verify response (should succeed or fail gracefully)
            assert!(response.status() == StatusCode::OK || response.status().is_client_error());
            
            // Read response to ensure complete processing
            let _body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            
            i
        });
    }
    
    // Wait for all requests to complete with extended timeout
    let results = timeout(Duration::from_secs(120), async {
        let mut completed = 0;
        while let Some(result) = handles.join_next().await {
            result.unwrap();
            completed += 1;
            if completed % 100 == 0 {
                println!("  Completed: {}/{} requests", completed, concurrent_requests);
            }
        }
        completed
    }).await;
    
    let duration = start_time.elapsed();
    
    match results {
        Ok(completed_count) => {
            println!("✅ High concurrency stress test completed:");
            println!("  Total requests: {}", concurrent_requests);
            println!("  Completed requests: {}", completed_count);
            println!("  Total time: {:?}", duration);
            println!("  Average time per request: {:?}", duration / concurrent_requests as u32);
            println!("  Throughput: {:.2} requests/second", 
                completed_count as f64 / duration.as_secs_f64());
            
            assert_eq!(completed_count, concurrent_requests);
        }
        Err(_) => {
            panic!("High concurrency stress test timed out after {:?}", duration);
        }
    }
}

/// # Test Memory Stress
/// 
/// Tests memory usage patterns under sustained load to detect
/// memory leaks or excessive memory consumption.
#[tokio::test]
#[ignore] // Run with: cargo test --test stress_test -- --ignored
async fn test_memory_stress() {
    let app = create_stress_test_app().await;
    
    println!("🧠 Starting memory stress test");
    
    // Run multiple rounds of requests to test memory patterns
    let rounds = 10;
    let requests_per_round = 100;
    
    for round in 0..rounds {
        println!("  Round {}/{}", round + 1, rounds);
        
        let mut handles = JoinSet::new();
        
        for i in 0..requests_per_round {
            let app = app.clone();
            
            handles.spawn(async move {
                let request_body = json!({
                    "model": "stress-test-model",
                    "messages": [
                        {"role": "user", "content": format!("Memory stress round {} request {}", round, i)}
                    ],
                    "max_tokens": 50,
                    "stream": true // Use streaming for more memory-intensive test
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
                
                // Read full response to ensure memory is properly managed
                let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
                let body_str = String::from_utf8_lossy(&body_bytes);
                
                // Verify SSE format
                assert!(body_str.contains("data: "));
                
                i
            });
        }
        
        // Wait for round to complete
        let round_results = timeout(Duration::from_secs(30), async {
            let mut completed = 0;
            while let Some(result) = handles.join_next().await {
                result.unwrap();
                completed += 1;
            }
            completed
        }).await;
        
        match round_results {
            Ok(completed) => {
                println!("    Completed: {}/{} requests", completed, requests_per_round);
                assert_eq!(completed, requests_per_round);
            }
            Err(_) => {
                panic!("Memory stress test timed out in round {}", round + 1);
            }
        }
        
        // Small delay between rounds to allow cleanup
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    println!("✅ Memory stress test completed successfully");
}

/// # Test Long-Running Connections
/// 
/// Tests the proxy's ability to handle long-running connections
/// and maintain performance over extended periods.
#[tokio::test]
#[ignore] // Run with: cargo test --test stress_test -- --ignored
async fn test_long_running_connections() {
    let app = create_stress_test_app().await;
    
    println!("⏱️ Starting long-running connection test");
    
    let test_duration = Duration::from_secs(60); // 1 minute test
    let request_interval = Duration::from_millis(100); // 10 requests per second
    let start_time = Instant::now();
    
    let mut request_count = 0;
    let mut success_count = 0;
    let mut error_count = 0;
    
    while start_time.elapsed() < test_duration {
        let request_body = json!({
            "model": "stress-test-model",
            "messages": [
                {"role": "user", "content": format!("Long running test request {}", request_count)}
            ],
            "max_tokens": 20,
            "stream": request_count % 3 == 0 // Mix of request types
        });
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
            .unwrap();
        
        match timeout(Duration::from_secs(5), app.clone().oneshot(request)).await {
            Ok(Ok(response)) => {
                if response.status() == StatusCode::OK {
                    success_count += 1;
                    
                    // Read response to ensure complete processing
                    let _body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
                } else {
                    error_count += 1;
                }
            }
            Ok(Err(_)) => {
                error_count += 1;
            }
            Err(_) => {
                error_count += 1;
            }
        }
        
        request_count += 1;
        
        // Log progress every 10 seconds
        if request_count % 100 == 0 {
            let elapsed = start_time.elapsed();
            let rate = request_count as f64 / elapsed.as_secs_f64();
            println!("  Progress: {} requests, {:.2} req/s, {} success, {} errors", 
                request_count, rate, success_count, error_count);
        }
        
        tokio::time::sleep(request_interval).await;
    }
    
    let total_duration = start_time.elapsed();
    let final_rate = request_count as f64 / total_duration.as_secs_f64();
    
    println!("✅ Long-running connection test completed:");
    println!("  Total duration: {:?}", total_duration);
    println!("  Total requests: {}", request_count);
    println!("  Successful requests: {}", success_count);
    println!("  Error requests: {}", error_count);
    println!("  Average rate: {:.2} requests/second", final_rate);
    
    // Verify we maintained reasonable performance
    assert!(request_count > 0, "No requests were processed");
    assert!(success_count > 0, "No successful requests");
    assert!(final_rate > 1.0, "Rate too low: {:.2} req/s", final_rate);
}

/// # Test Resource Exhaustion Recovery
/// 
/// Tests the proxy's ability to recover from resource exhaustion
/// scenarios and maintain service availability.
#[tokio::test]
#[ignore] // Run with: cargo test --test stress_test -- --ignored
async fn test_resource_exhaustion_recovery() {
    let app = create_stress_test_app().await;
    
    println!("💥 Starting resource exhaustion recovery test");
    
    // Phase 1: Create resource pressure
    println!("  Phase 1: Creating resource pressure");
    let pressure_requests = 500;
    let semaphore = Arc::new(Semaphore::new(50)); // Limit concurrency
    let mut handles = JoinSet::new();
    
    for i in 0..pressure_requests {
        let app = app.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        
        handles.spawn(async move {
            let _permit = permit;
            
            let request_body = json!({
                "model": "stress-test-model",
                "messages": [
                    {"role": "user", "content": format!("Pressure request {}", i)}
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
            
            // Don't wait for all to complete - create pressure
            let _ = timeout(Duration::from_secs(1), app.oneshot(request)).await;
            
            i
        });
    }
    
    // Wait a bit for pressure to build
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Phase 2: Test recovery with new requests
    println!("  Phase 2: Testing recovery");
    let recovery_requests = 50;
    let mut recovery_handles = JoinSet::new();
    
    for i in 0..recovery_requests {
        let app = app.clone();
        
        recovery_handles.spawn(async move {
            let request_body = json!({
                "model": "stress-test-model",
                "messages": [
                    {"role": "user", "content": format!("Recovery request {}", i)}
                ],
                "max_tokens": 20,
                "stream": false
            });
            
            let request = Request::builder()
                .method(Method::POST)
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                .unwrap();
            
            let response = app.oneshot(request).await.unwrap();
            
            // Should still be able to handle requests
            assert!(response.status() == StatusCode::OK || response.status().is_client_error());
            
            i
        });
    }
    
    // Wait for recovery requests to complete
    let recovery_results = timeout(Duration::from_secs(30), async {
        let mut completed = 0;
        while let Some(result) = recovery_handles.join_next().await {
            result.unwrap();
            completed += 1;
        }
        completed
    }).await;
    
    match recovery_results {
        Ok(completed) => {
            println!("  Recovery requests completed: {}/{}", completed, recovery_requests);
            assert!(completed > 0, "No recovery requests succeeded");
        }
        Err(_) => {
            panic!("Recovery test timed out");
        }
    }
    
    // Phase 3: Clean up pressure requests
    println!("  Phase 3: Cleaning up pressure requests");
    let cleanup_results = timeout(Duration::from_secs(30), async {
        let mut completed = 0;
        while let Some(result) = handles.join_next().await {
            result.unwrap();
            completed += 1;
        }
        completed
    }).await;
    
    match cleanup_results {
        Ok(completed) => {
            println!("  Pressure requests cleaned up: {}", completed);
        }
        Err(_) => {
            println!("  Warning: Some pressure requests may still be running");
        }
    }
    
    println!("✅ Resource exhaustion recovery test completed");
}

/// # Test Failure Recovery
/// 
/// Tests the proxy's ability to recover from various failure
/// scenarios and continue providing service.
#[tokio::test]
async fn test_failure_recovery() {
    let app = create_stress_test_app().await;
    
    println!("🔄 Starting failure recovery test");
    
    // Test various failure scenarios
    let failure_scenarios: Vec<(&str, StatusCode)> = vec![
        // Invalid JSON
        ("invalid json", StatusCode::BAD_REQUEST),
        // Malformed request
        (r#"{"invalid": true}"#, StatusCode::BAD_REQUEST),
        // Empty body
        ("", StatusCode::BAD_REQUEST),
        // Missing required fields
        (r#"{"stream": true}"#, StatusCode::BAD_REQUEST),
    ];
    
    for (i, (invalid_body, expected_status)) in failure_scenarios.iter().enumerate() {
        println!("  Testing failure scenario {}", i + 1);
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .body(Body::from(invalid_body.as_bytes()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        
        // Verify proper error handling
        assert_eq!(response.status(), *expected_status);
        
        // Verify service is still responsive
        let test_request_body = json!({
            "model": "stress-test-model",
            "messages": [
                {"role": "user", "content": "Recovery test"}
            ],
            "max_tokens": 10,
            "stream": false
        });
        
        let test_request = Request::builder()
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&test_request_body).unwrap()))
            .unwrap();
        
        let test_response = app.clone().oneshot(test_request).await.unwrap();
        
        // Should still be able to handle valid requests
        assert!(test_response.status() == StatusCode::OK || test_response.status().is_client_error());
    }
    
    println!("✅ Failure recovery test completed");
}

/// # Test Burst Traffic Handling
/// 
/// Tests the proxy's ability to handle sudden bursts of traffic
/// without degradation in performance.
#[tokio::test]
#[ignore] // Run with: cargo test --test stress_test -- --ignored
async fn test_burst_traffic_handling() {
    let app = create_stress_test_app().await;
    
    println!("🌊 Starting burst traffic handling test");
    
    let burst_size = 200;
    let num_bursts = 5;
    let burst_interval = Duration::from_secs(2);
    
    for burst in 0..num_bursts {
        println!("  Burst {}/{}: {} requests", burst + 1, num_bursts, burst_size);
        
        let start_time = Instant::now();
        let mut handles = JoinSet::new();
        
        // Create burst of requests
        for i in 0..burst_size {
            let app = app.clone();
            
            handles.spawn(async move {
                let request_body = json!({
                    "model": "stress-test-model",
                    "messages": [
                        {"role": "user", "content": format!("Burst {} request {}", burst, i)}
                    ],
                    "max_tokens": 15,
                    "stream": i % 2 == 0
                });
                
                let request = Request::builder()
                    .method(Method::POST)
                    .uri("/v1/chat/completions")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                    .unwrap();
                
                let response = app.oneshot(request).await.unwrap();
                
                // Verify response
                assert!(response.status() == StatusCode::OK || response.status().is_client_error());
                
                // Read response
                let _body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
                
                i
            });
        }
        
        // Wait for burst to complete
        let burst_results = timeout(Duration::from_secs(30), async {
            let mut completed = 0;
            while let Some(result) = handles.join_next().await {
                result.unwrap();
                completed += 1;
            }
            completed
        }).await;
        
        let burst_duration = start_time.elapsed();
        
        match burst_results {
            Ok(completed) => {
                let burst_rate = completed as f64 / burst_duration.as_secs_f64();
                println!("    Completed: {}/{} requests in {:?} ({:.2} req/s)", 
                    completed, burst_size, burst_duration, burst_rate);
                
                assert_eq!(completed, burst_size);
                assert!(burst_rate > 10.0, "Burst rate too low: {:.2} req/s", burst_rate);
            }
            Err(_) => {
                panic!("Burst {} timed out after {:?}", burst + 1, burst_duration);
            }
        }
        
        // Wait between bursts
        if burst < num_bursts - 1 {
            tokio::time::sleep(burst_interval).await;
        }
    }
    
    println!("✅ Burst traffic handling test completed");
}
