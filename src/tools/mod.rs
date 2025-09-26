//! # Tools Module
//!
//! This module consolidates all tool and function calling functionality
//! that was previously split between function_calling.rs and tool_support.rs.
//! It provides a unified interface for tool execution, validation, and streaming.

pub mod registry;
pub mod executor;
pub mod validation;
pub mod streaming;
pub mod message_builder;

// Re-export commonly used tool types
pub use registry::{FunctionRegistry, FunctionDefinition};
pub use executor::{ToolCallExecutor, FunctionExecutor};
pub use validation::ToolCallValidator;
pub use streaming::ToolCallStreamProcessor;
pub use message_builder::{ToolCallMessageBuilder, ToolCallResponseFormatter};

// Re-export schemas for convenience
pub use crate::schemas::{Tool, ToolCall, ToolChoice, FunctionCall};

/// Tool execution errors
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Function not found: {name}")]
    FunctionNotFound { name: String },

    #[error("Invalid tool choice: {message}")]
    InvalidToolChoice { message: String },

    #[error("Tool execution failed: {message}")]
    ExecutionFailed { message: String },

    #[error("Tool validation failed: {message}")]
    ValidationFailed { message: String },

    #[error("Serialization error: {source}")]
    Serialization { #[from] source: serde_json::Error },
}

/// Tool call history entry for tracking
#[derive(Debug, Clone)]
pub struct ToolCallHistoryEntry {
    pub tool_call_id: String,
    pub function_name: String,
    pub arguments: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub timestamp: std::time::SystemTime,
}

impl ToolCallHistoryEntry {
    /// Create a new tool call history entry
    pub fn new(tool_call_id: String, function_name: String, arguments: serde_json::Value) -> Self {
        Self {
            tool_call_id,
            function_name,
            arguments,
            result: None,
            error: None,
            timestamp: std::time::SystemTime::now(),
        }
    }

    /// Mark the entry as successful with a result
    pub fn with_result(mut self, result: serde_json::Value) -> Self {
        self.result = Some(result);
        self
    }

    /// Mark the entry as failed with an error
    pub fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self
    }
}

/// Tool role for message handling
#[derive(Debug, Clone, PartialEq)]
pub enum ToolRole {
    User,
    Assistant,
    Tool,
    System,
}

impl std::str::FromStr for ToolRole {
    type Err = ToolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(ToolRole::User),
            "assistant" => Ok(ToolRole::Assistant),
            "tool" => Ok(ToolRole::Tool),
            "system" => Ok(ToolRole::System),
            _ => Err(ToolError::ValidationFailed {
                message: format!("Unknown role: {}", s),
            }),
        }
    }
}

impl std::fmt::Display for ToolRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let role_str = match self {
            ToolRole::User => "user",
            ToolRole::Assistant => "assistant",
            ToolRole::Tool => "tool",
            ToolRole::System => "system",
        };
        write!(f, "{}", role_str)
    }
}

/// Tool use message for enhanced message handling
#[derive(Debug, Clone)]
pub struct ToolUseMessage {
    /// Message role
    pub role: ToolRole,
    /// Message content
    pub content: Option<String>,
    /// Tool calls made by the assistant
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Tool call ID (for tool role messages)
    pub tool_call_id: Option<String>,
    /// Message name
    pub name: Option<String>,
}

impl ToolUseMessage {
    /// Create a new tool use message
    pub fn new(role: ToolRole, content: Option<String>) -> Self {
        Self {
            role,
            content,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    /// Set tool calls for the message
    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.tool_calls = Some(tool_calls);
        self
    }

    /// Set tool call ID for the message
    pub fn with_tool_call_id(mut self, tool_call_id: String) -> Self {
        self.tool_call_id = Some(tool_call_id);
        self
    }

    /// Set name for the message
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_role_parsing() {
        assert_eq!("user".parse::<ToolRole>().unwrap(), ToolRole::User);
        assert_eq!("assistant".parse::<ToolRole>().unwrap(), ToolRole::Assistant);
        assert_eq!("tool".parse::<ToolRole>().unwrap(), ToolRole::Tool);
        assert_eq!("system".parse::<ToolRole>().unwrap(), ToolRole::System);

        assert!("invalid".parse::<ToolRole>().is_err());
    }

    #[test]
    fn test_tool_role_display() {
        assert_eq!(ToolRole::User.to_string(), "user");
        assert_eq!(ToolRole::Assistant.to_string(), "assistant");
        assert_eq!(ToolRole::Tool.to_string(), "tool");
        assert_eq!(ToolRole::System.to_string(), "system");
    }

    #[test]
    fn test_tool_use_message_creation() {
        let message = ToolUseMessage::new(ToolRole::User, Some("Hello".to_string()))
            .with_name("test_user".to_string());

        assert_eq!(message.role, ToolRole::User);
        assert_eq!(message.content, Some("Hello".to_string()));
        assert_eq!(message.name, Some("test_user".to_string()));
    }

    #[test]
    fn test_tool_call_history_entry() {
        let entry = ToolCallHistoryEntry::new(
            "call_123".to_string(),
            "test_function".to_string(),
            serde_json::json!({"param": "value"}),
        ).with_result(serde_json::json!({"result": "success"}));

        assert_eq!(entry.tool_call_id, "call_123");
        assert_eq!(entry.function_name, "test_function");
        assert!(entry.result.is_some());
        assert!(entry.error.is_none());
    }
}