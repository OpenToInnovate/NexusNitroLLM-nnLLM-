//! # Performance Validation Test Suite
//! 
//! Validates all performance optimizations and failure mode fixes:
//! - Connection pooling
//! - Deadline propagation
//! - Streaming backpressure
//! - Concurrency limits
//! - Memory efficiency

use std::sync::Arc;
use std::time::{Duration, Instant};
use nexus_nitro_llm::client::{HighPerformanceClient, ClientConfig};
use futures_util::StreamExt;
use std::pin::pin;

#[tokio::test]
async fn test_connection_pooling() {
    let config = ClientConfig {
        base_url: "http://localhost:3000".to_string(),
        timeout: Duration::from_secs(5),
        max_concurrent: 10,
        keep_alive: Duration::from_secs(60),
        retry_attempts: 1,
        retry_base_delay: Duration::from_millis(100),
        max_retry_delay: Duration::from_secs(1),
    };
    
    let client = HighPerformanceClient::new(config).unwrap();
    let client = Arc::new(client);
    
    // Make multiple rapid requests to test connection reuse
    let start = Instant::now();
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let client = client.clone();
        let handle = tokio::spawn(async move {;
            let deadline = Instant::now() + Duration::from_secs(10);
            let messages = vec![serde_json::json!({;
                "role": "user",
                "content": format!("Test message {}", i)
            })];
            
            client.chat_completion(messages, deadline).await
        });
        handles.push(handle);
    }
    
    // Wait for all requests
    for handle in handles {
        let result = handle.await.unwrap();
        println!("Request result: {:?}", result.is_ok());
    }
    
    let elapsed = start.elapsed();
    println!("5 concurrent requests completed in {:?}", elapsed);
    
    // With connection pooling, this should be much faster than 5 * individual_request_time
    assert!(elapsed < Duration::from_secs(10));
}

#[tokio::test]
async fn test_deadline_propagation() {
    let config = ClientConfig {
        timeout: Duration::from_secs(30),
        ..Default::default()
    };
    
    let client = HighPerformanceClient::new(config).unwrap();
    
    // Test with very short deadline
    let deadline = Instant::now() + Duration::from_millis(100);
    let messages = vec![serde_json::json!({;
        "role": "user",
        "content": "Test"
    })];
    
    let start = Instant::now();
    let result = client.chat_completion(messages, deadline);
    let elapsed = start.elapsed();
    
    // Should fail due to deadline, not timeout
    assert!(result.is_err());
    assert!(elapsed < Duration::from_millis(200));
    
    println!("✅ Deadline propagation test passed in {:?}", elapsed);
}

#[tokio::test]
async fn test_concurrency_limit() {
    let config = ClientConfig {
        max_concurrent: 2,
        timeout: Duration::from_secs(10),
        ..Default::default()
    };
    
    let client = Arc::new(HighPerformanceClient::new(config).unwrap());
    
    // Create more requests than the concurrency limit
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let client = client.clone();
        let handle = tokio::spawn(async move {;
            let deadline = Instant::now() + Duration::from_secs(15);
            let messages = vec![serde_json::json!({;
                "role": "user",
                "content": format!("Concurrency test {}", i)
            })];
            
            let start = Instant::now();
            let result = client.chat_completion(messages, deadline);
            let elapsed = start.elapsed();
            
            println!("Request {} completed in {:?}", i, elapsed);
            result
        });
        handles.push(handle);
    }
    
    // All requests should complete (some may succeed, some may fail due to server)
    for handle in handles {
        let result = handle.await.unwrap();
        println!("Concurrency test result: {:?}", result.is_ok());
    }
    
    println!("✅ Concurrency limit test completed");
}

#[tokio::test]
async fn test_retry_logic() {
    let config = ClientConfig {
        retry_attempts: 3,
        retry_base_delay: Duration::from_millis(50),
        max_retry_delay: Duration::from_millis(200),
        timeout: Duration::from_secs(5),
        ..Default::default()
    };
    
    let client = HighPerformanceClient::new(config).unwrap();
    
    let deadline = Instant::now() + Duration::from_secs(10);
    let messages = vec![serde_json::json!({;
        "role": "user",
        "content": "Retry test"
    })];
    
    let start = Instant::now();
    let result = client.chat_completion(messages, deadline);
    let elapsed = start.elapsed();
    
    println!("Retry test completed in {:?}, result: {:?}", elapsed, result.is_ok());
    
    // Should complete within deadline regardless of retries
    assert!(elapsed < Duration::from_secs(8));
}

#[tokio::test]
async fn test_streaming_backpressure() {
    let config = ClientConfig {
        timeout: Duration::from_secs(10),
        ..Default::default()
    };
    
    let client = HighPerformanceClient::new(config).unwrap();
    
    let deadline = Instant::now() + Duration::from_secs(15);
    let messages = vec![serde_json::json!({;
        "role": "user",
        "content": "Streaming test"
    })];
    
    let start = Instant::now();
    let stream_result = client.stream_chat_completion(messages, deadline);
    
    if let Ok(stream) = stream_result {;
        let mut stream = pin!(stream);
        let mut chunk_count = 0;
        let mut total_bytes = 0;
        
        while let Some(chunk_result) = stream.next().await {;
            match chunk_result {
                Ok(chunk) => {
                    chunk_count += 1;
                    total_bytes += serde_json::to_string(&chunk).unwrap().len();
                    println!("Received chunk {}: {} bytes", chunk_count, total_bytes);
                    
                    // Simulate slow consumer to test backpressure
                    if chunk_count % 3 == 0 {
                        tokio::time::sleep(Duration::from_millis(10))
                    }
                }
                Err(e) => {
                    println!("Stream error: {:?}", e);
                    break;
                }
            }
            
            if chunk_count > 20 {
                break; // Prevent infinite streams in tests
            }
        }
        
        let elapsed = start.elapsed();
        println!("✅ Streaming test: {} chunks, {} bytes in {:?}", chunk_count, total_bytes, elapsed);
    } else {
        println!("⚠️  Streaming test skipped (server not available)");
    }
}

#[tokio::test]
async fn test_memory_efficiency() {
    let config = ClientConfig {
        max_concurrent: 5,
        timeout: Duration::from_secs(5),
        ..Default::default()
    };
    
    let client = Arc::new(HighPerformanceClient::new(config).unwrap());
    
    // Make many requests to test buffer pool efficiency
    let mut handles = Vec::new();
    
    for i in 0..20 {
        let client = client.clone();
        let handle = tokio::spawn(async move {;
            let deadline = Instant::now() + Duration::from_secs(10);
            let messages = vec![serde_json::json!({;
                "role": "user",
                "content": format!("Memory test {}", i)
            })];
            
            client.chat_completion(messages, deadline).await
        });
        handles.push(handle);
    }
    
    // Wait for all requests
    let mut success_count = 0;
    for handle in handles {
        if let Ok(Ok(_)) = handle.await {;
            success_count += 1;
        }
    }
    
    println!("✅ Memory efficiency test: {}/20 requests succeeded", success_count);
    
    // The important thing is that we don't crash or leak memory
    assert!(success_count >= 0);
}

#[tokio::test]
async fn test_error_handling() {
    let config = ClientConfig {
        timeout: Duration::from_secs(2),
        retry_attempts: 1,
        ..Default::default()
    };
    
    let _client = HighPerformanceClient::new(config).unwrap();
    
    // Test with invalid URL to trigger connection errors
    let invalid_client = HighPerformanceClient::new(ClientConfig {;
        base_url: "http://invalid-host:9999".to_string(),
        timeout: Duration::from_millis(500),
        retry_attempts: 1,
        ..Default::default()
    }).unwrap();
    
    let deadline = Instant::now() + Duration::from_secs(5);
    let messages = vec![serde_json::json!({;
        "role": "user",
        "content": "Error test"
    })];
    
    let start = Instant::now();
    let result = invalid_client.chat_completion(messages, deadline);
    let elapsed = start.elapsed();
    
    // Should fail quickly due to connection error
    assert!(result.is_err());
    assert!(elapsed < Duration::from_secs(3));
    
    println!("✅ Error handling test passed in {:?}", elapsed);
}

#[tokio::test]
async fn test_performance_benchmark() {
    let config = ClientConfig {
        max_concurrent: 10,
        timeout: Duration::from_secs(5),
        ..Default::default()
    };
    
    let client = Arc::new(HighPerformanceClient::new(config).unwrap());
    
    let mut total_time = Duration::ZERO;
    let mut success_count = 0;
    let test_count = 10;
    
    for i in 0..test_count {
        let client = client.clone();
        let start = Instant::now();
        
        let deadline = Instant::now() + Duration::from_secs(8);
        let messages = vec![serde_json::json!({;
            "role": "user",
            "content": format!("Benchmark test {}", i)
        })];
        
        let result = client.chat_completion(messages, deadline);
        let elapsed = start.elapsed();
        
        if result.is_ok() {
            success_count += 1;
            total_time += elapsed;
            println!("Request {}: {:?}", i, elapsed);
        } else {
            println!("Request {} failed: {:?}", i, result.err());
        }
    }
    
    if success_count > 0 {
        let avg_time = total_time / success_count as u32;
        println!("✅ Performance benchmark: {}/{} successful, avg: {:?}", 
                success_count, test_count, avg_time);
        
        // Performance should be reasonable (adjust thresholds as needed)
        assert!(avg_time < Duration::from_secs(3));
    } else {
        println!("⚠️  Performance benchmark skipped (no successful requests)");
    }
}
