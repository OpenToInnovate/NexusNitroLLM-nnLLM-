//! # Schemas Module
//! 
//! This module contains all the data structures for OpenAI-compatible chat completions,
//! including support for both regular and streaming responses.
//! 
//! ## Key Concepts for C++ Developers:
//! 
//! - **Serde**: Rust's serialization framework, similar to nlohmann/json in C++
//! - **Option<T>**: Similar to `std::optional<T>` in C++17+
//! - **Vec<T>**: Similar to `std::vector<T>` in C++
//! - **HashMap<K, V>**: Similar to `std::unordered_map<K, V>` in C++

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// # Chat Completion Request
/// 
/// OpenAI-compatible chat completion request structure with support for streaming.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ChatCompletionRequest {
    /// List of messages in the conversation
    pub messages: Vec<Message>,
    /// Model identifier (optional, uses default if not provided)
    pub model: Option<String>,
    /// Maximum number of tokens to generate
    pub max_tokens: Option<u32>,
    /// Sampling temperature (0.0 to 2.0)
    pub temperature: Option<f32>,
    /// Nucleus sampling parameter (0.0 to 1.0)
    pub top_p: Option<f32>,
    /// Whether to stream the response (Server-Sent Events)
    pub stream: Option<bool>,
    /// Stop sequences to end generation
    pub stop: Option<Vec<String>>,
    /// Presence penalty (-2.0 to 2.0)
    pub presence_penalty: Option<f32>,
    /// Frequency penalty (-2.0 to 2.0)
    pub frequency_penalty: Option<f32>,
    /// Logit bias map for token adjustment
    pub logit_bias: Option<HashMap<String, f32>>,
    /// User identifier for tracking
    pub user: Option<String>,
    /// Number of completions to generate
    pub n: Option<u32>,
    /// Random seed for reproducible generation
    pub seed: Option<u64>,
    /// Whether to return log probabilities
    pub logprobs: Option<bool>,
    /// Number of top log probabilities to return
    pub top_logprobs: Option<u32>,
    /// List of tools available to the model
    pub tools: Option<Vec<Tool>>,
    /// Tool choice configuration
    pub tool_choice: Option<ToolChoice>,
}

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    pub content: Option<String>,
    pub name: Option<String>,
    /// Tool calls made by the assistant
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Function call (legacy OpenAI format)
    pub function_call: Option<FunctionCall>,
    /// Tool call ID (for tool role messages)
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: String,
    pub logprobs: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// # Streaming Response Structures
/// 
/// These structures implement OpenAI's Server-Sent Events (SSE) format
/// for streaming chat completions. Each chunk represents a partial response.
/// # Chat Completion Chunk (SSE Format)
/// 
/// Represents a single chunk in a streaming chat completion response.
/// This is the format sent over Server-Sent Events.
/// 
#[derive(Debug, Serialize)]
pub struct ChatCompletionChunk {
    /// Unique identifier for the completion
    pub id: String,
    /// Object type (always "chat.completion.chunk")
    pub object: String,
    /// Unix timestamp when the completion was created
    pub created: i64,
    /// Model identifier
    pub model: String,
    /// List of completion choices
    pub choices: Vec<StreamChoice>,
    /// Token usage (only in final chunk)
    pub usage: Option<Usage>,
}

/// # Stream Choice
/// 
/// Represents a single choice in a streaming completion chunk.
/// 
#[derive(Debug, Serialize)]
pub struct StreamChoice {
    /// Index of the choice
    pub index: u32,
    /// Delta content for this chunk
    pub delta: StreamDelta,
    /// Finish reason (null until final chunk)
    pub finish_reason: Option<String>,
}

/// # Stream Delta
/// 
/// Represents the delta (change) content in a streaming response.
/// 
#[derive(Debug, Serialize)]
pub struct StreamDelta {
    /// Role (only in first chunk)
    pub role: Option<String>,
    /// Content delta
    pub content: Option<String>,
    /// Tool calls (for function calling)
    pub tool_calls: Option<Vec<StreamToolCall>>,
    /// Function call (legacy)
    pub function_call: Option<StreamFunctionCall>,
}

/// # Streaming Tool Call
/// 
/// Represents a tool call in a streaming response.
#[derive(Debug, Serialize)]
pub struct StreamToolCall {
    /// Tool call index
    pub index: u32,
    /// Tool call ID (only in first chunk)
    pub id: Option<String>,
    /// Tool type (only in first chunk)
    #[serde(rename = "type")]
    pub tool_type: Option<String>,
    /// Function call details
    pub function: Option<StreamFunctionCall>,
}

/// # Streaming Function Call
/// 
/// Represents a function call in a streaming response.
#[derive(Debug, Serialize)]
pub struct StreamFunctionCall {
    /// Function name (only in first chunk)
    pub name: Option<String>,
    /// Function arguments (streamed)
    pub arguments: Option<String>,
}

/// # Streaming Error Response
/// 
/// Error response format for streaming requests.
/// 
#[derive(Debug, Serialize)]
pub struct StreamingError {
    /// Error details
    pub error: ErrorDetails,
}

/// # Error Details
/// 
/// Detailed error information for streaming responses.
/// 
#[derive(Debug, Serialize)]
pub struct ErrorDetails {
    /// Error message
    pub message: String,
    /// Error type
    pub r#type: String,
    /// Error code (optional)
    pub code: Option<String>,
}

/// # SSE Event Types
/// 
/// Types of Server-Sent Events for streaming responses.
#[derive(Debug, Clone)]
pub enum SSEEventType {
    /// Data chunk event
    Data,
    /// Error event
    Error,
    /// Keep-alive ping event
    Ping,
}

/// # SSE Event
/// 
/// Represents a single Server-Sent Event with proper formatting.
#[derive(Debug)]
pub struct SSEEvent {
    /// Event type
    pub event_type: SSEEventType,
    /// Event data (JSON string for data/error, empty for ping)
    pub data: String,
}

impl SSEEvent {
    /// # Create a data event
    /// 
    /// Creates an SSE data event with the given JSON data.
    pub fn data(json_data: String) -> Self {
        Self {
            event_type: SSEEventType::Data,
            data: json_data,
        }
    }
    
    /// # Create an error event
    /// 
    /// Creates an SSE error event with the given error JSON.
    pub fn error(error_json: String) -> Self {
        Self {
            event_type: SSEEventType::Error,
            data: error_json,
        }
    }
    
    /// # Create a ping event
    /// 
    /// Creates an SSE keep-alive ping event.
    pub fn ping() -> Self {
        Self {
            event_type: SSEEventType::Ping,
            data: String::new(),
        }
    }
    
    /// # Format as SSE string
    /// 
    /// Formats the event as a proper Server-Sent Events string.
    /// 
    /// ## Format:
    /// - Data events: `data: <json>\n\n`
    /// - Error events: `data: <json>\n\n`
    /// - Ping events: `: ping\n`
    pub fn to_sse_string(&self) -> String {
        match &self.event_type {
            SSEEventType::Data | SSEEventType::Error => {
                format!("data: {}\n\n", self.data)
            }
            SSEEventType::Ping => {
                ": ping\n".to_string()
            }
        }
    }
}

/// # Function Calling Support
/// 
/// OpenAI-compatible function calling structures for tool and function support.
/// # Tool Definition
/// 
/// Defines a tool (function) that the model can call.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tool {
    /// Tool type (currently only "function" is supported)
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function definition
    pub function: FunctionDefinition,
}

/// # Function Definition
/// 
/// Defines a function that can be called by the model.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FunctionDefinition {
    /// Function name
    pub name: String,
    /// Function description
    pub description: Option<String>,
    /// Function parameters schema (JSON Schema format)
    pub parameters: Option<serde_json::Value>,
}

/// # Tool Choice
/// 
/// Controls which tool the model should use.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// No tools (model should not call any tools)
    None,
    /// Auto (model can choose whether to call tools)
    Auto,
    /// Required (model must call a tool)
    Required,
    /// Specific tool choice
    Specific {
        /// Tool type
        #[serde(rename = "type")]
        tool_type: String,
        /// Function name
        function: FunctionChoice,
    },
}

/// # Function Choice
/// 
/// Specific function choice for tool selection.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FunctionChoice {
    /// Function name
    pub name: String,
}

/// # Tool Call
/// 
/// Represents a tool call made by the model.
#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct ToolCall {
    /// Tool call ID
    pub id: String,
    /// Tool type (currently only "function")
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function call details
    pub function: FunctionCall,
}

/// # Function Call
/// 
/// Represents a function call with name and arguments.
#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct FunctionCall {
    /// Function name
    pub name: String,
    /// Function arguments (JSON string)
    pub arguments: String,
}

/// # Tool Call Result
/// 
/// Result of executing a tool call.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolCallResult {
    /// Tool call ID
    pub tool_call_id: String,
    /// Result content
    pub content: String,
    /// Whether the tool call was successful
    pub is_error: Option<bool>,
}

/// # Function Calling Utilities
/// 
/// Helper functions for working with function calls.
impl FunctionCall {
    /// # Create a new function call
    /// 
    /// Creates a new function call with the given name and arguments.
    /// 
    /// ## Parameters:
    /// - `name`: Function name
    /// - `arguments`: Function arguments as a JSON string
    /// 
    /// ## Returns:
    /// - `Self`: New function call instance
    pub fn new(name: String, arguments: String) -> Self {
        Self { name, arguments }
    }
    
    /// # Parse arguments as JSON
    /// 
    /// Parses the arguments string as JSON and returns the parsed value.
    /// 
    /// ## Returns:
    /// - `Result<serde_json::Value, serde_json::Error>`: Parsed JSON or error
    pub fn parse_arguments(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.arguments)
    }
    
    /// # Create arguments from JSON
    /// 
    /// Creates a function call with arguments serialized from a JSON value.
    /// 
    /// ## Parameters:
    /// - `name`: Function name
    /// - `args`: Arguments as a JSON value
    /// 
    /// ## Returns:
    /// - `Result<Self, serde_json::Error>`: New function call or error
    pub fn with_json_args(name: String, args: &serde_json::Value) -> Result<Self, serde_json::Error> {
        let arguments = serde_json::to_string(args)?;
        Ok(Self::new(name, arguments))
    }
}

impl ToolCall {
    /// # Create a new tool call
    /// 
    /// Creates a new tool call with the given ID and function call.
    /// 
    /// ## Parameters:
    /// - `id`: Tool call ID
    /// - `function_call`: Function call details
    /// 
    /// ## Returns:
    /// - `Self`: New tool call instance
    pub fn new(id: String, function_call: FunctionCall) -> Self {
        Self {
            id,
            tool_type: "function".to_string(),
            function: function_call,
        }
    }
}

impl Tool {
    /// # Create a new function tool
    /// 
    /// Creates a new tool that represents a function.
    /// 
    /// ## Parameters:
    /// - `name`: Function name
    /// - `description`: Function description
    /// - `parameters`: Function parameters schema (JSON Schema)
    /// 
    /// ## Returns:
    /// - `Self`: New tool instance
    pub fn new_function(name: String, description: Option<String>, parameters: Option<serde_json::Value>) -> Self {
        Self {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name,
                description,
                parameters,
            },
        }
    }
}

impl Message {
    /// # Create a system message
    /// 
    /// Creates a new system message.
    /// 
    /// ## Parameters:
    /// - `content`: Message content
    /// 
    /// ## Returns:
    /// - `Self`: New system message
    pub fn system(content: String) -> Self {
        Self {
            role: "system".to_string(),
            content: Some(content),
            name: None,
            tool_calls: None,
            function_call: None,
            tool_call_id: None,
        }
    }
    
    /// # Create a user message
    /// 
    /// Creates a new user message.
    /// 
    /// ## Parameters:
    /// - `content`: Message content
    /// 
    /// ## Returns:
    /// - `Self`: New user message
    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content: Some(content),
            name: None,
            tool_calls: None,
            function_call: None,
            tool_call_id: None,
        }
    }
    
    /// # Create an assistant message
    /// 
    /// Creates a new assistant message.
    /// 
    /// ## Parameters:
    /// - `content`: Message content (optional for tool calls)
    /// 
    /// ## Returns:
    /// - `Self`: New assistant message
    pub fn assistant(content: Option<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
            name: None,
            tool_calls: None,
            function_call: None,
            tool_call_id: None,
        }
    }
    
    /// # Create a tool message
    /// 
    /// Creates a new tool message with the result of a tool call.
    /// 
    /// ## Parameters:
    /// - `tool_call_id`: ID of the tool call
    /// - `content`: Tool call result content
    /// 
    /// ## Returns:
    /// - `Self`: New tool message
    pub fn tool(tool_call_id: String, content: String) -> Self {
        Self {
            role: "tool".to_string(),
            content: Some(content),
            name: None,
            tool_calls: None,
            function_call: None,
            tool_call_id: Some(tool_call_id),
        }
    }
    
    /// # Add tool calls to assistant message
    /// 
    /// Adds tool calls to an assistant message.
    /// 
    /// ## Parameters:
    /// - `tool_calls`: List of tool calls
    /// 
    /// ## Returns:
    /// - `Self`: Updated message with tool calls
    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.tool_calls = Some(tool_calls);
        self
    }
    
    /// # Add function call to assistant message (legacy)
    /// 
    /// Adds a function call to an assistant message (legacy OpenAI format).
    /// 
    /// ## Parameters:
    /// - `function_call`: Function call details
    /// 
    /// ## Returns:
    /// - `Self`: Updated message with function call
    pub fn with_function_call(mut self, function_call: FunctionCall) -> Self {
        self.function_call = Some(function_call);
        self
    }
}