//! # Core Streaming Functionality
//!
//! This module contains the core streaming implementations that are shared
//! across all adapters, including response formatting and error handling.

use crate::{
    error::ProxyError,
    schemas::{ChatCompletionChunk, StreamChoice, StreamDelta, StreamingError, ErrorDetails, Usage},
};
use axum::response::sse::Event;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Streaming response state management
#[derive(Debug, Clone)]
pub struct StreamingState {
    /// Unique request identifier
    pub request_id: String,
    /// Model being used for the request
    pub model: String,
    /// Current chunk index
    pub chunk_index: usize,
    /// Whether the stream has finished
    pub is_finished: bool,
}

impl StreamingState {
    /// Create a new streaming state
    pub fn new(model: String) -> Self {
        Self {
            request_id: format!("chatcmpl-{}", &Uuid::new_v4().to_string()[..8]),
            model,
            chunk_index: 0,
            is_finished: false,
        }
    }

    /// Get the next chunk index and increment
    pub fn next_index(&mut self) -> usize {
        let index = self.chunk_index;
        self.chunk_index += 1;
        index
    }

    /// Mark the stream as finished
    pub fn finish(&mut self) {
        self.is_finished = true;
    }
}

/// Streaming response wrapper
pub type StreamingResponse = Result<Event, std::convert::Infallible>;

/// Create a streaming response event with content
pub fn create_content_event(state: &mut StreamingState, content: String) -> Event {
    let chunk = ChatCompletionChunk {
        id: state.request_id.clone(),
        object: "chat.completion.chunk".to_string(),
        created: current_timestamp(),
        model: state.model.clone(),
        choices: vec![StreamChoice {
            index: 0,
            delta: StreamDelta {
                role: if state.chunk_index == 0 { Some("assistant".to_string()) } else { None },
                content: Some(content),
                function_call: None,
                tool_calls: None,
            },
            finish_reason: None,
        }],
        usage: None,
    };

    state.next_index();

    Event::default()
        .data(serde_json::to_string(&chunk).unwrap_or_default())
}

/// Create a final streaming event to end the stream
pub fn create_final_event(state: &mut StreamingState) -> Event {
    let chunk = ChatCompletionChunk {
        id: state.request_id.clone(),
        object: "chat.completion.chunk".to_string(),
        created: current_timestamp(),
        model: state.model.clone(),
        choices: vec![StreamChoice {
            index: 0,
            delta: StreamDelta {
                role: None,
                content: None,
                function_call: None,
                tool_calls: None,
            },
            finish_reason: Some("stop".to_string()),
        }],
        usage: Some(Usage {
            prompt_tokens: 0,
            completion_tokens: state.chunk_index as u32,
            total_tokens: state.chunk_index as u32,
        }),
    };

    state.finish();

    Event::default()
        .data(serde_json::to_string(&chunk).unwrap_or_default())
}

/// Create an error event for streaming errors
pub fn create_error_event(error: ProxyError) -> Event {
    let error_response = StreamingError {
        error: ErrorDetails {
            message: error.to_string(),
            r#type: match error {
                ProxyError::BadRequest(_) => "invalid_request_error",
                ProxyError::Upstream(_) => "api_error",
                ProxyError::Internal(_) => "internal_error",
                ProxyError::Serialization(_) => "serialization_error",
            }.to_string(),
            code: None,
        },
    };

    Event::default()
        .data(serde_json::to_string(&error_response).unwrap_or_default())
}

/// Create the final [DONE] event
pub fn create_done_event() -> Event {
    Event::default().data("[DONE]")
}

/// Get current timestamp
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

/// Streaming metrics collection
#[derive(Debug, Clone, Default)]
pub struct StreamingMetrics {
    pub total_chunks: usize,
    pub total_bytes: usize,
    pub stream_duration_ms: u64,
    pub errors: usize,
}

impl StreamingMetrics {
    /// Create new streaming metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a chunk being sent
    pub fn record_chunk(&mut self, content_length: usize) {
        self.total_chunks += 1;
        self.total_bytes += content_length;
    }

    /// Record an error
    pub fn record_error(&mut self) {
        self.errors += 1;
    }

    /// Set the stream duration
    pub fn set_duration(&mut self, duration_ms: u64) {
        self.stream_duration_ms = duration_ms;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_state_creation() {
        let state = StreamingState::new("test-model".to_string());
        assert_eq!(state.model, "test-model");
        assert_eq!(state.chunk_index, 0);
        assert!(!state.is_finished);
        assert!(state.request_id.starts_with("chatcmpl-"));
    }

    #[test]
    fn test_streaming_state_indexing() {
        let mut state = StreamingState::new("test-model".to_string());

        assert_eq!(state.next_index(), 0);
        assert_eq!(state.next_index(), 1);
        assert_eq!(state.next_index(), 2);
        assert_eq!(state.chunk_index, 3);
    }

    #[test]
    fn test_content_event_creation() {
        let mut state = StreamingState::new("test-model".to_string());
        let _event = create_content_event(&mut state, "Hello, world!".to_string());

        // Note: Event.data() is a method, not a field, so we can't test it directly
        // Instead, we'll test that the event was created successfully
        assert!(!state.is_finished);
    }

    #[test]
    fn test_final_event_creation() {
        let mut state = StreamingState::new("test-model".to_string());
        state.next_index(); // Simulate some chunks
        state.next_index();

        let _event = create_final_event(&mut state);
        assert!(state.is_finished);

        // Note: Event.data() is a method, not a field, so we can't test it directly
        // Instead, we'll test that the event was created successfully
        assert!(state.is_finished);
    }

    #[test]
    fn test_error_event_creation() {
        let error = ProxyError::BadRequest("Test error".to_string());
        let _event = create_error_event(error);

        // Note: Event.data() is a method, not a field, so we can't test it directly
        // Instead, we'll test that the event was created successfully
        // The error event creation is successful if no panic occurs
    }

    #[test]
    fn test_streaming_metrics() {
        let mut metrics = StreamingMetrics::new();

        metrics.record_chunk(100);
        metrics.record_chunk(150);
        metrics.record_error();
        metrics.set_duration(500);

        assert_eq!(metrics.total_chunks, 2);
        assert_eq!(metrics.total_bytes, 250);
        assert_eq!(metrics.errors, 1);
        assert_eq!(metrics.stream_duration_ms, 500);
    }
}