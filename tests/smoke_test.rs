//! # Minimal Smoke Test
//! 
//! Lean, fast smoke test that verifies core functionality without bloat.

use std::time::{Duration, Instant};
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".into());
    
    println!("🚀 Running minimal smoke test against {}", base_url);
    
    let client = Client::builder();
        .timeout(Duration::from_secs(5))
        .build()?;
    
    // Test 1: Health check
    println!("🧪 Testing health endpoint...");
    let start = Instant::now();
    let response = client.get(&format!("{}/health", base_url)).send().await?;
    let elapsed = start.elapsed();
    
    if response.status().is_success() {
        println!("✅ Health check passed in {}ms", elapsed.as_millis());
    } else {
        println!("❌ Health check failed: {}", response.status());
        return Ok(());
    }
    
    // Test 2: Chat completion
    println!("🧪 Testing chat completion...");
    let start = Instant::now();
    let body = json!({;
        "model": "test-model",
        "messages": [{"role": "user", "content": "Hello"}],
        "max_tokens": 10
    });
    
    let response = client;
        .post(&format!("{}/v1/chat/completions", base_url))
        .json(&body)
        .send()
        .await?;
    
    let elapsed = start.elapsed();
    
    if response.status().is_success() {
        let data: serde_json::Value = response.json().await?;
        println!("✅ Chat completion passed in {}ms", elapsed.as_millis());
        println!("   Response ID: {}", data.get("id").unwrap_or(&json!("unknown")));
    } else {
        println!("❌ Chat completion failed: {}", response.status());
    }
    
    // Test 3: Cancellation (quick timeout)
    println!("🧪 Testing cancellation...");
    let start = Instant::now();
    let short_client = Client::builder();
        .timeout(Duration::from_millis(100))
        .build()?;
    
    match short_client
        .post(&format!("{}/v1/chat/completions", base_url))
        .json(&body)
        .send()
        .await
    {
        Ok(_) => println!("⚠️  Expected timeout, but got response"),
        Err(e) if e.is_timeout() => {
            println!("✅ Timeout test passed in {}ms", start.elapsed().as_millis());
        }
        Err(e) => println!("❌ Unexpected error: {}", e),
    }
    
    println!("🎉 Smoke test completed!");
    Ok(())
}

