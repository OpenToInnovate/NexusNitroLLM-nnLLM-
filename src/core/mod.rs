//! # Core Infrastructure Module
//!
//! This module contains the foundational components and infrastructure
//! shared across the NexusNitroLLM library, including configuration,
//! error handling, HTTP client management, and common utilities.

pub mod http_client;

// Re-export commonly used core types
pub use http_client::{HttpClientBuilder, HttpClientConfig, HttpClientError};