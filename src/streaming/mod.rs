//! # Streaming Module
//!
//! This module implements Server-Sent Events (SSE) streaming for chat completions,
//! providing OpenAI-compatible streaming responses with proper chunk formatting.
//!
//! ## Key Features:
//! - OpenAI-compatible SSE streaming
//! - Adapter-specific streaming implementations
//! - Zero-copy streaming with backpressure handling
//! - Connection pooling and compression support

pub mod core;
pub mod adapters;

// Re-export commonly used streaming types
pub use core::{
    StreamingState, StreamingResponse,
    create_error_event, StreamingMetrics
};
pub use adapters::{StreamingAdapter, StreamingHandler};

// Re-export from core streaming functionality
use crate::{
    adapters::Adapter,
    error::ProxyError,
    schemas::ChatCompletionRequest,
};

/// Create a streaming response for the given adapter and request
pub async fn create_streaming_response(
    adapter: &Adapter,
    request: ChatCompletionRequest,
) -> Result<adapters::StreamingResponse, ProxyError> {
    if !adapter.supports_streaming() {
        return Err(ProxyError::BadRequest(
            format!("Adapter {} does not support streaming", adapter.name())
        ));
    }

    // Delegate to adapter-specific streaming implementation
    match adapter {
        crate::adapters::Adapter::LightLLM(adapter) => {
            adapters::lightllm_streaming(adapter, request).await
        },
        crate::adapters::Adapter::OpenAI(adapter) => {
            adapters::openai_streaming(adapter, request).await
        },
        crate::adapters::Adapter::VLLM(adapter) => {
            adapters::vllm_streaming(adapter, request).await
        },
        crate::adapters::Adapter::AzureOpenAI(adapter) => {
            adapters::azure_streaming(adapter, request).await
        },
        crate::adapters::Adapter::Custom(adapter) => {
            adapters::custom_streaming(adapter, request).await
        },
        _ => Err(ProxyError::BadRequest("Streaming not supported for this adapter".to_string())),
    }
}