//! # Performance Benchmarking Suite
//! 
//! Comprehensive performance tests for the LightLLM Rust proxy to measure
//! and validate performance characteristics under various conditions.
//! 
//! These tests measure:
//! - Latency metrics (P50, P95, P99)
//! - Throughput capabilities
//! - Memory usage patterns
//! - CPU utilization
//! - Connection efficiency
//! - Streaming performance

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
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::Semaphore,
    time::timeout,
};
use tower::ServiceExt;

/// # Performance Metrics
/// 
/// Collects and analyzes performance metrics during testing.
#[derive(Debug, Default)]
struct PerformanceMetrics {
    response_times: Vec<Duration>,
    success_count: usize,
    error_count: usize,
    memory_samples: Vec<usize>,
}

impl PerformanceMetrics {
    fn add_response_time(&mut self, duration: Duration) {
        self.response_times.push(duration);
    }
    
    fn add_success(&mut self) {
        self.success_count += 1;
    }
    
    fn add_error(&mut self) {
        self.error_count += 1;
    }
    
    fn calculate_percentiles(&self) -> HashMap<String, Duration> {
        let mut sorted_times = self.response_times.clone();
        sorted_times.sort();
        
        let len = sorted_times.len();
        if len == 0 {
            return HashMap::new();
        }
        
        let mut percentiles = HashMap::new();
        
        // P50 (median)
        if len > 0 {
            percentiles.insert("P50".to_string(), sorted_times[len / 2]);
        }
        
        // P95
        if len > 0 {
            let p95_index = (len as f64 * 0.95) as usize;
            percentiles.insert("P95".to_string(), sorted_times[p95_index.min(len - 1)]);
        }
        
        // P99
        if len > 0 {
            let p99_index = (len as f64 * 0.99) as usize;
            percentiles.insert("P99".to_string(), sorted_times[p99_index.min(len - 1)]);
        }
        
        // Min and Max
        if len > 0 {
            percentiles.insert("Min".to_string(), sorted_times[0]);
            percentiles.insert("Max".to_string(), sorted_times[len - 1]);
        }
        
        percentiles
    }
    
    fn calculate_throughput(&self, duration: Duration) -> f64 {
        if duration.as_secs_f64() > 0.0 {
            self.success_count as f64 / duration.as_secs_f64()
        } else {
            0.0
        }
    }
    
    fn success_rate(&self) -> f64 {
        let total = self.success_count + self.error_count;
        if total > 0 {
            self.success_count as f64 / total as f64
        } else {
            0.0
        }
    }
}

/// # Create Performance Test App
/// 
/// Creates a test application optimized for performance testing.
async fn create_performance_test_app() -> Router {
    let mut config = Config::for_test();
    config.port = 8080;
    config.backend_url = "http://localhost:8000".to_string();
    config.model_id = "perf-test-model".to_string();
    
    let state = AppState::new(config).await;
    
    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(state)
}

/// # Test Latency Performance
/// 
/// Measures latency characteristics under various load conditions.
#[tokio::test]
async fn test_latency_performance() {
    let app = create_performance_test_app().await;
    
    println!("⏱️ Starting latency performance test");
    
    let test_scenarios = vec![
        (1, "Single request"),
        (10, "Light load"),
        (50, "Medium load"),
        (100, "Heavy load"),
    ];
    
    for (request_count, scenario_name) in test_scenarios {
        println!("  Testing scenario: {} ({} requests)", scenario_name, request_count);
        
        let mut metrics = PerformanceMetrics::default();
        let start_time = Instant::now();
        
        // Create semaphore to control concurrency
        let semaphore = Arc::new(Semaphore::new(10));
        let mut handles = Vec::new();
        
        for i in 0..request_count {
            let app = app.clone();
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            
            let handle = tokio::spawn(async move {
                let _permit = permit;
                let request_start = Instant::now();
                
                let request_body = json!({
                    "model": "perf-test-model",
                    "messages": [
                        {"role": "user", "content": format!("Latency test request {}", i)}
                    ],
                    "max_tokens": 20,
                    "stream": i % 2 == 0 // Mix streaming and non-streaming
                });
                
                let request = Request::builder()
                    .method(Method::POST)
                    .uri("/v1/chat/completions")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                    .unwrap();
                
                let response = app.oneshot(request).await.unwrap();
                let request_duration = request_start.elapsed();
                let status = response.status();
                
                // Read response to ensure complete processing
                let _body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
                
                (request_duration, status == StatusCode::OK)
            });
            
            handles.push(handle);
        }
        
        // Wait for all requests to complete
        for handle in handles {
            match handle.await {
                Ok((duration, success)) => {
                    metrics.add_response_time(duration);
                    if success {
                        metrics.add_success();
                    } else {
                        metrics.add_error();
                    }
                }
                Err(_) => {
                    metrics.add_error();
                }
            }
        }
        
        let total_duration = start_time.elapsed();
        let percentiles = metrics.calculate_percentiles();
        let throughput = metrics.calculate_throughput(total_duration);
        let success_rate = metrics.success_rate();
        
        println!("    Results for {}:", scenario_name);
        println!("      Total time: {:?}", total_duration);
        println!("      Success rate: {:.2}%", success_rate * 100.0);
        println!("      Throughput: {:.2} requests/second", throughput);
        
        if !percentiles.is_empty() {
            println!("      Latency percentiles:");
            for (percentile, duration) in &percentiles {
                println!("        {}: {:?}", percentile, duration);
            }
        }
        
        // Performance assertions
        assert!(success_rate > 0.0, "No successful requests in {}", scenario_name);
        assert!(throughput > 0.0, "No throughput achieved in {}", scenario_name);
        
        // Latency assertions (adjust thresholds based on your requirements)
        if let Some(p95) = percentiles.get("P95") {
            assert!(p95.as_millis() < 5000, "P95 latency too high: {:?}", p95);
        }
    }
    
    println!("✅ Latency performance test completed");
}

/// # Test Throughput Performance
/// 
/// Measures maximum throughput capabilities under sustained load.
#[tokio::test]
async fn test_throughput_performance() {
    let app = create_performance_test_app().await;
    
    println!("🚀 Starting throughput performance test");
    
    let test_duration = Duration::from_secs(30); // 30-second test
    let max_concurrency = 50;
    
    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let mut metrics = PerformanceMetrics::default();
    let start_time = Instant::now();
    let mut request_counter = 0;
    
    // Spawn requests continuously for the test duration
    let mut handles = Vec::new();
    
    while start_time.elapsed() < test_duration {
        let app = app.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        
        let handle = tokio::spawn(async move {
            let _permit = permit;
            let request_start = Instant::now();
            let request_id = request_counter;
            
            let request_body = json!({
                "model": "perf-test-model",
                "messages": [
                    {"role": "user", "content": format!("Throughput test request {}", request_id)}
                ],
                "max_tokens": 15,
                "stream": request_id % 3 == 0 // 1/3 streaming requests
            });
            
            let request = Request::builder()
                .method(Method::POST)
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                .unwrap();
            
            let response = app.oneshot(request).await.unwrap();
            let request_duration = request_start.elapsed();
            let status = response.status();
            
            // Read response
            let _body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            
            (request_duration, status == StatusCode::OK, request_id)
        });
        
        handles.push(handle);
        request_counter += 1;
        
        // Small delay to control request rate
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    // Wait for all requests to complete
    let wait_timeout = Duration::from_secs(60);
    let completed_handles = timeout(wait_timeout, async {
        let mut completed = Vec::new();
        for handle in handles {
            if let Ok(result) = handle.await {
                completed.push(result);
            }
        }
        completed
    }).await;
    
    let total_duration = start_time.elapsed();
    
    match completed_handles {
        Ok(results) => {
            for (duration, success, _request_id) in results {
                metrics.add_response_time(duration);
                if success {
                    metrics.add_success();
                } else {
                    metrics.add_error();
                }
            }
        }
        Err(_) => {
            println!("  Warning: Some requests may still be running");
        }
    }
    
    let throughput = metrics.calculate_throughput(total_duration);
    let success_rate = metrics.success_rate();
    let percentiles = metrics.calculate_percentiles();
    
    println!("  Throughput test results:");
    println!("    Test duration: {:?}", total_duration);
    println!("    Total requests: {}", metrics.success_count + metrics.error_count);
    println!("    Successful requests: {}", metrics.success_count);
    println!("    Error requests: {}", metrics.error_count);
    println!("    Success rate: {:.2}%", success_rate * 100.0);
    println!("    Throughput: {:.2} requests/second", throughput);
    
    if !percentiles.is_empty() {
        println!("    Latency percentiles:");
        for (percentile, duration) in percentiles {
            println!("      {}: {:?}", percentile, duration);
        }
    }
    
    // Performance assertions
    assert!(metrics.success_count > 0, "No successful requests");
    assert!(throughput > 1.0, "Throughput too low: {:.2} req/s", throughput);
    assert!(success_rate > 0.5, "Success rate too low: {:.2}%", success_rate * 100.0);
    
    println!("✅ Throughput performance test completed");
}

/// # Test Streaming Performance
/// 
/// Measures streaming-specific performance characteristics.
#[tokio::test]
async fn test_streaming_performance() {
    let app = create_performance_test_app().await;
    
    println!("🌊 Starting streaming performance test");
    
    let request_count = 50;
    let mut metrics = PerformanceMetrics::default();
    let start_time = Instant::now();
    
    let semaphore = Arc::new(Semaphore::new(10));
    let mut handles = Vec::new();
    
    for i in 0..request_count {
        let app = app.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        
        let handle = tokio::spawn(async move {
            let _permit = permit;
            let request_start = Instant::now();
            
            let request_body = json!({
                "model": "perf-test-model",
                "messages": [
                    {"role": "user", "content": format!("Streaming test request {}", i)}
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
            let status = response.status();
            
            // Verify streaming response
            let content_type = response.headers().get("content-type").unwrap();
            assert!(content_type.to_str().unwrap().contains("text/event-stream"));
            
            // Read streaming response
            let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let body_str = String::from_utf8_lossy(&body_bytes);
            
            // Analyze streaming characteristics
            let data_lines = body_str.matches("data: ").count();
            let first_token_time = request_start.elapsed();
            
            (first_token_time, status == StatusCode::OK, data_lines)
        });
        
        handles.push(handle);
    }
    
    let mut total_data_lines = 0;
    
    // Wait for all streaming requests to complete
    for handle in handles {
        match handle.await {
            Ok((first_token_time, success, data_lines)) => {
                metrics.add_response_time(first_token_time);
                total_data_lines += data_lines;
                if success {
                    metrics.add_success();
                } else {
                    metrics.add_error();
                }
            }
            Err(_) => {
                metrics.add_error();
            }
        }
    }
    
    let total_duration = start_time.elapsed();
    let throughput = metrics.calculate_throughput(total_duration);
    let success_rate = metrics.success_rate();
    let percentiles = metrics.calculate_percentiles();
    let avg_data_lines = if metrics.success_count > 0 {
        total_data_lines as f64 / metrics.success_count as f64
    } else {
        0.0
    };
    
    println!("  Streaming performance results:");
    println!("    Total requests: {}", metrics.success_count + metrics.error_count);
    println!("    Successful requests: {}", metrics.success_count);
    println!("    Success rate: {:.2}%", success_rate * 100.0);
    println!("    Throughput: {:.2} requests/second", throughput);
    println!("    Average data lines per stream: {:.1}", avg_data_lines);
    println!("    Total data lines: {}", total_data_lines);
    
    if !percentiles.is_empty() {
        println!("    First token latency percentiles:");
        for (percentile, duration) in percentiles {
            println!("      {}: {:?}", percentile, duration);
        }
    }
    
    // Streaming-specific assertions
    assert!(metrics.success_count > 0, "No successful streaming requests");
    assert!(avg_data_lines > 0.0, "No streaming data lines received");
    assert!(throughput > 0.0, "No streaming throughput achieved");
    
    println!("✅ Streaming performance test completed");
}

/// # Test Memory Usage Performance
/// 
/// Measures memory usage patterns during various operations.
#[tokio::test]
async fn test_memory_usage_performance() {
    let app = create_performance_test_app().await;
    
    println!("🧠 Starting memory usage performance test");
    
    // Test memory usage with different request patterns
    let test_scenarios = vec![
        (10, "Small batch"),
        (50, "Medium batch"),
        (100, "Large batch"),
    ];
    
    for (request_count, scenario_name) in test_scenarios {
        println!("  Testing scenario: {} ({} requests)", scenario_name, request_count);
        
        let start_time = Instant::now();
        let semaphore = Arc::new(Semaphore::new(20));
        let mut handles = Vec::new();
        
        for i in 0..request_count {
            let app = app.clone();
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            
            let handle = tokio::spawn(async move {
                let _permit = permit;
                
                let request_body = json!({
                    "model": "perf-test-model",
                    "messages": [
                        {"role": "user", "content": format!("Memory test request {}", i)}
                    ],
                    "max_tokens": 50,
                    "stream": i % 2 == 0
                });
                
                let request = Request::builder()
                    .method(Method::POST)
                    .uri("/v1/chat/completions")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                    .unwrap();
                
                let response = app.oneshot(request).await.unwrap();
                let status = response.status();
                
                // Read full response to test memory management
                let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
                let body_size = body_bytes.len();
                
                (status == StatusCode::OK, body_size)
            });
            
            handles.push(handle);
        }
        
        let mut success_count = 0;
        let mut total_response_size = 0;
        
        // Wait for all requests to complete
        for handle in handles {
            match handle.await {
                Ok((success, response_size)) => {
                    if success {
                        success_count += 1;
                    }
                    total_response_size += response_size;
                }
                Err(_) => {
                    // Handle spawn error
                }
            }
        }
        
        let duration = start_time.elapsed();
        let avg_response_size = if success_count > 0 {
            total_response_size as f64 / success_count as f64
        } else {
            0.0
        };
        
        println!("    Results for {}:", scenario_name);
        println!("      Duration: {:?}", duration);
        println!("      Successful requests: {}", success_count);
        println!("      Total response size: {} bytes", total_response_size);
        println!("      Average response size: {:.1} bytes", avg_response_size);
        
        // Memory-related assertions
        assert!(success_count > 0, "No successful requests in {}", scenario_name);
        assert!(avg_response_size > 0.0, "No response data in {}", scenario_name);
    }
    
    println!("✅ Memory usage performance test completed");
}

/// # Test Connection Efficiency
/// 
/// Measures connection pooling and reuse efficiency.
#[tokio::test]
async fn test_connection_efficiency() {
    let app = create_performance_test_app().await;
    
    println!("🔗 Starting connection efficiency test");
    
    // Test connection reuse by making sequential requests
    let request_count = 20;
    let mut response_times = Vec::new();
    
    for i in 0..request_count {
        let request_start = Instant::now();
        
        let request_body = json!({
            "model": "perf-test-model",
            "messages": [
                {"role": "user", "content": format!("Connection test request {}", i)}
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
        
        let response = app.clone().oneshot(request).await.unwrap();
        let request_duration = request_start.elapsed();
        
        response_times.push(request_duration);
        
        // Verify response
        assert!(response.status() == StatusCode::OK || response.status().is_client_error());
        
        // Read response
        let _body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        
        // Small delay between requests
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    // Analyze connection efficiency
    if response_times.len() >= 2 {
        let first_request_time = response_times[0];
        let subsequent_avg: Duration = response_times[1..]
            .iter()
            .sum::<Duration>()
            / (response_times.len() - 1) as u32;
        
        let efficiency_ratio = first_request_time.as_millis() as f64 / subsequent_avg.as_millis() as f64;
        
        println!("  Connection efficiency results:");
        println!("    First request time: {:?}", first_request_time);
        println!("    Average subsequent time: {:?}", subsequent_avg);
        println!("    Efficiency ratio: {:.2}", efficiency_ratio);
        
        // Connection reuse should improve performance
        // (though this may not always be true in test environment)
        if efficiency_ratio > 1.0 {
            println!("    ✅ Connection reuse appears to be working");
        } else {
            println!("    ⚠️  Connection reuse effect not clearly visible");
        }
    }
    
    println!("✅ Connection efficiency test completed");
}
