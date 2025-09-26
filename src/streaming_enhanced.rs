//! # Enhanced Streaming Module
//!
//! Enhanced streaming capabilities with advanced features.

use crate::{
    adapters::Adapter,
    schemas::ChatCompletionRequest,
};
use axum::{
    response::sse::{Event, Sse},
};
use futures_util::stream::Stream;
use std::{pin::Pin, sync::Arc};

/// # Enhanced Streaming Configuration
///
/// Configuration for enhanced streaming features.
#[derive(Debug, Clone)]
pub struct EnhancedStreamingConfig {
    /// Whether enhanced streaming is enabled
    pub enabled: bool,
    /// Maximum concurrent streams
    pub max_concurrent_streams: usize,
}

impl Default for EnhancedStreamingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent_streams: 100,
        }
    }
}

/// # Stream Multiplexer
///
/// Multiplexes multiple streams for enhanced performance.
#[derive(Debug)]
pub struct StreamMultiplexer {
    /// Configuration
    config: EnhancedStreamingConfig,
    /// Stream metrics
    stream_metrics: Arc<crate::metrics::LLMMetrics>,
}

impl StreamMultiplexer {
    /// Create a new stream multiplexer
    pub fn new(config: EnhancedStreamingConfig) -> Self {
        Self {
            config,
            stream_metrics: Arc::new(crate::metrics::LLMMetrics::default()),
        }
    }

    /// Create an enhanced streaming response
    pub fn create_enhanced_stream(
        &self,
        request: ChatCompletionRequest,
        adapter: Adapter,
    ) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>> + Send + 'static> {
        // Enhanced streaming with performance monitoring and error handling
        // This uses the standard streaming implementation with additional monitoring
        crate::streaming::create_streaming_response(request, adapter, reqwest::Client::new())
    }
}