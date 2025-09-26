//! # Direct Adapter Module
//!
//! This module provides the Direct adapter implementation for
//! embedded LLM integration that bypasses HTTP entirely.

use crate::{
    adapters::base::{AdapterTrait, AdapterUtils},
    error::ProxyError,
    schemas::{ChatCompletionRequest, ChatCompletionResponse, Message, Choice, Usage},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Configuration for the direct inference engine
#[derive(Clone, Debug)]
pub struct DirectInferenceConfig {
    pub model_id: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub context_window: usize,
}

impl Default for DirectInferenceConfig {
    fn default() -> Self {
        Self {
            model_id: "llama-2-7b-chat".to_string(),
            max_tokens: 1000,
            temperature: 0.7,
            top_p: 0.9,
            context_window: 4096,
        }
    }
}

/// Mock inference engine for demonstration
/// In production, this would be replaced with actual LLM libraries like:
/// - candle-core
/// - tch (PyTorch bindings)
/// - ort (ONNX Runtime)
/// - llama-cpp-rs
#[derive(Debug)]
pub struct MockInferenceEngine {
    config: DirectInferenceConfig,
    model_loaded: bool,
    conversation_history: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl MockInferenceEngine {
    pub fn new(config: DirectInferenceConfig) -> Self {
        Self {
            config,
            model_loaded: false,
            conversation_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn load_model(&mut self) -> Result<(), ProxyError> {
        tracing::info!("Loading model: {}", self.config.model_id);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        self.model_loaded = true;
        tracing::info!("Model loaded successfully");
        Ok(())
    }

    pub async fn generate(&self, prompt: &str, session_id: &str) -> Result<String, ProxyError> {
        if !self.model_loaded {
            return Err(ProxyError::Internal("Model not loaded - call initialize() first".to_string()));
        }

        let mut history = self.conversation_history.write().await;
        let session_history = history.entry(session_id.to_string()).or_insert_with(Vec::new);
        session_history.push(prompt.to_string());

        let response = self.mock_generate(prompt, session_history).await?;
        session_history.push(response.clone());

        if session_history.len() > 20 {
            session_history.drain(0..10);
        }

        Ok(response)
    }

    async fn mock_generate(&self, prompt: &str, history: &[String]) -> Result<String, ProxyError> {
        let inference_time = (prompt.len() / 10).max(50).min(500);
        tokio::time::sleep(tokio::time::Duration::from_millis(inference_time as u64)).await;

        let response = if prompt.to_lowercase().contains("hello") {
            "Hello! I'm a direct-mode LLM assistant. How can I help you today?"
        } else if prompt.to_lowercase().contains("code") || prompt.to_lowercase().contains("program") {
            "I'd be happy to help you with coding! What programming language or concept would you like to explore?"
        } else if prompt.to_lowercase().contains("explain") {
            "I'll provide a clear explanation. Let me break this down for you in an understandable way."
        } else if history.len() > 1 {
            "Based on our conversation, I understand you're looking for more information. Let me continue helping you."
        } else {
            "I understand your request. Let me provide you with a helpful response based on what you've asked."
        };

        Ok(response.to_string())
    }

    pub fn get_stats(&self) -> Value {
        json!({
            "model_id": self.config.model_id,
            "model_loaded": self.model_loaded,
            "inference_mode": "direct",
            "context_window": self.config.context_window,
            "performance": {
                "bypass_http": true,
                "zero_copy": true,
                "hardware_optimized": true
            }
        })
    }
}

/// # Direct Adapter
///
/// Direct integration adapter that bypasses HTTP for maximum performance
/// in embedded applications or when the LLM is running in the same process.
#[derive(Clone, Debug)]
pub struct DirectAdapter {
    /// Model ID for direct LLM integration
    model_id: String,
    /// Optional authentication token
    token: Option<String>,
    /// Direct inference engine
    engine: Arc<RwLock<MockInferenceEngine>>,
}

impl DirectAdapter {
    /// Create a new Direct adapter instance
    pub fn new(model_id: String, token: Option<String>) -> Self {
        let config = DirectInferenceConfig {
            model_id: model_id.clone(),
            ..Default::default()
        };

        let engine = MockInferenceEngine::new(config);

        Self {
            model_id,
            token,
            engine: Arc::new(RwLock::new(engine)),
        }
    }

    /// Initialize the direct inference engine
    pub async fn initialize(&self) -> Result<(), ProxyError> {
        let mut engine = self.engine.write().await;
        engine.load_model().await?;
        Ok(())
    }

    /// Get performance statistics
    pub async fn get_stats(&self) -> Value {
        let engine = self.engine.read().await;
        engine.get_stats()
    }

    /// Process chat completion requests directly
    pub async fn chat_completions(&self, req: ChatCompletionRequest) -> Result<ChatCompletionResponse, ProxyError> {
        AdapterUtils::log_request("direct", &AdapterUtils::extract_model(&req, &self.model_id), req.messages.len());

        let start_time = std::time::Instant::now();

        // Convert OpenAI messages to a single prompt
        let mut prompt = String::new();
        for message in &req.messages {
            match message.role.as_str() {
                "system" => {
                    if let Some(content) = &message.content {
                        prompt.push_str(&format!("System: {}\n", content));
                    }
                }
                "user" => {
                    if let Some(content) = &message.content {
                        prompt.push_str(&format!("User: {}\n", content));
                    }
                }
                "assistant" => {
                    if let Some(content) = &message.content {
                        prompt.push_str(&format!("Assistant: {}\n", content));
                    }
                }
                _ => {} // Skip unknown roles
            }
        }
        prompt.push_str("Assistant:");

        // Generate session ID based on request
        let session_id = format!("direct-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));

        // Auto-initialize engine if needed
        {
            let engine = self.engine.read().await;
            if !engine.model_loaded {
                drop(engine);
                let mut engine_mut = self.engine.write().await;
                if !engine_mut.model_loaded {
                    engine_mut.load_model().await?;
                }
            }
        }

        // Generate response using direct inference
        let engine = self.engine.read().await;
        let completion = engine.generate(&prompt, &session_id).await?;

        let response_time = start_time.elapsed().as_millis() as u64;
        AdapterUtils::log_response("direct", &AdapterUtils::extract_model(&req, &self.model_id), true, response_time);

        // Create OpenAI-compatible response
        let response = ChatCompletionResponse {
            id: format!("chatcmpl-direct-{}", chrono::Utc::now().timestamp()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: AdapterUtils::extract_model(&req, &self.model_id),
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: "assistant".to_string(),
                    content: Some(completion.trim().to_string()),
                    name: None,
                    function_call: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: "stop".to_string(),
                logprobs: None,
            }],
            usage: Some(Usage {
                prompt_tokens: prompt.split_whitespace().count() as u32,
                completion_tokens: completion.split_whitespace().count() as u32,
                total_tokens: (prompt.split_whitespace().count() + completion.split_whitespace().count()) as u32,
            }),
        };

        Ok(response)
    }
}

#[async_trait::async_trait]
impl AdapterTrait for DirectAdapter {
    fn name(&self) -> &'static str {
        "direct"
    }

    fn base_url(&self) -> &str {
        "direct://"
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn has_auth(&self) -> bool {
        self.token.is_some()
    }

    async fn chat_completions(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse, ProxyError> {
        self.chat_completions(request).await
    }
}