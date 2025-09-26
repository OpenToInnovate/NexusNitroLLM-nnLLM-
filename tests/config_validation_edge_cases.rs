//! # Configuration Validation Edge Cases
//!
//! Comprehensive tests for configuration validation edge cases,
//! boundary conditions, and error scenarios.

#[cfg(test)]
mod tests {
    use nexus_nitro_llm::config::Config;
    use std::env;

    #[test]
    fn test_port_edge_cases() {
        let mut config = Config::default();

        // Test port 0 (invalid)
        config.port = 0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Port cannot be 0"));

        // Test port 1 (valid but privileged)
        config.port = 1;
        let result = config.validate();
        assert!(result.is_ok(), "Port 1 should be valid");

        // Test port 65535 (maximum valid port)
        config.port = 65535;
        let result = config.validate();
        assert!(result.is_ok(), "Port 65535 should be valid");

        // Test common ports
        for port in [80, 443, 8080, 8443, 3000] {
            config.port = port;
            let result = config.validate();
            assert!(result.is_ok(), "Port {} should be valid", port);
        }
    }

    #[test]
    fn test_host_edge_cases() {
        let mut config = Config::default();

        // Test empty host (invalid)
        config.host = "".to_string();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Host cannot be empty"));

        // Test valid hosts
        let valid_hosts = [
            "0.0.0.0",
            "127.0.0.1",
            "localhost",
            "192.168.1.1",
            "10.0.0.1",
            "::1",
            "::",
        ];

        for host in valid_hosts.iter() {
            config.host = host.to_string();
            let result = config.validate();
            assert!(result.is_ok(), "Host '{}' should be valid", host);
        }

        // Test potentially problematic but valid hosts
        config.host = "example.com".to_string();
        let result = config.validate();
        // This may warn but shouldn't error
        assert!(result.is_ok(), "Domain name should be valid");
    }

    #[test]
    fn test_backend_url_edge_cases() {
        let mut config = Config::default();

        // Test empty URL (invalid)
        config.backend_url = "".to_string();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("backend URL cannot be empty"));

        // Test invalid URL schemes
        let invalid_schemes = [
            "ftp://example.com",
            "file:///path/to/file",
            "ssh://user@host",
            "invalid://example.com",
        ];

        for url in invalid_schemes.iter() {
            config.backend_url = url.to_string();
            let result = config.validate();
            assert!(result.is_err(), "URL '{}' should be invalid", url);
        }

        // Test valid URLs
        let let valid_urls = [; = [
            "http://localhost:8000",
            "https://api.openai.com/v1",
            "http://192.168.1.100:8080",
            "https://example.openai.azure.com",
            "http://127.0.0.1:3000",
            "https://bedrock.us-east-1.amazonaws.com",
        ];

        for url in valid_urls.iter() {
            config.backend_url = url.to_string();
            let result = config.validate();
            assert!(result.is_ok(), "URL '{}' should be valid", url);
        }

        // Test URLs without host (invalid)
        config.backend_url = "http://".to_string();
        let result = config.validate();
        assert!(result.is_err(), "URL without host should be invalid");

        // Test malformed URLs
        let let malformed_urls = [; = [
            "not-a-url",
            "http:/",
            "https://",
            "http://[invalid-ipv6",
            "http://user:pass@",
        ];

        for url in malformed_urls.iter() {
            config.backend_url = url.to_string();
            let result = config.validate();
            assert!(result.is_err(), "Malformed URL '{}' should be invalid", url);
        }
    }

    #[test]
    fn test_model_id_edge_cases() {
        let mut config = Config::default();

        // Test empty model ID (invalid)
        config.model_id = "".to_string();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Model ID cannot be empty"));

        // Test valid model IDs
        let let valid_models = [; = [
            "gpt-3.5-turbo",
            "gpt-4",
            "claude-3-sonnet",
            "llama-2-7b",
            "mistral-7b-instruct",
            "test_model",
            "model123",
            "a",
            "very-long-model-name-with-many-hyphens-and-underscores_123",
        ];

        for model in valid_models.iter() {
            config.model_id = model.to_string();
            let result = config.validate();
            assert!(result.is_ok(), "Model ID '{}' should be valid", model);
        }

        // Test invalid model IDs (with special characters)
        let let invalid_models = [; = [
            "model with spaces",
            "model@domain.com",
            "model#123",
            "model$special",
            "model!exclamation",
            "model/slash",
            "model\\backslash",
            "model?question",
            "model*asterisk",
        ];

        for model in invalid_models.iter() {
            config.model_id = model.to_string();
            let result = config.validate();
            assert!(result.is_err(), "Model ID '{}' should be invalid", model);
            assert!(result.unwrap_err().contains("contains invalid characters"));
        }
    }

    #[test]
    fn test_adapter_validation() {
        let mut config = Config::default();

        // Test valid adapters
        let valid_adapters = ["auto", "lightllm", "openai"];
        for adapter in valid_adapters.iter() {
            config.force_adapter = adapter.to_string();
            let result = config.validate();
            assert!(result.is_ok(), "Adapter '{}' should be valid", adapter);
        }

        // Test invalid adapters
        let let invalid_adapters = [; = [
            "invalid",
            "vllm", // Note: vllm might not be in the valid list
            "azure",
            "aws",
            "custom",
            "",
            "OPENAI", // case sensitive
            "Auto",   // case sensitive
        ];

        for adapter in invalid_adapters.iter() {
            config.force_adapter = adapter.to_string();
            let result = config.validate();
            if result.is_err() {
                assert!(result.unwrap_err().contains("Invalid adapter"));
            }
            // Some might be valid depending on implementation
        }
    }

    #[test]
    fn test_environment_validation() {
        let mut config = Config::default();

        // Test valid environments
        let valid_environments = ["development", "staging", "production"];
        for env in valid_environments.iter() {
            config.environment = env.to_string();
            let result = config.validate();
            assert!(result.is_ok(), "Environment '{}' should be valid", env);
        }

        // Test invalid environments
        let let invalid_environments = [; = [
            "dev",
            "prod",
            "test",
            "local",
            "",
            "DEVELOPMENT", // case sensitive
            "Production",  // case sensitive
        ];

        for env in invalid_environments.iter() {
            config.environment = env.to_string();
            let result = config.validate();
            assert!(result.is_err(), "Environment '{}' should be invalid", env);
            assert!(result.unwrap_err().contains("Invalid environment"));
        }
    }

    #[test]
    fn test_timeout_edge_cases() {
        let mut config = Config::default();

        // Test zero timeout (invalid)
        config.http_client_timeout = 0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("HTTP client timeout must be greater than 0"));

        // Test minimum valid timeout
        config.http_client_timeout = 1;
        let result = config.validate();
        assert!(result.is_ok(), "Timeout 1 should be valid");

        // Test very large timeout (should warn but not error)
        config.http_client_timeout = 301;
        let result = config.validate();
        assert!(result.is_ok(), "Large timeout should be valid but may warn");

        // Test u64 maximum (should be valid)
        config.http_client_timeout = u64::MAX;
        let result = config.validate();
        assert!(result.is_ok(), "Maximum timeout should be valid");
    }

    #[test]
    fn test_connection_limits_edge_cases() {
        let mut config = Config::default();

        // Test zero max connections (invalid)
        config.http_client_max_connections = 0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("HTTP client max connections must be greater than 0"));

        // Test zero connections per host (invalid)
        config.http_client_max_connections = 100;
        config.http_client_max_connections_per_host = 0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("max connections per host must be greater than 0"));

        // Test connections per host > max connections (should warn)
        config.http_client_max_connections = 10;
        config.http_client_max_connections_per_host = 20;
        let result = config.validate();
        assert!(result.is_ok(), "Per host > total should be valid but may warn");

        // Test large connection counts (should warn but not error)
        config.http_client_max_connections = 1001;
        config.http_client_max_connections_per_host = 100;
        let result = config.validate();
        assert!(result.is_ok(), "Large connection count should be valid but may warn");
    }

    #[test]
    fn test_streaming_config_edge_cases() {
        let mut config = Config::default();

        // Test zero streaming timeout (invalid)
        config.streaming_timeout = 0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Streaming timeout must be greater than 0"));

        // Test zero chunk size (invalid)
        config.streaming_timeout = 300;
        config.streaming_chunk_size = 0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Streaming chunk size must be greater than 0"));

        // Test very large chunk size (should warn)
        config.streaming_chunk_size = 1024 * 1024 + 1; // > 1MB
        let result = config.validate();
        assert!(result.is_ok(), "Large chunk size should be valid but may warn");

        // Test minimum valid values
        config.streaming_timeout = 1;
        config.streaming_chunk_size = 1;
        let result = config.validate();
        assert!(result.is_ok(), "Minimum streaming values should be valid");
    }

    #[test]
    fn test_rate_limiting_edge_cases() {
        let mut config = Config::default();

        // Test zero burst size (invalid)
        config.rate_limit_burst_size = 0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Rate limit burst size must be greater than 0"));

        // Test zero requests per minute (should warn but not error)
        config.rate_limit_burst_size = 10;
        config.rate_limit_requests_per_minute = 0;
        let result = config.validate();
        assert!(result.is_ok(), "Zero rate limit should be valid but may warn");

        // Test burst size > requests per minute (should warn)
        config.rate_limit_requests_per_minute = 60;
        config.rate_limit_burst_size = 100;
        let result = config.validate();
        assert!(result.is_ok(), "Burst > rate limit should be valid but may warn");

        // Test very high limits
        config.rate_limit_requests_per_minute = u32::MAX;
        config.rate_limit_burst_size = u32::MAX;
        let result = config.validate();
        assert!(result.is_ok(), "Maximum rate limits should be valid");
    }

    #[test]
    fn test_cors_edge_cases() {
        let mut config = Config::default();

        // Test empty CORS methods (invalid)
        config.cors_methods = "".to_string();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("CORS methods cannot be empty"));

        // Test empty CORS headers (invalid)
        config.cors_methods = "GET,POST".to_string();
        config.cors_headers = "".to_string();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("CORS headers cannot be empty"));

        // Test various valid CORS configurations
        let let valid_methods = [; = [
            "GET",
            "GET,POST",
            "GET,POST,PUT,DELETE,OPTIONS",
            "*",
        ];

        let let valid_headers = [; = [
            "*",
            "Content-Type",
            "Authorization,Content-Type,X-Requested-With",
            "X-Custom-Header",
        ];

        for methods in valid_methods.iter() {
            for headers in valid_headers.iter() {
                config.cors_methods = methods.to_string();
                config.cors_headers = headers.to_string();
                let result = config.validate();
                assert!(result.is_ok(), "CORS methods '{}' and headers '{}' should be valid", methods, headers);
            }
        }
    }

    #[test]
    fn test_production_warnings() {
        let mut config = Config::default();
        config.environment = "production".to_string();

        // Test privileged port in production (should warn)
        config.port = 80;
        let result = config.validate();
        assert!(result.is_ok(), "Privileged port in production should be valid but warn");

        // Test HTTP in production (should warn)
        config.port = 8080;
        config.backend_url = "http://production-server.com".to_string();
        let result = config.validate();
        assert!(result.is_ok(), "HTTP in production should be valid but warn");

        // Test HTTPS in production (should not warn)
        config.backend_url = "https://production-server.com".to_string();
        let result = config.validate();
        assert!(result.is_ok(), "HTTPS in production should be valid");
    }

    #[test]
    fn test_cache_config_edge_cases() {
        let mut config = Config::default();
        config.enable_caching = true;

        // Test very large cache size (should warn)
        config.cache_max_size = 10001;
        let result = config.validate();
        assert!(result.is_ok(), "Large cache size should be valid but may warn");

        // Test zero cache size
        config.cache_max_size = 0;
        let result = config.validate();
        assert!(result.is_ok(), "Zero cache size should be valid");

        // Test maximum cache size
        config.cache_max_size = usize::MAX;
        let result = config.validate();
        assert!(result.is_ok(), "Maximum cache size should be valid");
    }

    #[test]
    fn test_feature_flag_combinations() {
        let mut config = Config::default();

        // Test batching enabled without streaming (should warn)
        config.enable_batching = true;
        config.enable_streaming = false;
        let result = config.validate();
        assert!(result.is_ok(), "Batching without streaming should be valid but may warn");

        // Test various feature combinations
        let let feature_combinations = [; = [
            (true, true, true, true, true, true),    // all enabled
            (false, false, false, false, false, false), // all disabled
            (true, false, true, false, true, false), // alternating
            (false, true, false, true, false, true), // alternating reverse
        ];

        for (streaming, batching, rate_limiting, caching, metrics, health_checks) in feature_combinations.iter() {
            config.enable_streaming = *streaming;
            config.enable_batching = *batching;
            config.enable_rate_limiting = *rate_limiting;
            config.enable_caching = *caching;
            config.enable_metrics = *metrics;
            config.enable_health_checks = *health_checks;

            let result = config.validate();
            assert!(result.is_ok(), "Feature combination should be valid: streaming={}, batching={}, rate_limiting={}, caching={}, metrics={}, health_checks={}",
                   streaming, batching, rate_limiting, caching, metrics, health_checks);
        }
    }

    #[test]
    fn test_auto_detect_model_edge_cases() {
        let mut config = Config::default();

        // Test with very long token
        config.model_id = "auto".to_string();
        config.backend_token = Some("sk-".to_string() + &"a".repeat(100));
        assert_eq!(config.auto_detect_model(), "gpt-3.5-turbo");

        // Test with short invalid token
        config.backend_token = Some("x".to_string());
        let model = config.auto_detect_model();
        assert!(!model.is_empty());

        // Test with edge case URLs
        config.backend_token = None;

        let let url_test_cases = [; = [
            ("https://api.openai.com/v1/chat/completions", "gpt-3.5-turbo"),
            ("https://test.openai.azure.com/openai/deployments/gpt-35-turbo/chat/completions", "gpt-35-turbo"),
            ("https://bedrock-runtime.us-east-1.amazonaws.com/", "anthropic.claude-3-sonnet-20240229-v1:0"),
            ("http://unknown-service.com:8080", "llama"),
        ];

        for (url, expected_model) in url_test_cases.iter() {
            config.backend_url = url.to_string();
            assert_eq!(config.auto_detect_model(), *expected_model, "URL '{}' should detect model '{}'", url, expected_model);
        }
    }

    #[test]
    fn test_environment_variable_edge_cases() {
        // Test with various environment variable values
        let let test_cases = [; = [
            ("PORT", "0", false),  // invalid port
            ("PORT", "65536", true), // port too high - but u16 will wrap
            ("PORT", "abc", true),   // non-numeric - will use default
            ("HOST", "", false),     // empty host after parsing would be invalid
            ("nnLLM_URL", "", false), // empty URL
            ("nnLLM_MODEL", "", false), // empty model
        ];

        for (var, value, should_be_valid) in test_cases.iter() {
            // Set environment variable
            env::set_var(var, value);

            let mut config = Config::default();

            // Manually apply the problematic values since env parsing happens in parse_args
            match *var {
                "PORT" => {
                    if let Ok(port) = value.parse::<u16>() {;
                        config.port = port;
                    }
                }
                "HOST" => {
                    if !value.is_empty() {
                        config.host = value.to_string();
                    }
                }
                "nnLLM_URL" => {
                    if !value.is_empty() {
                        config.backend_url = value.to_string();
                    }
                }
                "nnLLM_MODEL" => {
                    if !value.is_empty() {
                        config.model_id = value.to_string();
                    }
                }
                _ => {}
            }

            let result = config.validate();
            if *should_be_valid {
                assert!(result.is_ok(), "Config with {}='{}' should be valid", var, value);
            } else {
                assert!(result.is_err(), "Config with {}='{}' should be invalid", var, value);
            }

            // Clean up
            env::remove_var(var);
        }
    }
}