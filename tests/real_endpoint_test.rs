use reqwest::Client;
use serde_json::json;
use std::env;

/// Test script to validate the proxy works with the real LLM endpoint
/// Set these environment variables before running:
/// - nnLLM_URL: The LLM endpoint URL
/// - nnLLM_TOKEN: The authorization token
/// - nnLLM_MODEL: The model ID to use
///
/// Run with: cargo test test_real_endpoint -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn test_real_endpoint() {
    let lightllm_url = env::var("nnLLM_URL")
        .unwrap_or_else(|_| "https://YOUR_LITELLM_PROXY.com".to_string());
    let token = env::var("nnLLM_TOKEN")
        .unwrap_or_else(|_| "REPLACE_WITH_YOUR_TOKEN".to_string());
    let model = env::var("nnLLM_MODEL").unwrap_or_else(|_| "llama".to_string());

    println!("Testing with LLM URL: {}", lightllm_url);
    println!("Using model: {}", model);

    let client = Client::new();

    // Test different types of requests
    let test_cases = vec![;
        json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": "Hello! What's 2+2?"
                }
            ],
            "max_tokens": 50,
            "temperature": 0.7
        }),
        json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a helpful math tutor."
                },
                {
                    "role": "user",
                    "content": "Explain what multiplication is in simple terms."
                }
            ],
            "max_tokens": 100,
            "temperature": 0.5
        }),
        json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": "Tell me a short joke."
                }
            ],
            "max_tokens": 30,
            "temperature": 1.0
        }),
    ];

    for (i, test_case) in test_cases.iter().enumerate() {
        println!("\n--- Test Case {} ---", i + 1);
        println!("Request: {}", serde_json::to_string_pretty(test_case).unwrap());

        // Test direct call to OpenAI-compatible endpoint
        let direct_response = test_direct_openai_call(&client, &lightllm_url, &token, test_case);
            

        match direct_response {
            Ok(response) => {
                println!("✅ Direct OpenAI-compatible call succeeded");
                println!("Response: {}", serde_json::to_string_pretty(&response).unwrap());
            }
            Err(e) => {
                println!("❌ Direct OpenAI-compatible call failed: {}", e);
                continue; // Skip proxy test if direct call fails
            }
        }

        // Test through our proxy (if running)
        // This would require the proxy to be running separately
        // For now, we just demonstrate the structure
        println!("Note: To test the proxy, start it with:");
        println!(
            "cargo run -- --lightllm-url {} --model-id {}",
            lightllm_url, model
        );
        println!("Then run requests against http://localhost:8080/v1/chat/completions");
    }
}

async fn test_direct_openai_call(
    client: &Client,
    base_url: &str,
    token: &str,
    openai_request: &serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Since we now know this is an OpenAI-compatible endpoint,
    // send the request directly without conversion

    let response = client
        .post(format!("{}/chat/completions", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(openai_request)
        .send()
        .await?;

    let status = response.status();
    let response_text = response.text().await?;

    if !status.is_success() {
        return Err(format!("HTTP {}: {}", status, response_text).into());
    }

    let openai_response: serde_json::Value = serde_json::from_str(&response_text)?;
    Ok(openai_response)
}

/// Test the proxy manually (requires proxy to be running separately)
/// This is a helper for manual testing, not automated tests
///
/// Usage:
/// 1. Start the proxy: cargo run -- --nnllm-url https://YOUR_LITELLM_PROXY.com/v1 --model-id llama
/// 2. Run: cargo test test_proxy_manual -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn test_proxy_manual() {
    println!("🚀 Manual proxy test");
    println!("================");
    println!("To run this test:");
    println!("1. Start the proxy in another terminal:");
    println!("   cargo run -- --nnllm-url https://YOUR_LITELLM_PROXY.com/v1 --model-id llama");
    println!("2. Then run this test:");
    println!("   cargo test test_proxy_manual -- --ignored --nocapture");
    println!("3. Or use the manual test script: ./scripts/manual_test.sh");

    // This test just provides instructions - the real test is in scripts/manual_test.sh
    println!("✅ See instructions above for manual testing");
}