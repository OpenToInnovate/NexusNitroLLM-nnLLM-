//! # Tool Call Streaming Support
//!
//! This module provides streaming support for tool calls,
//! integrating with the main streaming infrastructure.

use crate::{
    schemas::{ToolCall, ChatCompletionChunk, StreamChoice, StreamDelta, StreamToolCall, StreamFunctionCall},
    streaming::core::{StreamingState, StreamingResponse},
    error::ProxyError,
};
use axum::response::sse::Event;
use serde_json::json;
use super::{ToolCallHistoryEntry, executor::ToolCallExecutor};

/// Tool call stream processor for handling streaming tool calls
pub struct ToolCallStreamProcessor {
    /// Tool call executor for processing calls
    executor: Option<ToolCallExecutor>,
    /// Current streaming state
    state: Option<StreamingState>,
    /// Buffer for partial tool calls
    buffer: String,
}

impl ToolCallStreamProcessor {
    /// Create a new tool call stream processor
    pub fn new() -> Self {
        Self {
            executor: None,
            state: None,
            buffer: String::new(),
        }
    }

    /// Set the tool call executor
    pub fn with_executor(mut self, executor: ToolCallExecutor) -> Self {
        self.executor = Some(executor);
        self
    }

    /// Initialize streaming state
    pub fn init_streaming(&mut self, model: String) {
        self.state = Some(StreamingState::new(model));
    }

    /// Process a streaming chunk for tool calls
    pub fn process_chunk(&mut self, chunk: &str) -> Result<Option<Event>, ProxyError> {
        // Add chunk to buffer
        self.buffer.push_str(chunk);

        // Try to parse complete tool calls from buffer
        if let Some(tool_call) = self.try_parse_tool_call()? {
            return self.create_tool_call_event(tool_call);
        }

        Ok(None)
    }

    /// Process a complete tool call and create streaming response
    pub async fn process_tool_call(
        &mut self,
        tool_call: ToolCall,
    ) -> Result<Vec<StreamingResponse>, ProxyError> {
        let mut responses = Vec::new();

        // Create tool call start event
        responses.push(Ok(self.create_tool_call_start_event(&tool_call)?));

        // Execute tool call if executor is available
        if let Some(ref mut executor) = self.executor {
            match executor.execute_tool_call(tool_call.clone()).await {
                Ok(result) => {
                    // Create success event
                    responses.push(Ok(self.create_tool_call_result_event(&tool_call, result)?));
                }
                Err(error) => {
                    // Create error event
                    responses.push(Ok(self.create_tool_call_error_event(&tool_call, error)?));
                }
            }
        }

        // Create tool call end event
        responses.push(Ok(self.create_tool_call_end_event(&tool_call)?));

        Ok(responses)
    }

    /// Create streaming events for tool call history
    pub fn create_history_events(&self, history: &[ToolCallHistoryEntry]) -> Vec<StreamingResponse> {
        history
            .iter()
            .map(|entry| Ok(self.create_history_event(entry)))
            .collect()
    }

    /// Flush any remaining buffer content
    pub fn flush(&mut self) -> Result<Option<Event>, ProxyError> {
        if !self.buffer.is_empty() {
            let content = self.buffer.clone();
            self.buffer.clear();

            if self.state.is_some() {
                // Create the event using the content we already have
                let state = self.state.as_mut().unwrap();
                return Ok(Some(Self::create_content_event_static(state.model.clone(), state, content)));
            }
        }

        Ok(None)
    }

    /// Try to parse a complete tool call from the buffer
    fn try_parse_tool_call(&mut self) -> Result<Option<ToolCall>, ProxyError> {
        // Look for tool call markers or complete JSON structures
        if let Some(start) = self.buffer.find("{\"tool_call\"") {
            if let Some(end) = self.buffer[start..].find("}\n") {
                let tool_call_str = &self.buffer[start..start + end + 1];

                match serde_json::from_str::<ToolCall>(tool_call_str) {
                    Ok(tool_call) => {
                        // Remove parsed content from buffer
                        self.buffer.drain(..start + end + 2);
                        return Ok(Some(tool_call));
                    }
                    Err(_) => {
                        // Parsing failed, might be incomplete - keep in buffer
                    }
                }
            }
        }

        Ok(None)
    }

    /// Create tool call event from parsed tool call
    fn create_tool_call_event(&mut self, tool_call: ToolCall) -> Result<Option<Event>, ProxyError> {
        let state = self.state.as_mut().ok_or_else(|| {
            ProxyError::Internal("Streaming state not initialized".to_string())
        })?;

        let chunk = ChatCompletionChunk {
            id: state.request_id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            model: state.model.clone(),
            choices: vec![StreamChoice {
                index: 0,
                delta: StreamDelta {
                    role: None,
                    content: None,
                    function_call: None,
                    tool_calls: Some(vec![StreamToolCall {
                        index: 0,
                        id: Some(tool_call.id),
                        tool_type: Some(tool_call.tool_type),
                        function: Some(StreamFunctionCall {
                            name: Some(tool_call.function.name),
                            arguments: Some(tool_call.function.arguments),
                        }),
                    }]),
                },
                finish_reason: None,
            }],
            usage: None,
        };

        state.next_index();

        Ok(Some(Event::default().data(
            serde_json::to_string(&chunk).map_err(|e| {
                ProxyError::Serialization(format!("Failed to serialize tool call chunk: {}", e))
            })?
        )))
    }

    /// Create tool call start event
    fn create_tool_call_start_event(&self, tool_call: &ToolCall) -> Result<Event, ProxyError> {
        let data = json!({
            "type": "tool_call_start",
            "tool_call_id": tool_call.id,
            "function_name": tool_call.function.name
        });

        Ok(Event::default()
            .event("tool_call_start")
            .data(data.to_string()))
    }

    /// Create tool call result event
    fn create_tool_call_result_event(
        &self,
        tool_call: &ToolCall,
        result: serde_json::Value,
    ) -> Result<Event, ProxyError> {
        let data = json!({
            "type": "tool_call_result",
            "tool_call_id": tool_call.id,
            "function_name": tool_call.function.name,
            "result": result
        });

        Ok(Event::default()
            .event("tool_call_result")
            .data(data.to_string()))
    }

    /// Create tool call error event
    fn create_tool_call_error_event(
        &self,
        tool_call: &ToolCall,
        error: crate::tools::ToolError,
    ) -> Result<Event, ProxyError> {
        let data = json!({
            "type": "tool_call_error",
            "tool_call_id": tool_call.id,
            "function_name": tool_call.function.name,
            "error": error.to_string()
        });

        Ok(Event::default()
            .event("tool_call_error")
            .data(data.to_string()))
    }

    /// Create tool call end event
    fn create_tool_call_end_event(&self, tool_call: &ToolCall) -> Result<Event, ProxyError> {
        let data = json!({
            "type": "tool_call_end",
            "tool_call_id": tool_call.id,
            "function_name": tool_call.function.name
        });

        Ok(Event::default()
            .event("tool_call_end")
            .data(data.to_string()))
    }

    /// Create history event for tool call entry
    fn create_history_event(&self, entry: &ToolCallHistoryEntry) -> Event {
        let data = json!({
            "type": "tool_call_history",
            "tool_call_id": entry.tool_call_id,
            "function_name": entry.function_name,
            "arguments": entry.arguments,
            "result": entry.result,
            "error": entry.error,
            "timestamp": entry.timestamp.duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        });

        Event::default()
            .event("tool_call_history")
            .data(data.to_string())
    }

    /// Create content event (static version to avoid borrowing conflicts)
    fn create_content_event_static(_model: String, state: &mut StreamingState, content: String) -> Event {
        let chunk = ChatCompletionChunk {
            id: state.request_id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            model: state.model.clone(),
            choices: vec![StreamChoice {
                index: 0,
                delta: StreamDelta {
                    role: if state.chunk_index == 0 {
                        Some("assistant".to_string())
                    } else {
                        None
                    },
                    content: Some(content),
                    function_call: None,
                    tool_calls: None,
                },
                finish_reason: None,
            }],
            usage: None,
        };

        state.next_index();

        Event::default().data(serde_json::to_string(&chunk).unwrap_or_default())
    }
}

impl Default for ToolCallStreamProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for tool call streaming
pub mod utils {
    use super::*;

    /// Check if a chunk contains tool call data
    pub fn is_tool_call_chunk(chunk: &str) -> bool {
        chunk.contains("tool_call") || chunk.contains("function_call")
    }

    /// Extract tool call ID from chunk
    pub fn extract_tool_call_id(chunk: &str) -> Option<String> {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(chunk) {
            value
                .get("tool_call_id")
                .and_then(|id| id.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Create a tool call delta chunk
    pub fn create_tool_call_delta(
        tool_call_id: String,
        function_name: Option<String>,
        arguments: Option<String>,
    ) -> Result<Event, ProxyError> {
        let data = json!({
            "type": "tool_call_delta",
            "tool_call_id": tool_call_id,
            "function_name": function_name,
            "arguments": arguments
        });

        Ok(Event::default()
            .event("tool_call_delta")
            .data(data.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        schemas::FunctionCall,
        tools::{registry::FunctionRegistry, executor::ToolCallExecutor},
    };

    #[test]
    fn test_tool_call_stream_processor_creation() {
        let processor = ToolCallStreamProcessor::new();
        assert!(processor.executor.is_none());
        assert!(processor.state.is_none());
        assert!(processor.buffer.is_empty());
    }

    #[tokio::test]
    async fn test_tool_call_processing() {
        let registry = FunctionRegistry::new();
        let executor = ToolCallExecutor::new(registry);

        let mut processor = ToolCallStreamProcessor::new().with_executor(executor);
        processor.init_streaming("test-model".to_string());

        let tool_call = ToolCall {
            id: "call_123".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "test_function".to_string(),
                arguments: "{}".to_string(),
            },
        };

        let responses = processor.process_tool_call(tool_call).await.unwrap();
        assert!(!responses.is_empty());
    }

    #[test]
    fn test_chunk_processing() {
        let mut processor = ToolCallStreamProcessor::new();
        processor.init_streaming("test-model".to_string());

        let chunk = "partial content";
        let result = processor.process_chunk(chunk);
        assert!(result.is_ok());
    }

    #[test]
    fn test_buffer_flushing() {
        let mut processor = ToolCallStreamProcessor::new();
        processor.init_streaming("test-model".to_string());
        processor.buffer = "remaining content".to_string();

        let result = processor.flush().unwrap();
        assert!(result.is_some());
        assert!(processor.buffer.is_empty());
    }

    #[test]
    fn test_utility_functions() {
        assert!(utils::is_tool_call_chunk("{\"tool_call\": {}}"));
        assert!(!utils::is_tool_call_chunk("regular content"));

        let chunk = r#"{"tool_call_id": "call_123"}"#;
        assert_eq!(
            utils::extract_tool_call_id(chunk),
            Some("call_123".to_string())
        );
    }
}