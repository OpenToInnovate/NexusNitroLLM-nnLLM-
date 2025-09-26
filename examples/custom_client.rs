//! Custom client example showing how to use adapters directly
//!
//! This example demonstrates how to use the library components
//! directly without starting a full server.

use nexus_nitro_llm::{
    Config, Adapter, ChatCompletionRequest, Message, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create configuration
    let mut config = Config::for_test();
    config.backend_url = "http://localhost:8000".to_string();
    config.backend_type = "lightllm".to_string();
    config.model_id = "llama".to_string();

    // Create adapter
    let adapter = Adapter::from_config(&config);

    // Create a chat completion request
    let request = ChatCompletionRequest {
        model: Some("llama".to_string()),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: Some("Hello! What's the weather like today?".to_string()),
                name: None,
                tool_calls: None,
                function_call: None,
                tool_call_id: None,
            }
        ],
        max_tokens: Some(100),
        temperature: Some(0.7),
        top_p: None,
        n: None,
        stream: Some(false),
        stop: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        logprobs: None,
        seed: None,
        top_logprobs: None,
        tools: None,
        tool_choice: None,
    };

    println!("Sending request to backend...");
    println!("Request: {}", serde_json::to_string_pretty(&request)?);

    // Send request using adapter
    match adapter.chat_completions(request).await {
        Ok(response) => {
            println!("Response received:");
            println!("Status: {}", response.status());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}