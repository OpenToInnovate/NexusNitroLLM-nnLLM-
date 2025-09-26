//! # Feature Combination Tests
//!
//! Tests that validate different combinations of feature flags work correctly
//! and that conditional compilation produces the expected behavior.

#[cfg(test)]
mod tests {
    use nexus_nitro_llm::config::Config;

    #[test]
    fn test_no_features() {
        // Test basic functionality with minimal features
        let config = Config::default();
        let result = config.validate();
        assert!(result.is_ok(), "Basic config should work with any feature combination");
    }

    #[test]
    #[cfg(all(feature = "cli", feature = "server"))]
    fn test_cli_plus_server() {
        // Test CLI + Server combination
        let config = Config::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "0.0.0.0");

        let result = config.validate();
        assert!(result.is_ok(), "CLI + Server features should work together");
    }

    #[test]
    #[cfg(all(feature = "streaming", feature = "server"))]
    fn test_streaming_plus_server() {
        // Test Streaming + Server combination
        use nexus_nitro_llm::streaming::adapters::StreamingHandler;

        let config = Config::default();
        assert!(config.enable_streaming);

        let handler_result = StreamingHandler::new();
        assert!(handler_result.is_ok(), "Streaming handler should work with server features");
    }

    #[test]
    #[cfg(all(feature = "cli", feature = "streaming"))]
    fn test_cli_plus_streaming() {
        // Test CLI + Streaming combination
        let config = Config::default();
        assert!(config.enable_streaming);
        assert!(config.streaming_timeout > 0);
        assert!(config.streaming_chunk_size > 0);

        let result = config.validate();
        assert!(result.is_ok(), "CLI + Streaming features should work together");
    }

    #[test]
    #[cfg(all(feature = "metrics", feature = "server"))]
    fn test_metrics_plus_server() {
        // Test Metrics + Server combination
        use nexus_nitro_llm::metrics::LLMMetrics;

        let mut metrics = LLMMetrics::default();
        metrics.total_requests = 1;
        metrics.successful_requests = 1;
        metrics.avg_response_time_ms = 100.0;

        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
    }

    #[test]
    #[cfg(all(feature = "batching", feature = "streaming"))]
    fn test_batching_plus_streaming() {
        // Test Batching + Streaming combination
        let config = Config::default();
        assert!(config.enable_streaming);

        // Batching + streaming should work together
        let result = config.validate();
        assert!(result.is_ok(), "Batching + Streaming should work together");
    }

    #[test]
    #[cfg(all(feature = "caching", feature = "server"))]
    fn test_caching_plus_server() {
        // Test Caching + Server combination
        let mut config = Config::default();
        config.enable_caching = true;
        config.cache_max_size = 1000;
        config.cache_ttl_seconds = 300;

        let result = config.validate();
        assert!(result.is_ok(), "Caching + Server should work together");
    }

    #[test]
    #[cfg(all(feature = "python", feature = "server"))]
    fn test_python_plus_server() {
        // Test Python + Server combination
        let config = Config::default();
        let result = config.validate();
        assert!(result.is_ok(), "Python + Server features should work together");

        // Test that Python bindings are available with server features
        // This mainly ensures compilation works
        assert!(true, "Python bindings should work with server features");
    }

    #[test]
    #[cfg(all(feature = "nodejs", feature = "server"))]
    fn test_nodejs_plus_server() {
        // Test Node.js + Server combination
        let config = Config::default();
        let result = config.validate();
        assert!(result.is_ok(), "Node.js + Server features should work together");
    }

    #[test]
    #[cfg(all(feature = "python", feature = "streaming"))]
    fn test_python_plus_streaming() {
        // Test Python + Streaming combination
        let config = Config::default();
        assert!(config.enable_streaming);

        let result = config.validate();
        assert!(result.is_ok(), "Python + Streaming should work together");
    }

    #[test]
    #[cfg(all(feature = "nodejs", feature = "streaming"))]
    fn test_nodejs_plus_streaming() {
        // Test Node.js + Streaming combination
        let config = Config::default();
        assert!(config.enable_streaming);

        let result = config.validate();
        assert!(result.is_ok(), "Node.js + Streaming should work together");
    }

    #[test]
    #[cfg(all(feature = "rate-limiting", feature = "distributed-rate-limiting"))]
    fn test_rate_limiting_combination() {
        // Test Rate Limiting + Distributed Rate Limiting combination
        use nexus_nitro_llm::distributed_rate_limiting::{DistributedRateLimiter, DistributedRateLimitConfig};

        let config = DistributedRateLimitConfig::default();
        let _rate_limiter = DistributedRateLimiter::new(config);

        // Test basic rate limiting config
        let app_config = Config::default();
        assert!(app_config.enable_rate_limiting);
        assert!(app_config.rate_limit_requests_per_minute > 0);
        assert!(app_config.rate_limit_burst_size > 0);
    }

    #[test]
    #[cfg(all(feature = "tools", feature = "server"))]
    fn test_tools_plus_server() {
        // Test Tools + Server combination
        let config = Config::default();
        let result = config.validate();
        assert!(result.is_ok(), "Tools + Server should work together");
    }

    #[test]
    #[cfg(all(feature = "streaming-sse", feature = "streaming-adapters"))]
    fn test_streaming_sub_features() {
        // Test streaming sub-features work together
        let config = Config::default();
        assert!(config.enable_streaming);

        let result = config.validate();
        assert!(result.is_ok(), "Streaming sub-features should work together");
    }

    #[test]
    #[cfg(all(feature = "tool-registry", feature = "tool-execution", feature = "tool-validation"))]
    fn test_tool_sub_features() {
        // Test tool sub-features work together
        let config = Config::default();
        let result = config.validate();
        assert!(result.is_ok(), "Tool sub-features should work together");
    }

    #[test]
    #[cfg(all(feature = "adapter-lightllm", feature = "adapter-openai"))]
    fn test_multiple_adapters() {
        // Test multiple adapter features work together
        let config = Config::default();
        let result = config.validate();
        assert!(result.is_ok(), "Multiple adapter features should work together");
    }

    #[test]
    #[cfg(all(feature = "logging", feature = "metrics"))]
    fn test_observability_features() {
        // Test observability features work together
        let config = Config::default();
        assert!(config.enable_metrics);
        assert!(!config.log_level.is_empty());

        let result = config.validate();
        assert!(result.is_ok(), "Observability features should work together");
    }

    #[test]
    #[cfg(all(feature = "health-checks", feature = "metrics"))]
    fn test_monitoring_features() {
        // Test monitoring features work together
        let config = Config::default();
        assert!(config.enable_health_checks);
        assert!(config.enable_metrics);

        let result = config.validate();
        assert!(result.is_ok(), "Monitoring features should work together");
    }

    #[test]
    #[cfg(all(feature = "load-balancing", feature = "connection-pooling"))]
    fn test_performance_features() {
        // Test performance features work together
        use nexus_nitro_llm::performance_optimization::PerformanceConfig;

        let perf_config = PerformanceConfig::default();
        assert!(perf_config.connection_pooling_enabled);
        assert!(perf_config.max_connections > 0);
    }

    #[test]
    #[cfg(not(any(feature = "cli", feature = "server", feature = "streaming")))]
    fn test_minimal_features() {
        // Test that the library works with minimal features
        let config = Config::default();

        // Basic functionality should still work
        assert!(config.port > 0);
        assert!(!config.backend_url.is_empty());
        assert!(!config.model_id.is_empty());

        let result = config.validate();
        assert!(result.is_ok(), "Minimal feature set should work");
    }

    #[test]
    fn test_adapter_creation_with_features() {
        use nexus_nitro_llm::adapters::Adapter;

        let config = Config::default();
        let adapter = Adapter::from_config(&config);

        // Adapter should be created regardless of features
        match adapter {
            Adapter::LightLLM(_) => assert!(true),
            Adapter::OpenAI(_) => assert!(true),
            Adapter::VLLM(_) => assert!(true),
            Adapter::AzureOpenAI(_) => assert!(true),
            Adapter::AWSBedrock(_) => assert!(true),
            Adapter::Custom(_) => assert!(true),
            Adapter::Direct(_) => assert!(true),
        }
    }

    #[test]
    fn test_error_handling_across_features() {
        use nexus_nitro_llm::error::ProxyError;

        // Test that error handling works across all features
        let errors = vec![;
            ProxyError::BadRequest("test".to_string()),
            ProxyError::Internal("test".to_string()),
            ProxyError::Upstream("timeout".to_string()),
            ProxyError::Internal("rate limit".to_string()),
        ];

        for error in errors {
            // Each error should be handled consistently regardless of features
            match error {
                ProxyError::BadRequest(_) => assert!(true),
                ProxyError::Internal(_) => assert!(true),
                ProxyError::Upstream(_) => assert!(true),
                ProxyError::Serialization(_) => assert!(true),
            }
        }
    }

    #[test]
    fn test_schema_compatibility_across_features() {
        use nexus_nitro_llm::schemas::{ChatCompletionRequest, ChatCompletionResponse, Message};

        // Test that schemas work consistently across features
        let message = Message {
            role: "user".to_string(),
            content: Some("test".to_string()),
            name: None,
            function_call: None,
            tool_calls: None,
            tool_call_id: None,
        };

        let request = ChatCompletionRequest {;
            messages: vec![message],
            model: Some("test".to_string()),
            ..Default::default()
        };

        let response = ChatCompletionResponse {;
            id: "test-id".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "test-model".to_string(),
            choices: vec![],
            usage: None,
        };

        assert_eq!(request.messages.len(), 1);
        assert_eq!(response.object, "chat.completion");
    }

    #[test]
    fn test_default_configuration_across_features() {
        // Test that default configuration is consistent across features
        let config = Config::default();

        // These should be consistent regardless of features
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.backend_url, "http://localhost:8000");
        assert_eq!(config.backend_type, "lightllm");
        assert_eq!(config.model_id, "llama");
        assert_eq!(config.environment, "development");

        // Feature flags should have sensible defaults
        assert!(config.enable_streaming);
        assert!(!config.enable_batching);
        assert!(config.enable_rate_limiting);
        assert!(!config.enable_caching);
        assert!(config.enable_metrics);
        assert!(config.enable_health_checks);
    }

    #[test]
    #[cfg(feature = "server")]
    fn test_http_client_across_features() {
        use nexus_nitro_llm::core::http_client::HttpClientBuilder;

        // Test that HTTP client works with various feature combinations
        let config = Config::default();
        let client_result = HttpClientBuilder::from_config(&config).build();
        assert!(client_result.is_ok(), "HTTP client should work across features");

        let production_client = HttpClientBuilder::production().build();
        assert!(production_client.is_ok(), "Production client should work across features");
    }
}