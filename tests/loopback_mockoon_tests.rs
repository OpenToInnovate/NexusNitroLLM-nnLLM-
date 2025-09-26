//! # Localhost:8080 Loopback Tests with Mockoon
//!
//! Tests the proxy server running on localhost:8080 against a mock OpenAI-compatible API
//! using Mockoon CLI. This tests the actual proxy server as it would be used in production.

use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

/// Configuration for loopback tests
const PROXY_URL: &str = "http://127.0.0.1:8080";
const MOCKOON_URL: &str = "http://127.0.0.1:3000";

/// Wait for Mockoon server to be ready
async fn wait_for_mockoon_server() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut attempts = 0;
    let max_attempts = 30;

    while attempts < max_attempts {
        match client.get(MOCKOON_URL).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    println!("✅ Mockoon server is ready for loopback tests");
                    return Ok(());
                }
            }
            Err(_) => {
                // Server not ready yet
            }
        }

        sleep(Duration::from_millis(500));
        attempts += 1;
    }

    Err("Mockoon server failed to become ready within timeout".into())
}

/// Wait for proxy server to be ready
async fn wait_for_proxy_server() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut attempts = 0;
    let max_attempts = 60; // Give proxy server more time to start

    while attempts < max_attempts {
        match client.get(&format!("{}/health", PROXY_URL)).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    println!("✅ Proxy server is ready for loopback tests");
                    return Ok(());
                }
            }
            Err(_) => {
                // Server not ready yet
            }
        }

        sleep(Duration::from_millis(1000))
        attempts += 1;
    }

    Err("Proxy server failed to become ready within timeout".into())
}

/// Test proxy server health check
#[tokio::test]
async fn test_loopback_health_check() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Mockoon server not running");
        return;
    }

    if wait_for_proxy_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Proxy server not running on {}", PROXY_URL);
        return;
    }

    let client = Client::new();
    let response = client.get(&format!("{}/health", PROXY_URL)).send().await.unwrap();
    
    assert_eq!(response.status(), 200);
    
    let data: serde_json::Value = response.json().await.unwrap();
    assert_eq!(data["status"], "ok");
}

/// Test proxy server chat completions
#[tokio::test]
async fn test_loopback_chat_completions() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Mockoon server not running");
        return;
    }

    if wait_for_proxy_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Proxy server not running on {}", PROXY_URL);
        return;
    }

    let client = Client::new();
    let request_body = json!({;
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": "Hello from loopback test!"
            }
        ],
        "max_tokens": 50
    });

    let response = client;
        .post(&format!("{}/v1/chat/completions", PROXY_URL))
        .json(&request_body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    
    let data: serde_json::Value = response.json().await.unwrap();
    assert!(data["id"].is_string());
    assert!(data["choices"].is_array());
    assert!(!data["choices"].as_array().unwrap().is_empty());
    assert!(data["choices"][0]["message"]["content"].is_string());
}

/// Test proxy server streaming chat completions
#[tokio::test]
async fn test_loopback_streaming_chat_completions() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Mockoon server not running");
        return;
    }

    if wait_for_proxy_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Proxy server not running on {}", PROXY_URL);
        return;
    }

    let client = Client::new();
    let request_body = json!({;
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": "Stream this message from loopback!"
            }
        ],
        "stream": true,
        "max_tokens": 50
    });

    let response = client;
        .post(&format!("{}/v1/chat/completions", PROXY_URL))
        .json(&request_body)
        .header("accept", "text/event-stream")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/event-stream"));
}

/// Test proxy server models endpoint
#[tokio::test]
async fn test_loopback_models_endpoint() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Mockoon server not running");
        return;
    }

    if wait_for_proxy_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Proxy server not running on {}", PROXY_URL);
        return;
    }

    let client = Client::new();
    let response = client.get(&format!("{}/v1/models", PROXY_URL)).send().await.unwrap();
    
    assert_eq!(response.status(), 200);
    
    let data: serde_json::Value = response.json().await.unwrap();
    assert_eq!(data["object"], "list");
    assert!(data["data"].is_array());
    assert!(!data["data"].as_array().unwrap().is_empty());
    assert_eq!(data["data"][0]["id"], "gpt-3.5-turbo");
}

/// Test proxy server error handling
#[tokio::test]
async fn test_loopback_error_handling() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Mockoon server not running");
        return;
    }

    if wait_for_proxy_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Proxy server not running on {}", PROXY_URL);
        return;
    }

    let client = Client::new();

    // Test malformed JSON
    let response = client;
        .post(&format!("{}/v1/chat/completions", PROXY_URL))
        .header("content-type", "application/json")
        .body("invalid json")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 400);

    // Test empty messages array (should trigger 400 from Mockoon)
    let request_body = json!({;
        "model": "gpt-3.5-turbo",
        "messages": [],
        "max_tokens": 50
    });

    let response = client;
        .post(&format!("{}/v1/chat/completions", PROXY_URL))
        .json(&request_body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 400);
}

/// Test proxy server with different models
#[tokio::test]
async fn test_loopback_different_models() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Mockoon server not running");
        return;
    }

    if wait_for_proxy_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Proxy server not running on {}", PROXY_URL);
        return;
    }

    let client = Client::new();
    let models = vec!["gpt-3.5-turbo", "gpt-4", "gpt-4-turbo-preview"];

    for model in models {
        let request_body = json!({;
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": format!("Test message for {} from loopback", model)
                }
            ],
            "max_tokens": 50
        });

        let response = client;
            .post(&format!("{}/v1/chat/completions", PROXY_URL))
            .json(&request_body)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        
        let data: serde_json::Value = response.json().await.unwrap();
        assert!(data["id"].is_string());
        assert!(data["choices"].is_array());
    }
}

/// Test proxy server concurrent requests
#[tokio::test]
async fn test_loopback_concurrent_requests() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Mockoon server not running");
        return;
    }

    if wait_for_proxy_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Proxy server not running on {}", PROXY_URL);
        return;
    }

    let client = Client::new();
    let request_body = json!({;
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": "Concurrent test from loopback"
            }
        ],
        "max_tokens": 50
    });

    // Send 5 concurrent requests
    let mut handles = vec![];
    for i in 0..5 {
        let client_clone = client.clone();
        let request_body_clone = request_body.clone();
        
        let handle = tokio::spawn(async move {;
            let response = client_clone;
                .post(&format!("{}/v1/chat/completions", PROXY_URL))
                .json(&request_body_clone)
                .header("x-request-id", format!("loopback-test-{}", i))
                .send()
                .await
                .unwrap();

            response.status()
        });
        
        handles.push(handle);
    }

    // Wait for all requests to complete
    let mut success_count = 0;
    for handle in handles {
        let status = handle.await.unwrap();
        if status.is_success() {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 5);
}

/// Test proxy server large request handling
#[tokio::test]
async fn test_loopback_large_request() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Mockoon server not running");
        return;
    }

    if wait_for_proxy_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Proxy server not running on {}", PROXY_URL);
        return;
    }

    let client = Client::new();
    
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

    let response = client;
        .post(&format!("{}/v1/chat/completions", PROXY_URL))
        .json(&request_body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    
    let data: serde_json::Value = response.json().await.unwrap();
    assert!(data["id"].is_string());
    assert!(data["choices"].is_array());
}

/// Test proxy server authentication headers
#[tokio::test]
async fn test_loopback_authentication_headers() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Mockoon server not running");
        return;
    }

    if wait_for_proxy_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Proxy server not running on {}", PROXY_URL);
        return;
    }

    let client = Client::new();
    let request_body = json!({;
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": "Test with authentication headers"
            }
        ],
        "max_tokens": 50
    });

    let response = client;
        .post(&format!("{}/v1/chat/completions", PROXY_URL))
        .json(&request_body)
        .header("authorization", "Bearer test-token")
        .header("user-agent", "NexusNitroLLM-Loopback-Test/1.0")
        .header("x-client-version", "1.0")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    
    let data: serde_json::Value = response.json().await.unwrap();
    assert!(data["id"].is_string());
    assert!(data["choices"].is_array());
}

/// Test proxy server timeout handling
#[tokio::test]
async fn test_loopback_timeout_handling() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Mockoon server not running");
        return;
    }

    if wait_for_proxy_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Proxy server not running on {}", PROXY_URL);
        return;
    }

    let client = Client::builder();
        .timeout(Duration::from_secs(5)) // Short timeout
        .build()
        .unwrap();

    // Send request that will timeout (timeout_test=true triggers 35s delay in Mockoon)
    let request_body = json!({;
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": "This should timeout"
            }
        ],
        "timeout_test": true,
        "max_tokens": 50
    });

    let response = client;
        .post(&format!("{}/v1/chat/completions", PROXY_URL))
        .json(&request_body)
        .send()
        

    // Should get a timeout error
    match response {
        Ok(resp) => {
            // If we get a response, it should be an error status
            assert!(resp.status().is_client_error() || resp.status().is_server_error());
        }
        Err(e) => {
            // Timeout error is expected
            assert!(e.is_timeout() || e.to_string().contains("timeout"));
        }
    }
}

/// Comprehensive loopback integration test
#[tokio::test]
async fn test_loopback_integration_comprehensive() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Mockoon server not running");
        return;
    }

    if wait_for_proxy_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Proxy server not running on {}", PROXY_URL);
        return;
    }

    let client = Client::new();
    println!("🧪 Running comprehensive loopback integration tests...");

    // Test 1: Health check
    let response = client.get(&format!("{}/health", PROXY_URL)).send().await.unwrap();
    assert_eq!(response.status(), 200);
    println!("✅ Health check passed");

    // Test 2: Models list
    let response = client.get(&format!("{}/v1/models", PROXY_URL)).send().await.unwrap();
    assert_eq!(response.status(), 200);
    println!("✅ Models endpoint passed");

    // Test 3: Chat completion
    let request_body = json!({;
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": "Hello from comprehensive loopback test!"
            }
        ],
        "max_tokens": 50
    });

    let response = client;
        .post(&format!("{}/v1/chat/completions", PROXY_URL))
        .json(&request_body)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    println!("✅ Chat completion passed");

    // Test 4: Streaming chat completion
    let streaming_request_body = json!({;
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": "Stream this from comprehensive loopback test!"
            }
        ],
        "stream": true,
        "max_tokens": 50
    });

    let response = client;
        .post(&format!("{}/v1/chat/completions", PROXY_URL))
        .json(&streaming_request_body)
        .header("accept", "text/event-stream")
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    println!("✅ Streaming chat completion passed");

    println!("🎉 All loopback integration tests passed!");
}

/// Test loopback performance characteristics
#[tokio::test]
async fn test_loopback_performance() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Mockoon server not running");
        return;
    }

    if wait_for_proxy_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Proxy server not running on {}", PROXY_URL);
        return;
    }

    let client = Client::new();
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

    // Measure response time for multiple requests
    let start_time = std::time::Instant::now();
    let request_count = 10;

    for i in 0..request_count {
        let response = client;
            .post(&format!("{}/v1/chat/completions", PROXY_URL))
            .json(&request_body)
            .header("x-request-id", format!("perf-test-{}", i))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
    }

    let elapsed = start_time.elapsed();
    let avg_time = elapsed / request_count;
    
    println!("🚀 Loopback Performance Results:");
    println!("   Total requests: {}", request_count);
    println!("   Total time: {:?}", elapsed);
    println!("   Average time per request: {:?}", avg_time);
    
    // Assert reasonable performance (should be under 2 seconds per request through proxy)
    assert!(avg_time.as_millis() < 2000, "Average response time too slow: {:?}", avg_time);
}

/// Test loopback with different backend configurations
#[tokio::test]
async fn test_loopback_backend_configurations() {
    if wait_for_mockoon_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Mockoon server not running");
        return;
    }

    if wait_for_proxy_server().await.is_err() {
        println!("⚠️  Skipping loopback test - Proxy server not running on {}", PROXY_URL);
        return;
    }

    let client = Client::new();
    
    // Test different model configurations
    let models = ["gpt-3.5-turbo", "gpt-4", "gpt-4-turbo-preview"];
    
    for model in models {
        let request_body = json!({;
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": format!("Testing {} model configuration", model)
                }
            ],
            "max_tokens": 20
        });

        let response = client;
            .post(&format!("{}/v1/chat/completions", PROXY_URL))
            .json(&request_body)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        
        let data: serde_json::Value = response.json().await.unwrap();
        assert!(data["id"].is_string());
        assert!(data["choices"].is_array());
        
        println!("✅ Model {} configuration test passed", model);
    }
}
