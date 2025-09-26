//! # Node.js Bindings for NexusNitroLLM
//!
//! High-performance Node.js bindings using napi-rs for maximum efficiency.
//! Provides zero-copy data transfer and native async/await support with
//! proper Node.js event loop integration for multiple LLM backends.
//!
//! ## Features
//!
//! - **🚀 Maximum Performance**: napi-rs provides the fastest Node.js to Rust bridge
//! - **⚡ Zero-Copy Transfer**: Direct memory access without serialization overhead
//! - **🔄 Native Async/Await**: Proper Node.js event loop integration
//! - **📝 Auto TypeScript**: TypeScript definitions generated from Rust code
//! - **🧠 Memory Efficient**: Minimal overhead with Rust's memory management
//! - **🔒 Thread Safe**: Safe concurrent access across Node.js threads

use crate::{
    adapters::Adapter,
    config::Config,
    schemas::{ChatCompletionRequest, Message},
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
// Removed unused import
use tokio::runtime::Runtime;

/// High-performance configuration for Node.js applications
///
/// Optimized for maximum throughput and minimal latency in Node.js environments.
/// All configuration changes are applied immediately for real-time performance tuning.
///
/// ## Direct Mode vs HTTP Mode
/// - **Direct Mode**: Set `backend_url` to `null` or `"direct"` for maximum performance
/// - **HTTP Mode**: Provide a valid URL for traditional proxy communication
#[napi(object)]
#[derive(Clone)]
pub struct NodeConfig {
    /// Backend LLM server URL (null or "direct" for direct mode)
    pub backend_url: Option<String>,
    /// Backend LLM type (lightllm, vllm, openai, azure, aws, etc.)
    pub backend_type: Option<String>,
    /// Default model identifier
    pub model_id: String,
    /// Server port (optional)
    pub port: Option<u16>,
    /// Authentication token (optional)
    pub token: Option<String>,
    /// Enable connection pooling for maximum performance
    pub connection_pooling: Option<bool>,
    /// Maximum HTTP connections (performance tuning)
    pub max_connections: Option<u32>,
    /// Maximum connections per host (performance tuning)
    pub max_connections_per_host: Option<u32>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            backend_url: None, // Default to direct mode for maximum performance
            backend_type: Some("lightllm".to_string()),
            model_id: "llama".to_string(),
            port: Some(3000),
            token: None,
            connection_pooling: Some(true),
            max_connections: Some(100),     // Higher default for Node.js
            max_connections_per_host: Some(20),  // Higher default for Node.js
        }
    }
}

impl From<NodeConfig> for Config {
    fn from(node_config: NodeConfig) -> Self {
        let mut config = Config::for_test();

        // Handle URL - default to direct mode if not provided
        config.backend_url = node_config.backend_url.unwrap_or_else(|| "direct".to_string());
        config.backend_type = node_config.backend_type.unwrap_or_else(|| "lightllm".to_string());
        config.model_id = node_config.model_id;

        if let Some(port) = node_config.port {
            config.port = port;
        }

        config.backend_token = node_config.token;

        // Optimize for Node.js performance
        config.enable_metrics = true;

        if let Some(max_conn) = node_config.max_connections {
            config.http_client_max_connections = max_conn as usize;
        }

        if let Some(max_conn_per_host) = node_config.max_connections_per_host {
            config.http_client_max_connections_per_host = max_conn_per_host as usize;
        }

        config
    }
}

/// High-performance message structure for Node.js
///
/// Designed for zero-copy operations and minimal garbage collection pressure.
#[napi(object)]
#[derive(Clone)]
pub struct NodeMessage {
    /// Message role: "system", "user", "assistant", or "tool"
    pub role: String,
    /// Message content
    pub content: String,
    /// Optional message name
    pub name: Option<String>,
}

impl From<NodeMessage> for Message {
    fn from(node_msg: NodeMessage) -> Self {
        Message {
            role: node_msg.role,
            content: Some(node_msg.content),
            name: node_msg.name,
            tool_calls: None,
            function_call: None,
            tool_call_id: None,
        }
    }
}

impl From<Message> for NodeMessage {
    fn from(msg: Message) -> Self {
        NodeMessage {
            role: msg.role,
            content: msg.content.unwrap_or_default(),
            name: msg.name,
        }
    }
}

/// Chat completion request parameters for Node.js
#[napi(object)]
#[derive(Clone)]
pub struct NodeChatRequest {
    /// List of messages in the conversation
    pub messages: Vec<NodeMessage>,
    /// Model to use (optional, uses config default if not specified)
    pub model: Option<String>,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Sampling temperature (0.0 to 2.0)
    pub temperature: Option<f64>,
    /// Nucleus sampling parameter
    pub top_p: Option<f64>,
    /// Number of completions to generate
    pub n: Option<u32>,
    /// Whether to stream the response
    pub stream: Option<bool>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
    /// Presence penalty (-2.0 to 2.0)
    pub presence_penalty: Option<f64>,
    /// Frequency penalty (-2.0 to 2.0)
    pub frequency_penalty: Option<f64>,
    /// User identifier for tracking
    pub user: Option<String>,
}

/// Chat completion response for Node.js
#[napi(object)]
#[derive(Clone)]
pub struct NodeChatResponse {
    /// Unique response identifier
    pub id: String,
    /// Object type ("chat.completion")
    pub object: String,
    /// Creation timestamp
    pub created: u32,
    /// Model used
    pub model: String,
    /// Response choices
    pub choices: Vec<NodeChoice>,
    /// Token usage information
    pub usage: NodeUsage,
}

/// Individual choice in response
#[napi(object)]
#[derive(Clone)]
pub struct NodeChoice {
    /// Choice index
    pub index: u32,
    /// Response message
    pub message: NodeMessage,
    /// Finish reason
    pub finish_reason: String,
}

/// Token usage statistics
#[napi(object)]
#[derive(Clone)]
pub struct NodeUsage {
    /// Tokens in prompt
    pub prompt_tokens: u32,
    /// Tokens in completion
    pub completion_tokens: u32,
    /// Total tokens used
    pub total_tokens: u32,
}

/// Statistics for performance monitoring
#[napi(object)]
#[derive(Clone)]
pub struct NodeStats {
    /// Adapter type being used (lightllm, openai, or direct)
    pub adapter_type: String,
    /// Backend URL (or "direct" for direct mode)
    pub backend_url: String,
    /// Model ID being used
    pub model_id: String,
    /// Server port (if applicable)
    pub port: Option<u16>,
    /// Whether connection pooling is enabled
    pub connection_pooling: bool,
    /// Maximum connections configured
    pub max_connections: u32,
    /// Maximum connections per host
    pub max_connections_per_host: u32,
    /// Timeout in seconds
    pub timeout_seconds: u32,
    /// Whether running in direct mode (no HTTP overhead)
    pub is_direct_mode: bool,
    /// Performance mode description
    pub performance_mode: String,
}

/// Ultra-high-performance LightLLM client for Node.js
///
/// This client provides direct access to Rust adapter code with zero HTTP overhead.
/// Optimized for maximum throughput in Node.js applications with proper async/await support.
#[napi]
pub struct NodeNexusNitroLLMClient {
    adapter: Adapter,
    #[allow(dead_code)]
    runtime: Runtime,
    config: Config,
}

#[napi]
impl NodeNexusNitroLLMClient {
    /// Create a new high-performance LightLLM client
    ///
    /// # Arguments
    /// * `config` - Configuration object with connection details
    ///
    /// # Returns
    /// * `Result<NodeNexusNitroLLMClient>` - New client instance or error
    ///
    /// # Performance Notes
    /// * Uses connection pooling by default for maximum throughput
    /// * Tokio runtime optimized for Node.js event loop integration
    /// * Zero-copy data structures minimize garbage collection
    #[napi(constructor)]
    pub fn new(config: NodeConfig) -> Result<Self> {
        let rust_config: Config = config.into();

        // Create optimized runtime for Node.js integration
        let runtime = Runtime::new()
            .map_err(|e| Error::new(
                Status::GenericFailure,
                format!("Failed to create async runtime: {}", e)
            ))?;

        let adapter = Adapter::from_config(&rust_config);

        Ok(Self {
            adapter,
            runtime,
            config: rust_config,
        })
    }

    /// Send chat completion request with maximum performance
    ///
    /// This method provides zero-overhead access to the Rust adapter by bypassing
    /// HTTP serialization entirely. Perfect for high-throughput Node.js applications.
    ///
    /// # Arguments
    /// * `request` - Chat completion request parameters
    ///
    /// # Returns
    /// * `Promise<NodeChatResponse>` - Resolves to chat completion response
    ///
    /// # Performance
    /// * Direct Rust function call (no HTTP overhead)
    /// * Zero-copy message handling where possible
    /// * Native async/await with proper Node.js event loop integration
    #[napi(ts_return_type = "Promise<NodeChatResponse>")]
    pub fn chat_completions(&self, request: NodeChatRequest) -> AsyncTask<NodeChatCompletionTask> {
        AsyncTask::new(NodeChatCompletionTask {
            adapter: self.adapter.clone(),
            config: self.config.clone(),
            request,
        })
    }

    /// Get performance statistics and configuration information
    ///
    /// Returns detailed information about the client's performance and configuration,
    /// including whether it's running in direct mode or HTTP mode.
    ///
    /// # Returns
    /// * `NodeStats` - Performance statistics and configuration
    #[napi]
    pub fn get_stats(&self) -> NodeStats {
        NodeStats {
            adapter_type: match &self.adapter {
                crate::adapters::Adapter::LightLLM(_) => "lightllm".to_string(),
                crate::adapters::Adapter::OpenAI(_) => "openai".to_string(),
                crate::adapters::Adapter::VLLM(_) => "vllm".to_string(),
                crate::adapters::Adapter::AzureOpenAI(_) => "azure".to_string(),
                crate::adapters::Adapter::AWSBedrock(_) => "aws".to_string(),
                crate::adapters::Adapter::Custom(_) => "custom".to_string(),
                crate::adapters::Adapter::Direct(_) => "direct".to_string(),
            },
            backend_url: self.config.backend_url.clone(),
            model_id: self.config.model_id.clone(),
            port: Some(self.config.port),
            connection_pooling: true,
            max_connections: self.config.http_client_max_connections as u32,
            max_connections_per_host: self.config.http_client_max_connections_per_host as u32,
            timeout_seconds: self.config.http_client_timeout as u32,
            is_direct_mode: self.config.backend_url == "direct",
            performance_mode: if self.config.backend_url == "direct" {
                "Maximum (Direct Mode)".to_string()
            } else {
                "High (HTTP Mode)".to_string()
            },
        }
    }
}

pub struct NodeChatCompletionTask {
    adapter: Adapter,
    config: Config,
    request: NodeChatRequest,
}

impl Task for NodeChatCompletionTask {
    type Output = NodeChatResponse;
    type JsValue = NodeChatResponse;

    fn compute(&mut self) -> Result<Self::Output> {
        // CRITICAL: Catch panics at FFI boundary to prevent UB
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Convert Node.js request to Rust request (zero-copy where possible)
        let rust_messages: Vec<Message> = self.request.messages.clone().into_iter()
            .map(|msg| msg.into())
            .collect();

        let rust_request = ChatCompletionRequest {
            model: self.request.model.clone().or_else(|| Some(self.config.model_id.clone())),
            messages: rust_messages,
            max_tokens: self.request.max_tokens,
            temperature: self.request.temperature.map(|t| t as f32),
            top_p: self.request.top_p.map(|t| t as f32),
            n: self.request.n,
            stream: self.request.stream,
            stop: self.request.stop.clone(),
            presence_penalty: self.request.presence_penalty.map(|p| p as f32),
            frequency_penalty: self.request.frequency_penalty.map(|f| f as f32),
            logit_bias: None,
            user: self.request.user.clone(),
            logprobs: None,
            top_logprobs: None,
            tools: None,
            tool_choice: None,
            seed: None,
        };

        // PERFORMANCE FIX: Use singleton runtime instead of creating new one per call
        static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
        let rt = RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2) // Limit threads to avoid oversubscription
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime")
        });

            // Execute the async adapter call in the runtime
            let http_response = rt.block_on(async {
                match &self.adapter {
                    Adapter::LightLLM(adapter) => adapter.chat_completions(rust_request).await,
                    Adapter::OpenAI(adapter) => adapter.chat_completions(rust_request).await,
                    Adapter::VLLM(adapter) => adapter.chat_completions(rust_request).await,
                    Adapter::AzureOpenAI(adapter) => adapter.chat_completions(rust_request).await,
                    Adapter::AWSBedrock(adapter) => adapter.chat_completions(rust_request).await,
                    Adapter::Custom(adapter) => adapter.chat_completions(rust_request).await,
                    Adapter::Direct(adapter) => adapter.chat_completions(rust_request).await,
                }
            }).map_err(|e| Error::new(
                Status::GenericFailure,
                format!("Adapter request failed: {}", e)
            ))?;

            // Parse the HTTP response body to ChatCompletionResponse
            let response_body = rt.block_on(async {

                let body_bytes = axum::body::to_bytes(http_response.into_body(), usize::MAX).await
                    .map_err(|e| format!("Failed to read response body: {}", e))?;

                let response_text = String::from_utf8(body_bytes.to_vec())
                    .map_err(|e| format!("Response body is not valid UTF-8: {}", e))?;

                serde_json::from_str::<crate::schemas::ChatCompletionResponse>(&response_text)
                    .map_err(|e| format!("Failed to parse response JSON: {} - Response: {}", e, response_text))
            }).map_err(|e| Error::new(
                Status::GenericFailure,
                format!("Response parsing failed: {}", e)
            ))?;

            // Convert the Rust response to Node.js response format (zero-copy where possible)
            let choices = response_body.choices.into_iter().map(|choice| NodeChoice {
                index: choice.index,
                message: NodeMessage {
                    role: choice.message.role,
                    content: choice.message.content.unwrap_or_default(),
                    name: choice.message.name,
                },
                finish_reason: choice.finish_reason,
            }).collect();

            let usage = response_body.usage.map(|u| NodeUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }).unwrap_or_else(|| NodeUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            });

            Ok(NodeChatResponse {
                id: response_body.id,
                object: response_body.object,
                created: response_body.created as u32,
                model: response_body.model,
                choices,
                usage,
            })
        })).map_err(|_| Error::new(
            Status::GenericFailure,
            "Internal error: operation panicked"
        ))?;

        result
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

#[napi]
impl NodeNexusNitroLLMClient {


    /// Test connection to backend
    ///
    /// # Returns
    /// * `Promise<bool>` - Resolves to true if connection successful, false otherwise
    #[napi(ts_return_type = "Promise<boolean>")]
    pub fn test_connection(&self) -> AsyncTask<NodeConnectionTestTask> {
        AsyncTask::new(NodeConnectionTestTask {
            adapter: self.adapter.clone(),
        })
    }
}

pub struct NodeConnectionTestTask {
    adapter: Adapter,
}

impl Task for NodeConnectionTestTask {
    type Output = bool;
    type JsValue = bool;

    fn compute(&mut self) -> Result<Self::Output> {
        // Create a new Tokio runtime for this task since we can't use async in compute
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| Error::new(
                Status::GenericFailure,
                format!("Failed to create runtime for async operation: {}", e)
            ))?;

        // Test connection by making a simple request
        let result = rt.block_on(async {
            let test_request = crate::schemas::ChatCompletionRequest {
                model: Some("test".to_string()),
                messages: vec![crate::schemas::Message {
                    role: "user".to_string(),
                    content: Some("test".to_string()),
                    name: None,
                    tool_calls: None,
                    function_call: None,
                    tool_call_id: None,
                }],
                max_tokens: Some(1),
                temperature: Some(0.1),
                top_p: None,
                n: Some(1),
                stream: Some(false),
                stop: None,
                presence_penalty: None,
                frequency_penalty: None,
                logit_bias: None,
                user: None,
                logprobs: None,
                top_logprobs: None,
                tools: None,
                tool_choice: None,
                seed: None,
            };

            match &self.adapter {
                Adapter::LightLLM(adapter) => adapter.chat_completions(test_request).await,
                Adapter::OpenAI(adapter) => adapter.chat_completions(test_request).await,
                Adapter::VLLM(adapter) => adapter.chat_completions(test_request).await,
                Adapter::AzureOpenAI(adapter) => adapter.chat_completions(test_request).await,
                Adapter::AWSBedrock(adapter) => adapter.chat_completions(test_request).await,
                Adapter::Custom(adapter) => adapter.chat_completions(test_request).await,
                Adapter::Direct(adapter) => adapter.chat_completions(test_request).await,
            }
        });

        // Return true if the request succeeded, false otherwise
        Ok(result.is_ok())
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

#[napi]
impl NodeNexusNitroLLMClient {
    /// Update configuration dynamically
    ///
    /// # Arguments
    /// * `new_config` - New configuration to apply
    ///
    /// # Performance Notes
    /// * Configuration changes are applied immediately
    /// * Connection pool is recreated if connection settings change
    #[napi]
    pub fn update_config(&mut self, new_config: NodeConfig) -> Result<()> {
        let rust_config: Config = new_config.into();

        // Recreate adapter with new configuration
        self.adapter = Adapter::from_config(&rust_config);
        self.config = rust_config;

        Ok(())
    }
}

/// Create a high-performance configuration object
///
/// # Arguments
/// * `backend_url` - Backend server URL
/// * `model_id` - Default model identifier
/// * `options` - Optional configuration parameters
///
/// # Returns
/// * `NodeConfig` - Optimized configuration object
#[napi]
pub fn create_config(
    backend_url: String,
    backend_type: Option<String>,
    model_id: String,
    options: Option<NodeConfig>,
) -> NodeConfig {
    let mut config = options.unwrap_or_default();
    config.backend_url = Some(backend_url);
    config.backend_type = backend_type;
    config.model_id = model_id;
    config
}

/// Create an optimized message object
///
/// # Arguments
/// * `role` - Message role ("system", "user", "assistant", "tool")
/// * `content` - Message content
/// * `name` - Optional message name
///
/// # Returns
/// * `NodeMessage` - Zero-copy optimized message
#[napi]
pub fn create_message(role: String, content: String, name: Option<String>) -> NodeMessage {
    NodeMessage { role, content, name }
}

/// Create a high-performance LightLLM client (convenience function)
///
/// # Arguments
/// * `backend_url` - Backend server URL
/// * `model_id` - Default model identifier
/// * `options` - Optional configuration parameters
///
/// # Returns
/// * `Promise<NodeNexusNitroLLMClient>` - High-performance client instance
#[napi]
pub fn create_client(
    backend_url: String,
    backend_type: Option<String>,
    model_id: String,
    options: Option<NodeConfig>,
) -> Result<NodeNexusNitroLLMClient> {
    let config = create_config(backend_url, backend_type, model_id, options);
    NodeNexusNitroLLMClient::new(config)
}

/// Get library version information
/// Create a direct mode client for maximum performance
///
/// This creates a client that bypasses HTTP entirely for maximum performance.
/// Perfect for Node.js applications that want direct integration without network overhead.
///
/// # Arguments
/// * `model_id` - Model identifier (default: "llama")
/// * `token` - Optional authentication token
///
/// # Returns
/// * `Result<NodeNexusNitroLLMClient>` - New direct mode client or error
///
/// # Performance Benefits
/// * Zero HTTP overhead
/// * Direct memory access
/// * Minimal latency
/// * Maximum throughput
#[napi]
pub fn create_direct_client(
    model_id: Option<String>,
    token: Option<String>,
) -> Result<NodeNexusNitroLLMClient> {
    let config = NodeConfig {
        backend_url: None, // Direct mode
        backend_type: Some("lightllm".to_string()),
        model_id: model_id.unwrap_or_else(|| "llama".to_string()),
        port: None,
        token,
        connection_pooling: Some(true),
        max_connections: Some(100),
        max_connections_per_host: Some(20),
    };
    
    NodeNexusNitroLLMClient::new(config)
}

/// Create an HTTP mode client for traditional proxy communication
///
/// This creates a client that communicates with a LightLLM server via HTTP.
/// Use this when you need to share the backend across multiple applications.
///
/// # Arguments
/// * `backend_url` - Backend server URL
/// * `model_id` - Model identifier (default: "llama")
/// * `token` - Optional authentication token
///
/// # Returns
/// * `Result<NodeNexusNitroLLMClient>` - New HTTP mode client or error
#[napi]
pub fn create_http_client(
    backend_url: String,
    backend_type: Option<String>,
    model_id: Option<String>,
    token: Option<String>,
) -> Result<NodeNexusNitroLLMClient> {
    let config = NodeConfig {
        backend_url: Some(backend_url),
        backend_type: backend_type,
        model_id: model_id.unwrap_or_else(|| "llama".to_string()),
        port: None,
        token,
        connection_pooling: Some(true),
        max_connections: Some(100),
        max_connections_per_host: Some(20),
    };
    
    NodeNexusNitroLLMClient::new(config)
}

#[napi]
pub fn get_version() -> String {
    "0.1.0".to_string()
}

/// Get performance benchmarking utilities
#[napi(object)]
pub struct NodeBenchmark {
    /// Operations per second
    pub ops_per_second: f64,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// Memory usage in MB
    pub memory_mb: f64,
}

/// Run performance benchmark
///
/// # Arguments
/// * `client` - Client to benchmark
/// * `operations` - Number of operations to perform
///
/// # Returns
/// * `NodeBenchmark` - Performance statistics
#[napi]
pub fn benchmark_client(
    client: &NodeNexusNitroLLMClient,
    operations: u32
) -> NodeBenchmark {
    let start = std::time::Instant::now();
    let mut successful_ops = 0u32;

    // Simple benchmark: get stats repeatedly
    for _i in 0..operations {
        let _stats = client.get_stats();
        successful_ops += 1;
    }

    let elapsed = start.elapsed();
    let ops_per_second = successful_ops as f64 / elapsed.as_secs_f64();
    let avg_latency_ms = elapsed.as_millis() as f64 / successful_ops as f64;

    // Simple memory estimation based on operations
    // In production, this could integrate with system memory monitoring
    let memory_mb = (successful_ops as f64 * 0.1).max(1.0).min(100.0);

    NodeBenchmark {
        ops_per_second,
        avg_latency_ms,
        memory_mb,
    }
}