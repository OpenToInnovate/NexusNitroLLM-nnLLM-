//! # Tool Call Message Builder
//!
//! This module provides message building functionality for tool calls,
//! handling message formatting and response construction.

use crate::{
    schemas::{
        ToolCall, FunctionCall, ChatCompletionResponse,
        Choice, Usage, Message
    },
    error::ProxyError,
};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use super::{ToolCallHistoryEntry, ToolRole, ToolUseMessage};

/// Tool call message builder for constructing chat messages with tool calls
#[derive(Clone, Debug)]
pub struct ToolCallMessageBuilder {
    /// Current message being built
    current_message: Option<Message>,
    /// Tool calls for the current message
    tool_calls: Vec<ToolCall>,
    /// Message history
    message_history: Vec<Message>,
    /// Next message ID counter
    message_counter: usize,
}

impl ToolCallMessageBuilder {
    /// Create a new tool call message builder
    pub fn new() -> Self {
        Self {
            current_message: None,
            tool_calls: Vec::new(),
            message_history: Vec::new(),
            message_counter: 0,
        }
    }

    /// Start building a new user message
    pub fn user_message(mut self, content: String) -> Self {
        self.current_message = Some(Message {
            role: "user".to_string(),
            content: Some(content),
            name: None,
            tool_calls: None,
            function_call: None,
            tool_call_id: None,
        });
        self
    }

    /// Start building a new assistant message
    pub fn assistant_message(mut self, content: Option<String>) -> Self {
        self.current_message = Some(Message {
            role: "assistant".to_string(),
            content,
            name: None,
            tool_calls: None,
            function_call: None,
            tool_call_id: None,
        });
        self
    }

    /// Start building a new tool message
    pub fn tool_message(mut self, tool_call_id: String, content: String) -> Self {
        self.current_message = Some(Message {
            role: "tool".to_string(),
            content: Some(content),
            name: None,
            tool_calls: None,
            function_call: None,
            tool_call_id: Some(tool_call_id),
        });
        self
    }

    /// Add a tool call to the current message
    pub fn with_tool_call(mut self, tool_call: ToolCall) -> Self {
        self.tool_calls.push(tool_call);
        self
    }

    /// Add multiple tool calls to the current message
    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.tool_calls.extend(tool_calls);
        self
    }

    /// Set the message name
    pub fn with_name(mut self, name: String) -> Self {
        if let Some(ref mut message) = self.current_message {
            message.name = Some(name);
        }
        self
    }

    /// Finalize the current message and add it to history
    pub fn build_message(mut self) -> Result<(Self, Message), ProxyError> {
        let mut message = self.current_message.take()
            .ok_or_else(|| ProxyError::Internal("No current message to build".to_string()))?;

        // Add tool calls if any
        if !self.tool_calls.is_empty() {
            message.tool_calls = Some(self.tool_calls.clone());
        }

        // Add to history
        self.message_history.push(message.clone());
        self.message_counter += 1;

        // Clear tool calls for next message
        self.tool_calls.clear();

        Ok((self, message))
    }

    /// Get all messages in history
    pub fn messages(&self) -> &[Message] {
        &self.message_history
    }

    /// Convert message history to completion messages (already Message type)
    pub fn to_completion_messages(&self) -> Vec<Message> {
        self.message_history.clone()
    }

    /// Clear all message history
    pub fn clear_history(mut self) -> Self {
        self.message_history.clear();
        self.message_counter = 0;
        self
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.message_counter
    }
}

impl Default for ToolCallMessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool call response formatter for creating completion responses
pub struct ToolCallResponseFormatter {
    /// Request ID generator
    id_generator: Box<dyn Fn() -> String + Send + Sync>,
    /// Model name
    model: String,
    /// Default usage stats
    default_usage: Usage,
}

impl ToolCallResponseFormatter {
    /// Create a new response formatter
    pub fn new(model: String) -> Self {
        Self {
            id_generator: Box::new(|| format!("chatcmpl-{}", &Uuid::new_v4().to_string()[..8])),
            model,
            default_usage: Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
        }
    }

    /// Set custom ID generator
    pub fn with_id_generator<F>(mut self, generator: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.id_generator = Box::new(generator);
        self
    }

    /// Create a completion response with tool calls
    pub fn create_tool_call_response(
        &self,
        content: Option<String>,
        tool_calls: Vec<ToolCall>,
        usage: Option<Usage>,
    ) -> ChatCompletionResponse {
        ChatCompletionResponse {
            id: (self.id_generator)(),
            object: "chat.completion".to_string(),
            created: current_timestamp(),
            model: self.model.clone(),
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: "assistant".to_string(),
                    content,
                    name: None,
                    tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
                    function_call: None,
                    tool_call_id: None,
                },
                finish_reason: "tool_calls".to_string(),
                logprobs: None,
            }],
            usage: Some(usage.unwrap_or(self.default_usage.clone())),
        }
    }

    /// Create a completion response from tool call results
    pub fn create_tool_result_response(
        &self,
        results: Vec<(String, Result<Value, String>)>,
        usage: Option<Usage>,
    ) -> ChatCompletionResponse {
        let content = self.format_tool_results(&results);

        ChatCompletionResponse {
            id: (self.id_generator)(),
            object: "chat.completion".to_string(),
            created: current_timestamp(),
            model: self.model.clone(),
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: "assistant".to_string(),
                    content: Some(content),
                    name: None,
                    tool_calls: None,
                    function_call: None,
                    tool_call_id: None,
                },
                finish_reason: "stop".to_string(),
                logprobs: None,
            }],
            usage: Some(usage.unwrap_or(self.default_usage.clone())),
        }
    }

    /// Create error response for tool call failures
    pub fn create_error_response(&self, error: ProxyError) -> ChatCompletionResponse {
        let error_content = format!("Tool call failed: {}", error);

        ChatCompletionResponse {
            id: (self.id_generator)(),
            object: "chat.completion".to_string(),
            created: current_timestamp(),
            model: self.model.clone(),
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: "assistant".to_string(),
                    content: Some(error_content),
                    name: None,
                    tool_calls: None,
                    function_call: None,
                    tool_call_id: None,
                },
                finish_reason: "error".to_string(),
                logprobs: None,
            }],
            usage: Some(self.default_usage.clone()),
        }
    }

    /// Format tool results into a readable response
    fn format_tool_results(&self, results: &[(String, Result<Value, String>)]) -> String {
        let mut formatted = String::new();

        formatted.push_str("Tool execution results:\n\n");

        for (tool_call_id, result) in results {
            formatted.push_str(&format!("Tool Call ID: {}\n", tool_call_id));

            match result {
                Ok(value) => {
                    formatted.push_str("Status: Success\n");
                    formatted.push_str(&format!(
                        "Result: {}\n\n",
                        serde_json::to_string_pretty(value).unwrap_or_else(|_| "Invalid JSON".to_string())
                    ));
                }
                Err(error) => {
                    formatted.push_str("Status: Error\n");
                    formatted.push_str(&format!("Error: {}\n\n", error));
                }
            }
        }

        formatted
    }
}

/// Convert tool call history to tool use messages
pub fn history_to_tool_messages(history: &[ToolCallHistoryEntry]) -> Vec<ToolUseMessage> {
    history
        .iter()
        .map(|entry| {
            let mut message = ToolUseMessage::new(
                ToolRole::Tool,
                Some(format!(
                    "Function: {}\nArguments: {}\nResult: {}",
                    entry.function_name,
                    serde_json::to_string_pretty(&entry.arguments).unwrap_or_default(),
                    entry.result.as_ref()
                        .map(|r| serde_json::to_string_pretty(r).unwrap_or_default())
                        .or_else(|| entry.error.as_ref().map(|e| format!("Error: {}", e)))
                        .unwrap_or_else(|| "No result".to_string())
                ))
            );

            message = message
                .with_tool_call_id(entry.tool_call_id.clone())
                .with_name(entry.function_name.clone());

            message
        })
        .collect()
}

/// Create a function call from a tool call
pub fn tool_call_to_function_call(tool_call: &ToolCall) -> FunctionCall {
    tool_call.function.clone()
}

/// Create a tool call from a function call
pub fn function_call_to_tool_call(function_call: FunctionCall) -> ToolCall {
    ToolCall {
        id: format!("call_{}", &Uuid::new_v4().to_string()[..8]),
        tool_type: "function".to_string(),
        function: function_call,
    }
}

/// Get current timestamp
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_builder_creation() {
        let builder = ToolCallMessageBuilder::new();
        assert!(builder.current_message.is_none());
        assert!(builder.tool_calls.is_empty());
        assert_eq!(builder.message_count(), 0);
    }

    #[test]
    fn test_user_message_building() {
        let (builder, message) = ToolCallMessageBuilder::new()
            .user_message("Hello, world!".to_string())
            .build_message()
            .unwrap();

        assert_eq!(message.role, "user");
        assert_eq!(message.content, Some("Hello, world!".to_string()));
        assert_eq!(builder.message_count(), 1);
    }

    #[test]
    fn test_assistant_message_with_tool_calls() {
        let tool_call = ToolCall {
            id: "call_123".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "test_function".to_string(),
                arguments: serde_json::to_string(&serde_json::json!({})).unwrap(),
            },
        };

        let (_, message) = ToolCallMessageBuilder::new()
            .assistant_message(Some("I'll call a function".to_string()))
            .with_tool_call(tool_call.clone())
            .build_message()
            .unwrap();

        assert_eq!(message.role, "assistant");
        assert!(message.tool_calls.is_some());
        assert_eq!(message.tool_calls.unwrap().len(), 1);
    }

    #[test]
    fn test_response_formatter_creation() {
        let formatter = ToolCallResponseFormatter::new("test-model".to_string());
        assert_eq!(formatter.model, "test-model");
    }

    #[test]
    fn test_tool_call_response_creation() {
        let formatter = ToolCallResponseFormatter::new("test-model".to_string());

        let tool_call = ToolCall {
            id: "call_123".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "test_function".to_string(),
                arguments: serde_json::to_string(&serde_json::json!({})).unwrap(),
            },
        };

        let response = formatter.create_tool_call_response(
            Some("Calling function".to_string()),
            vec![tool_call],
            None,
        );

        assert_eq!(response.model, "test-model");
        assert_eq!(response.choices.len(), 1);
        assert!(response.choices[0].message.tool_calls.is_some());
    }

    #[test]
    fn test_tool_result_response_creation() {
        let formatter = ToolCallResponseFormatter::new("test-model".to_string());

        let results = vec![
            ("call_123".to_string(), Ok(serde_json::json!({"result": "success"}))),
            ("call_456".to_string(), Err("Function failed".to_string())),
        ];

        let response = formatter.create_tool_result_response(results, None);

        assert_eq!(response.model, "test-model");
        assert!(response.choices[0].message.content.is_some());
        assert!(response.choices[0].message.content.as_ref().unwrap().contains("Tool execution results"));
    }

    #[test]
    fn test_conversion_functions() {
        let function_call = FunctionCall {
            name: "test_function".to_string(),
            arguments: "{}".to_string(),
        };

        let tool_call = function_call_to_tool_call(function_call.clone());
        assert_eq!(tool_call.function.name, function_call.name);

        let converted_back = tool_call_to_function_call(&tool_call);
        assert_eq!(converted_back.name, function_call.name);
    }
}