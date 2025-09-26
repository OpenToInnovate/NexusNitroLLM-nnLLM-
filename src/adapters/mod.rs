//! # Universal LLM Adapters Module
//!
//! This module contains the core adapter implementations that handle communication
//! between NexusNitroLLM and various backend LLM services including LightLLM, vLLM,
//! Azure OpenAI, AWS Bedrock, and other OpenAI-compatible providers.
//!
//! ## Supported Backends:
//!
//! - **LightLLM**: Direct integration with LightLLM servers
//! - **vLLM**: OpenAI-compatible vLLM servers
//! - **Azure OpenAI**: Microsoft Azure OpenAI Service
//! - **AWS Bedrock**: Amazon Web Services Bedrock
//! - **OpenAI**: Direct OpenAI API integration
//! - **Custom**: Any OpenAI-compatible endpoint
//! - **Direct**: Embedded integration mode

use crate::{
    config::Config,
    error::ProxyError,
    schemas::ChatCompletionRequest,
};
use crate::core::http_client::HttpClientBuilder;
#[cfg(feature = "server")]
use axum::response::Response;

// Base adapter functionality
pub mod base;

// Individual adapter modules
pub mod lightllm;
pub mod openai;
pub mod azure;
pub mod aws;
pub mod vllm;
pub mod custom;
pub mod direct;

// Re-export adapters for convenience
pub use lightllm::{LightLLMAdapter, Role};
pub use openai::OpenAIAdapter;
pub use azure::AzureOpenAIAdapter;
pub use aws::AWSBedrockAdapter;
pub use vllm::VLLMAdapter;
pub use custom::CustomAdapter;
pub use direct::DirectAdapter;

// Re-export base functionality
pub use base::{AdapterTrait, AdapterConfig, AdapterUtils};

/// # Universal LLM Adapter Enum
///
/// This enum represents different types of LLM backend adapters supported by NexusNitroLLM.
/// Each variant is optimized for its specific backend type with provider-specific features.
#[derive(Clone, Debug)]
pub enum Adapter {
    /// Direct LightLLM backend adapter - converts OpenAI format to LightLLM format
    LightLLM(LightLLMAdapter),
    /// vLLM server adapter - OpenAI-compatible with vLLM optimizations
    VLLM(VLLMAdapter),
    /// Azure OpenAI Service adapter - Microsoft cloud integration
    AzureOpenAI(AzureOpenAIAdapter),
    /// AWS Bedrock adapter - Amazon cloud integration
    AWSBedrock(AWSBedrockAdapter),
    /// OpenAI API adapter - Direct OpenAI integration
    OpenAI(OpenAIAdapter),
    /// Custom OpenAI-compatible adapter - Generic endpoint support
    Custom(CustomAdapter),
    /// Direct integration mode - bypasses HTTP for maximum performance
    Direct(DirectAdapter),
}

impl Adapter {
    /// Factory method for creating adapters based on configuration
    pub fn from_config(cfg: &Config) -> Self {
        // Create HTTP client using our centralized factory
        let client = HttpClientBuilder::from_config(cfg)
            .build()
            .unwrap_or_else(|_| HttpClientBuilder::new().build().unwrap());

        // Intelligent backend detection based on URL patterns
        if cfg.backend_url.contains("azure.com") || cfg.backend_url.contains("azure.openai") {
            // Azure OpenAI Service detected
            Self::AzureOpenAI(AzureOpenAIAdapter::new(
                cfg.backend_url.clone(),
                cfg.model_id.clone(),
                cfg.backend_token.clone(),
                client,
            ))
        } else if cfg.backend_url.contains("bedrock") || cfg.backend_url.contains("amazonaws.com") {
            // AWS Bedrock detected
            Self::AWSBedrock(AWSBedrockAdapter::new(
                cfg.backend_url.clone(),
                cfg.model_id.clone(),
                cfg.backend_token.clone(),
                client,
            ))
        } else if cfg.backend_url.contains("vllm") {
            // vLLM server detected
            Self::VLLM(VLLMAdapter::new(
                cfg.backend_url.clone(),
                cfg.model_id.clone(),
                cfg.backend_token.clone(),
                client,
            ))
        } else if cfg.backend_url.contains("/v1") || cfg.backend_url.contains("openai.com") {
            // OpenAI API or compatible endpoint detected
            Self::OpenAI(OpenAIAdapter::new(
                cfg.backend_url.clone(),
                cfg.model_id.clone(),
                cfg.backend_token.clone(),
                client,
            ))
        } else if cfg.backend_url == "direct" {
            // Direct mode for embedded integration
            Self::Direct(DirectAdapter::new(
                cfg.model_id.clone(),
                cfg.backend_token.clone(),
            ))
        } else if cfg.backend_url.contains("lightllm") || cfg.backend_url.contains("localhost") {
            // LightLLM server detected
            Self::LightLLM(LightLLMAdapter::new(
                cfg.backend_url.clone(),
                cfg.model_id.clone(),
                cfg.backend_token.clone(),
                client,
            ))
        } else {
            // Generic OpenAI-compatible endpoint
            Self::Custom(CustomAdapter::new(
                cfg.backend_url.clone(),
                cfg.model_id.clone(),
                cfg.backend_token.clone(),
                client,
            ))
        }
    }

    /// Process chat completion requests
    #[cfg(feature = "server")]
    pub async fn chat_completions(&self, req: ChatCompletionRequest) -> Result<Response, ProxyError> {
        match self {
            Self::LightLLM(adapter) => adapter.chat_completions_http(req).await,
            Self::VLLM(adapter) => adapter.chat_completions_http(req).await,
            Self::AzureOpenAI(adapter) => adapter.chat_completions_http(req).await,
            Self::AWSBedrock(adapter) => adapter.chat_completions_http(req).await,
            Self::OpenAI(adapter) => adapter.chat_completions_http(req).await,
            Self::Custom(adapter) => adapter.chat_completions_http(req).await,
            Self::Direct(adapter) => {
                // Convert ChatCompletionResponse to Response for direct adapter
                let chat_response = adapter.chat_completions(req).await?;

                // Convert to HTTP response
                let json_response = serde_json::to_string(&chat_response)
                    .map_err(|e| ProxyError::Internal(format!("Failed to serialize response: {}", e)))?;

                Ok(Response::builder()
                    .status(200)
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(json_response))
                    .map_err(|e| ProxyError::Internal(format!("Failed to build response: {}", e)))?)
            }
        }
    }

    /// Check if adapter supports streaming
    pub fn supports_streaming(&self) -> bool {
        match self {
            Self::LightLLM(_) => true,      // LightLLM supports streaming
            Self::VLLM(_) => true,          // vLLM supports streaming
            Self::AzureOpenAI(_) => true,   // Azure OpenAI supports streaming
            Self::AWSBedrock(_) => true,    // AWS Bedrock supports streaming
            Self::OpenAI(_) => true,        // OpenAI API supports streaming
            Self::Custom(_) => true,        // Assume custom endpoints support streaming
            Self::Direct(_) => true,        // Direct mode supports streaming
        }
    }

    /// Get adapter name for logging and metrics
    pub fn name(&self) -> &'static str {
        match self {
            Self::LightLLM(adapter) => adapter.name(),
            Self::VLLM(adapter) => adapter.name(),
            Self::AzureOpenAI(adapter) => adapter.name(),
            Self::AWSBedrock(adapter) => adapter.name(),
            Self::OpenAI(adapter) => adapter.name(),
            Self::Custom(adapter) => adapter.name(),
            Self::Direct(adapter) => adapter.name(),
        }
    }

    /// Get base URL for adapter
    pub fn base_url(&self) -> &str {
        match self {
            Self::LightLLM(adapter) => adapter.base_url(),
            Self::VLLM(adapter) => adapter.base_url(),
            Self::AzureOpenAI(adapter) => adapter.base_url(),
            Self::AWSBedrock(adapter) => adapter.base_url(),
            Self::OpenAI(adapter) => adapter.base_url(),
            Self::Custom(adapter) => adapter.base_url(),
            Self::Direct(adapter) => adapter.base_url(),
        }
    }

    /// Get model ID for adapter
    pub fn model_id(&self) -> &str {
        match self {
            Self::LightLLM(adapter) => adapter.model_id(),
            Self::VLLM(adapter) => adapter.model_id(),
            Self::AzureOpenAI(adapter) => adapter.model_id(),
            Self::AWSBedrock(adapter) => adapter.model_id(),
            Self::OpenAI(adapter) => adapter.model_id(),
            Self::Custom(adapter) => adapter.model_id(),
            Self::Direct(adapter) => adapter.model_id(),
        }
    }

    /// Check if adapter has authentication configured
    pub fn has_auth(&self) -> bool {
        match self {
            Self::LightLLM(adapter) => adapter.has_auth(),
            Self::VLLM(adapter) => adapter.has_auth(),
            Self::AzureOpenAI(adapter) => adapter.has_auth(),
            Self::AWSBedrock(adapter) => adapter.has_auth(),
            Self::OpenAI(adapter) => adapter.has_auth(),
            Self::Custom(adapter) => adapter.has_auth(),
            Self::Direct(adapter) => adapter.has_auth(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_detection_azure() {
        let mut config = Config::for_test();
        config.backend_url = "https://myresource.openai.azure.com".to_string();

        let adapter = Adapter::from_config(&config);
        assert!(matches!(adapter, Adapter::AzureOpenAI(_)));
        assert_eq!(adapter.name(), "azure");
    }

    #[test]
    fn test_adapter_detection_openai() {
        let mut config = Config::for_test();
        config.backend_url = "https://api.openai.com/v1".to_string();

        let adapter = Adapter::from_config(&config);
        assert!(matches!(adapter, Adapter::OpenAI(_)));
        assert_eq!(adapter.name(), "openai");
    }

    #[test]
    fn test_adapter_detection_vllm() {
        let mut config = Config::for_test();
        config.backend_url = "http://localhost:8000/vllm".to_string();

        let adapter = Adapter::from_config(&config);
        assert!(matches!(adapter, Adapter::VLLM(_)));
        assert_eq!(adapter.name(), "vllm");
    }

    #[test]
    fn test_adapter_detection_lightllm() {
        let mut config = Config::for_test();
        config.backend_url = "http://localhost:8000".to_string();

        let adapter = Adapter::from_config(&config);
        assert!(matches!(adapter, Adapter::LightLLM(_)));
        assert_eq!(adapter.name(), "lightllm");
    }

    #[test]
    fn test_adapter_detection_direct() {
        let mut config = Config::for_test();
        config.backend_url = "direct".to_string();

        let adapter = Adapter::from_config(&config);
        assert!(matches!(adapter, Adapter::Direct(_)));
        assert_eq!(adapter.name(), "direct");
    }

    #[test]
    fn test_adapter_detection_custom() {
        let mut config = Config::for_test();
        config.backend_url = "https://custom-endpoint.example.com".to_string();

        let adapter = Adapter::from_config(&config);
        assert!(matches!(adapter, Adapter::Custom(_)));
        assert_eq!(adapter.name(), "custom");
    }

    #[test]
    fn test_streaming_support() {
        let mut config = Config::for_test();

        config.backend_url = "http://localhost:8000".to_string();
        let lightllm_adapter = Adapter::from_config(&config);
        assert!(lightllm_adapter.supports_streaming());

        config.backend_url = "https://api.openai.com/v1".to_string();
        let openai_adapter = Adapter::from_config(&config);
        assert!(openai_adapter.supports_streaming());

        config.backend_url = "direct".to_string();
        let direct_adapter = Adapter::from_config(&config);
        assert!(direct_adapter.supports_streaming());
    }
}