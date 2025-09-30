//! # NexusNitroLLM (nnLLM) - Universal LLM Integration Library
//!
//! A high-performance Rust library for seamless LLM integration with OpenAI-compatible APIs.
//! NexusNitroLLM provides a unified interface to multiple LLM backends including LightLLM, vLLM, 
//! and cloud providers (Azure OpenAI, AWS Bedrock) with enterprise-grade performance and reliability.
//!
//! ## Features
//!
//! - **Universal LLM Support**: LightLLM, vLLM, Azure OpenAI, AWS Bedrock, and more
//! - **OpenAI-Compatible API**: Seamless integration with existing OpenAI clients
//! - **Enterprise Performance**: Connection pooling, HTTP/2, compression, zero-copy operations
//! - **Memory Safe**: Rust's ownership system eliminates memory leaks and buffer overflows
//! - **Production Ready**: Comprehensive error handling, logging, CORS, and testing
//! - **Multi-Provider Authentication**: Flexible token support for various backend services
//! - **Advanced Features**: Rate limiting, caching, batching, metrics collection, load balancing
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use nexus_nitro_llm::{Config, AppState, create_router};
//! use axum::Server;
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create configuration for LightLLM backend
//!     let config = Config::for_test(); // or Config::parse_args() for CLI
//!
//!     // Create application state
//!     let state = AppState::new(config).await;
//!
//!     // Create router
//!     let app = create_router(state);
//!
//!     // Start server
//!     let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
//!     println!("NexusNitroLLM server listening on {}", addr);
//!     axum_server::bind(addr).serve(app.into_make_service()).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The library is organized into several key modules:
//!
//! - [`config`] - Configuration management with CLI and environment support
//! - [`adapters`] - Universal backend adapters for LightLLM, vLLM, Azure, AWS, and more
//! - [`routes`] - HTTP route handlers and application state
//! - [`schemas`] - Request/response data structures
//! - [`error`] - Custom error types and handling
//! - [`streaming`] - Streaming response support
//! - [`metrics`] - Performance metrics collection
//! - [`caching`] - Response caching system
//! - [`rate_limiting`] - Rate limiting and throttling
//! - [`batching`] - Request batching for efficiency

// Core infrastructure
pub mod core;
pub mod client;
pub mod config;
pub mod error;
pub mod schemas;
pub mod graceful_shutdown;

// API format compatibility layers
pub mod anthropic;

// Domain modules
pub mod adapters;

#[cfg(feature = "tools")]
pub mod tools;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "streaming")]
pub mod streaming;

// Feature-gated modules
#[cfg(feature = "batching")]
pub mod batching;

#[cfg(feature = "metrics")]
pub mod metrics;

#[cfg(feature = "rate-limiting")]
pub mod rate_limiting;

#[cfg(feature = "distributed-rate-limiting")]
pub mod distributed_rate_limiting;

#[cfg(feature = "caching")]
pub mod caching;

// Legacy route module for compatibility
#[cfg(feature = "server")]
pub mod routes;

// Python bindings (feature-gated)
#[cfg(feature = "python")]
pub mod python;

#[cfg(feature = "nodejs")]
pub mod nodejs;

// Re-export commonly used types for convenience
pub use config::Config;
pub use error::ProxyError;
pub use adapters::{Adapter, LightLLMAdapter, OpenAIAdapter};
pub use schemas::{ChatCompletionRequest, Message, Tool, ToolChoice, FunctionCall, ToolCall};
pub use core::http_client::{HttpClientBuilder, HttpClientConfig};
pub use graceful_shutdown::{GracefulShutdown, ServerLifecycle, ShutdownConfig, setup_shutdown_handler};

// Tool support re-exports
#[cfg(feature = "tools")]
pub use tools::{
    ToolError, ToolRole, ToolUseMessage, ToolCallHistoryEntry,
    FunctionRegistry, FunctionDefinition,
    ToolCallExecutor, FunctionExecutor,
    ToolCallValidator,
    ToolCallStreamProcessor,
    ToolCallMessageBuilder, ToolCallResponseFormatter
};

// Server re-exports (feature-gated)
#[cfg(feature = "server")]
pub use server::{AppState, create_router};

#[cfg(feature = "server")]
pub use server::handlers::chat_completions;

// Streaming re-exports
#[cfg(feature = "streaming")]
pub use streaming::{StreamingHandler, create_streaming_response};

// Enhanced features re-exports (feature-gated)
#[cfg(feature = "metrics")]
pub use metrics::{LLMMetrics, MetricsCollector};

#[cfg(feature = "caching")]
pub use caching::{CacheManager, CacheConfig, CacheStats};

#[cfg(feature = "rate-limiting")]
pub use rate_limiting::{AdvancedRateLimiter, RateLimitRequest, RateLimitResult};

#[cfg(feature = "batching")]
pub use batching::{BatchProcessor, BatchRequest, BatchStats};

/// The result type used throughout the library
pub type Result<T> = std::result::Result<T, ProxyError>;