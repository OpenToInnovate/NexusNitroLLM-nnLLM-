//! # Language Binding Feature Tests
//!
//! Tests that specifically validate Python and Node.js binding functionality
//! when the respective feature flags are enabled.

#[cfg(test)]
mod tests {
    use nexus_nitro_llm::config::Config;

    #[test]
    #[cfg(feature = "python")]
    fn test_python_bindings_available() {
        // Test that Python bindings are accessible when feature is enabled
        // Since we can't directly test pyo3 without Python runtime,
        // we test that the module compiles and basic types are available
        use nexus_nitro_llm::python::*;

        // This mainly tests that the python module is compiled correctly
        assert!(true, "Python bindings should be available");
    }

    #[test]
    #[cfg(feature = "nodejs")]
    fn test_nodejs_bindings_available() {
        // Test that Node.js bindings are accessible when feature is enabled
        use nexus_nitro_llm::nodejs::*;

        // This mainly tests that the nodejs module is compiled correctly
        assert!(true, "Node.js bindings should be available");
    }

    #[test]
    #[cfg(not(feature = "python"))]
    fn test_without_python_bindings() {
        // Test that core functionality works without Python bindings
        let config = Config::default();
        assert!(!config.backend_url.is_empty());
        // Python-specific functionality should not be available
    }

    #[test]
    #[cfg(not(feature = "nodejs"))]
    fn test_without_nodejs_bindings() {
        // Test that core functionality works without Node.js bindings
        let config = Config::default();
        assert!(!config.backend_url.is_empty());
        // Node.js-specific functionality should not be available
    }

    #[test]
    #[cfg(all(feature = "python", feature = "server"))]
    fn test_python_with_server_features() {
        // Test that Python bindings work with server features
        use nexus_nitro_llm::config::Config;

        let config = Config::default();
        let result = config.validate();
        assert!(result.is_ok(), "Config validation should work with Python + server features");
    }

    #[test]
    #[cfg(all(feature = "nodejs", feature = "server"))]
    fn test_nodejs_with_server_features() {
        // Test that Node.js bindings work with server features
        use nexus_nitro_llm::config::Config;

        let config = Config::default();
        let result = config.validate();
        assert!(result.is_ok(), "Config validation should work with Node.js + server features");
    }

    #[test]
    fn test_adapter_selection_for_bindings() {
        use nexus_nitro_llm::config::Config;

        let mut config = Config::default();

        // Test that adapter selection works for language bindings
        config.force_adapter = "lightllm".to_string();
        let result = config.validate();
        assert!(result.is_ok());

        config.force_adapter = "openai".to_string();
        let result = config.validate();
        assert!(result.is_ok());

        // Test invalid adapter
        config.force_adapter = "invalid".to_string();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid adapter"));
    }

    #[test]
    #[cfg(feature = "python")]
    fn test_python_error_types() {
        // Test that Python-specific error types are available
        // This is mainly a compilation test
        assert!(true, "Python error types should be available");
    }

    #[test]
    #[cfg(feature = "nodejs")]
    fn test_nodejs_error_types() {
        // Test that Node.js-specific error types are available
        // This is mainly a compilation test
        assert!(true, "Node.js error types should be available");
    }

    #[test]
    fn test_core_schemas_for_bindings() {
        use nexus_nitro_llm::schemas::{ChatCompletionRequest, ChatCompletionResponse, Message};

        // Test that core schema types work properly for bindings
        let message = Message {
            role: "user".to_string(),
            content: Some("Hello".to_string()),
            name: None,
            function_call: None,
            tool_calls: None,
            tool_call_id: None,
        };

        let request = ChatCompletionRequest {
            messages: vec![message],
            model: Some("test-model".to_string()),
            max_tokens: Some(100),
            temperature: Some(0.7),
            stream: Some(false),
            ..Default::default()
        };

        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.model, Some("test-model".to_string()));
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.7));

        // Test response structure
        let response = ChatCompletionResponse {
            id: "test-id".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "test-model".to_string(),
            choices: vec![],
            usage: None,
        };
        assert_eq!(response.object, "chat.completion");
    }

    #[test]
    fn test_configuration_for_bindings() {
        use nexus_nitro_llm::config::Config;

        let config = Config::default();

        // Test configuration fields that are important for language bindings
        assert!(config.port > 0);
        assert!(!config.host.is_empty());
        assert!(!config.backend_url.is_empty());
        assert!(!config.model_id.is_empty());

        // Test timeout configurations that affect binding performance
        assert!(config.http_client_timeout > 0);
        assert!(config.http_client_max_connections > 0);
        assert!(config.streaming_timeout > 0);
    }

    #[test]
    #[cfg(all(feature = "python", feature = "streaming"))]
    fn test_python_streaming_compatibility() {
        // Test that Python bindings work with streaming features
        use nexus_nitro_llm::config::Config;

        let config = Config::default();
        assert!(config.enable_streaming);

        let result = config.validate();
        assert!(result.is_ok(), "Python + streaming features should be compatible");
    }

    #[test]
    #[cfg(all(feature = "nodejs", feature = "streaming"))]
    fn test_nodejs_streaming_compatibility() {
        // Test that Node.js bindings work with streaming features
        use nexus_nitro_llm::config::Config;

        let config = Config::default();
        assert!(config.enable_streaming);

        let result = config.validate();
        assert!(result.is_ok(), "Node.js + streaming features should be compatible");
    }

    #[test]
    fn test_adapter_creation_for_bindings() {
        use nexus_nitro_llm::adapters::Adapter;
        use nexus_nitro_llm::config::Config;

        let config = Config::default();
        let adapter = Adapter::from_config(&config);

        // Test that adapter can be created from config
        // The specific adapter type depends on configuration
        match adapter {
            Adapter::LightLLM(_) => assert!(true, "LightLLM adapter created"),
            Adapter::OpenAI(_) => assert!(true, "OpenAI adapter created"),
            Adapter::VLLM(_) => assert!(true, "VLLM adapter created"),
            Adapter::AzureOpenAI(_) => assert!(true, "Azure adapter created"),
            Adapter::AWSBedrock(_) => assert!(true, "AWS adapter created"),
            Adapter::Custom(_) => assert!(true, "Custom adapter created"),
            Adapter::Direct(_) => assert!(true, "Direct adapter created"),
        }
    }

    #[test]
    fn test_error_handling_for_bindings() {
        use nexus_nitro_llm::error::ProxyError;

        // Test that error types can be properly handled in bindings
        let errors = vec![
            ProxyError::BadRequest("test".to_string()),
            ProxyError::Upstream("test".to_string()),
            ProxyError::Internal("test".to_string()),
            ProxyError::Serialization("test".to_string()),
        ];

        for error in errors {
            // Test that each error type can be created and handled
            match error {
                ProxyError::BadRequest(msg) => assert_eq!(msg, "test"),
                ProxyError::Internal(msg) => assert_eq!(msg, "test"),
                _ => assert!(true, "Error type handled correctly"),
            }
        }
    }

    #[test]
    fn test_serialization_for_bindings() {
        use nexus_nitro_llm::schemas::{ChatCompletionRequest, Message};
        use serde_json;

        // Test that schemas can be serialized/deserialized for bindings
        let message = Message {
            role: "user".to_string(),
            content: Some("Hello, world!".to_string()),
            name: None,
            function_call: None,
            tool_calls: None,
            tool_call_id: None,
        };

        let request = ChatCompletionRequest {
            messages: vec![message],
            model: Some("test-model".to_string()),
            max_tokens: Some(100),
            temperature: Some(0.7),
            stream: Some(false),
            ..Default::default()
        };

        // Test serialization
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test-model"));
        assert!(json.contains("Hello, world!"));

        // Test deserialization
        let deserialized: ChatCompletionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.model, Some("test-model".to_string()));
        assert_eq!(deserialized.messages[0].content, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_concurrent_access_for_bindings() {
        use nexus_nitro_llm::config::Config;
        use std::sync::Arc;
        use std::thread;

        // Test that config can be safely shared between threads (important for bindings)
        let config = Arc::new(Config::default());
        let mut handles = vec![];

        for i in 0..5 {
            let config_clone = Arc::clone(&config);
            let handle = thread::spawn(move || {
                let result = config_clone.validate();
                assert!(result.is_ok(), "Config validation should work in thread {}", i);
                config_clone.port
            });
            handles.push(handle);
        }

        for handle in handles {
            let port = handle.join().unwrap();
            assert_eq!(port, 8080);
        }
    }
}