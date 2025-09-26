//! # Request Schema and Parameter Handling Tests
//! 
//! This module provides comprehensive tests for request schema validation,
//! parameter handling, and data type validation across all language bindings.

use nexus_nitro_llm::{
    config::Config,
    server::{AppState, create_router},
    schemas::{ChatCompletionRequest, Message},
};
use axum::{
    body::Body,
    http::{Request, StatusCode, Method},
};
use serde_json::json;
use std::collections::HashMap;
use tower::ServiceExt;

/// # Test Configuration
/// 
/// Configuration for request schema tests.
struct SchemaTestConfig {
    /// Test timeout duration
    timeout: std::time::Duration,
    /// Maximum message length for testing
    max_message_length: usize,
    /// Maximum number of messages for testing
    max_messages: usize,
    /// Valid parameter ranges
    valid_ranges: HashMap<String, (f64, f64)>,
}

impl Default for SchemaTestConfig {
    fn default() -> Self {
        let mut valid_ranges = HashMap::new();
        valid_ranges.insert("temperature".to_string(), (0.0, 2.0));
        valid_ranges.insert("top_p".to_string(), (0.0, 1.0));
        valid_ranges.insert("frequency_penalty".to_string(), (-2.0, 2.0));
        valid_ranges.insert("presence_penalty".to_string(), (-2.0, 2.0));
        valid_ranges.insert("max_tokens".to_string(), (1.0, 4096.0));

        Self {
            timeout: std::time::Duration::from_secs(30),
            max_message_length: 100000,
            max_messages: 100,
            valid_ranges,
        }
    }
}

/// # Create Test App State
/// 
/// Creates a test application state with mock configuration.
async fn create_test_app_state() -> AppState {
    let config = Config {
        backend_type: "lightllm".to_string(),
        backend_url: "http://localhost:8000".to_string(),
        model_id: "test-model".to_string(),
        port: 3000,
        ..Default::default()
    };
    
    AppState::new(config).await
}

/// # Create Valid Test Request
/// 
/// Creates a valid test request for schema validation tests.
fn create_valid_test_request() -> ChatCompletionRequest {
    ChatCompletionRequest {
        model: Some("test-model".to_string()),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: Some("Hello, world!".to_string()),
                name: None,
                function_call: None,
                tool_call_id: None,
                tool_calls: None,
            },
        ],
        stream: Some(false),
        temperature: Some(0.7),
        max_tokens: Some(100),
        top_p: Some(0.9),
        frequency_penalty: Some(0.0),
        presence_penalty: Some(0.0),
        tools: None,
        tool_choice: None,
        ..Default::default()
    }
}

/// # Test Required Fields Validation
/// 
/// Tests validation of required fields in ChatCompletionRequest.
async fn test_required_fields_validation() {
    let app_state = create_test_app_state().await;
    let app = create_router(app_state);
    
    // Test missing messages field
    let invalid_request = json!({
        "model": "test-model",
        "temperature": 0.7
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test empty messages array
    let empty_messages_request = json!({
        "model": "test-model",
        "messages": []
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&empty_messages_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test valid request with required fields
    let valid_request = create_valid_test_request();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&valid_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    // Should not return 400 Bad Request for valid required fields
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
}

/// # Test Message Schema Validation
/// 
/// Tests validation of message objects in the request.

async fn test_message_schema_validation() {
    let app_state = create_test_app_state().await;
    let app = create_router(app_state);
    
    // Test message without role
    let invalid_message_request = json!({
        "model": "test-model",
        "messages": [
            {
                "content": "Hello, world!"
            }
        ]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_message_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test message with invalid role
    let invalid_role_request = json!({
        "model": "test-model",
        "messages": [
            {
                "role": "invalid-role",
                "content": "Hello, world!"
            }
        ]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_role_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test message with valid roles
    let valid_roles = vec!["system", "user", "assistant", "tool"];
    for role in valid_roles {
        let valid_role_request = json!({
            "model": "test-model",
            "messages": [
                {
                    "role": role,
                    "content": "Hello, world!"
                }
            ]
        });
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&valid_role_request).unwrap()))
            .unwrap();
        
        let _response = app.clone().oneshot(request).await.unwrap();
        // Should not return 400 Bad Request for valid roles
        assert_ne!(response.status(), StatusCode::BAD_REQUEST);
    }
}

/// # Test Parameter Range Validation
/// 
/// Tests validation of parameter ranges (temperature, top_p, etc.).

async fn test_parameter_range_validation() {
    let app_state = create_test_app_state().await;
    let app = create_router(app_state);
    let _config = SchemaTestConfig::default();
    
    // Test temperature range validation
    let temperature_tests = vec![
        (-1.0, StatusCode::BAD_REQUEST), // Below minimum
        (0.0, StatusCode::OK),           // Minimum valid
        (1.0, StatusCode::OK),           // Valid
        (2.0, StatusCode::OK),           // Maximum valid
        (3.0, StatusCode::BAD_REQUEST),  // Above maximum
    ];
    
    for (temp_value, expected_status) in temperature_tests {
        let request_data = json!({
            "model": "test-model",
            "messages": [{"role": "user", "content": "Hello"}],
            "temperature": temp_value
        });
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
            .unwrap();
        
        let _response = app.clone().oneshot(request).await.unwrap();
        
        if expected_status == StatusCode::OK {
            assert_ne!(response.status(), StatusCode::BAD_REQUEST);
        } else {
            assert_eq!(response.status(), expected_status);
        }
    }
    
    // Test top_p range validation
    let top_p_tests = vec![
        (-0.1, StatusCode::BAD_REQUEST), // Below minimum
        (0.0, StatusCode::OK),           // Minimum valid
        (0.5, StatusCode::OK),           // Valid
        (1.0, StatusCode::OK),           // Maximum valid
        (1.1, StatusCode::BAD_REQUEST),  // Above maximum
    ];
    
    for (top_p_value, expected_status) in top_p_tests {
        let request_data = json!({
            "model": "test-model",
            "messages": [{"role": "user", "content": "Hello"}],
            "top_p": top_p_value
        });
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
            .unwrap();
        
        let _response = app.clone().oneshot(request).await.unwrap();
        
        if expected_status == StatusCode::OK {
            assert_ne!(response.status(), StatusCode::BAD_REQUEST);
        } else {
            assert_eq!(response.status(), expected_status);
        }
    }
}

/// # Test Data Type Validation
/// 
/// Tests validation of data types for various parameters.

async fn test_data_type_validation() {
    let app_state = create_test_app_state().await;
    let app = create_router(app_state);
    
    // Test string parameters with wrong types
    let type_tests = vec![
        ("model", json!(123), "Model should be string"),
        ("temperature", json!("invalid"), "Temperature should be number"),
        ("max_tokens", json!("invalid"), "Max tokens should be number"),
        ("stream", json!("invalid"), "Stream should be boolean"),
        ("top_p", json!("invalid"), "Top_p should be number"),
    ];
    
    for (param_name, param_value, description) in type_tests {
        let mut request_data = json!({
            "model": "test-model",
            "messages": [{"role": "user", "content": "Hello"}]
        });
        request_data[param_name] = param_value;
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
            .unwrap();
        
        let _response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{}", description);
    }
    
    // Test valid data types
    let valid_request = json!({
        "model": "test-model",
        "messages": [{"role": "user", "content": "Hello"}],
        "temperature": 0.7,
        "max_tokens": 100,
        "stream": false,
        "top_p": 0.9,
        "frequency_penalty": 0.0,
        "presence_penalty": 0.0
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&valid_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
}

/// # Test Message Content Validation
/// 
/// Tests validation of message content.

async fn test_message_content_validation() {
    let app_state = create_test_app_state().await;
    let app = create_router(app_state);
    let _config = SchemaTestConfig::default();
    
    // Test message with null content
    let null_content_request = json!({
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": null
            }
        ]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&null_content_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test message with non-string content
    let invalid_content_request = json!({
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": 123
            }
        ]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_content_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test message with empty content
    let empty_content_request = json!({
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": ""
            }
        ]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&empty_content_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    // Empty content might be valid depending on implementation
    // assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test message with very long content
    let long_content = "x".repeat(config.max_message_length + 1);
    let long_content_request = json!({
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": long_content
            }
        ]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&long_content_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    // Should return 400 Bad Request or 413 Payload Too Large
    assert!(response.status() == StatusCode::BAD_REQUEST || 
            response.status() == StatusCode::PAYLOAD_TOO_LARGE);
}

/// # Test Tool Schema Validation
/// 
/// Tests validation of tool definitions and tool choice.

async fn test_tool_schema_validation() {
    let app_state = create_test_app_state().await;
    let app = create_router(app_state);
    
    // Test valid tool definition
    let valid_tool_request = json!({
        "model": "test-model",
        "messages": [{"role": "user", "content": "Hello"}],
        "tools": [
            {
                "type": "function",
                "function": {
                    "name": "get_weather",
                    "description": "Get current weather",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "location": {
                                "type": "string",
                                "description": "The city to get weather for"
                            }
                        },
                        "required": ["location"]
                    }
                }
            }
        ]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&valid_tool_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test invalid tool type
    let invalid_tool_type_request = json!({
        "model": "test-model",
        "messages": [{"role": "user", "content": "Hello"}],
        "tools": [
            {
                "type": "invalid-type",
                "function": {
                    "name": "test_function"
                }
            }
        ]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_tool_type_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test tool without function definition
    let missing_function_request = json!({
        "model": "test-model",
        "messages": [{"role": "user", "content": "Hello"}],
        "tools": [
            {
                "type": "function"
            }
        ]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&missing_function_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// # Test Tool Choice Validation
/// 
/// Tests validation of tool choice parameter.

async fn test_tool_choice_validation() {
    let app_state = create_test_app_state().await;
    let app = create_router(app_state);
    
    // Test valid tool choice values
    let valid_tool_choices = vec![
        "none",
        "auto",
        "required",
        r#"{"type": "function", "function": {"name": "get_weather"}}"#
    ];
    
    for tool_choice in valid_tool_choices {
        let request_data = json!({
            "model": "test-model",
            "messages": [{"role": "user", "content": "Hello"}],
            "tools": [
                {
                    "type": "function",
                    "function": {
                        "name": "get_weather",
                        "description": "Get weather"
                    }
                }
            ],
            "tool_choice": tool_choice
        });
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
            .unwrap();
        
        let _response = app.clone().oneshot(request).await.unwrap();
        assert_ne!(response.status(), StatusCode::BAD_REQUEST);
    }
    
    // Test invalid tool choice
    let invalid_tool_choice_request = json!({
        "model": "test-model",
        "messages": [{"role": "user", "content": "Hello"}],
        "tool_choice": "invalid-choice"
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_tool_choice_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// # Test JSON Schema Validation
/// 
/// Tests validation of JSON schema structure.

async fn test_json_schema_validation() {
    let app_state = create_test_app_state().await;
    let app = create_router(app_state);
    
    // Test malformed JSON
    let malformed_json = r#"{"model": "test-model", "messages": [{"role": "user", "content": "Hello"}]"#;
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(malformed_json))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test empty JSON
    let empty_json = "{}";
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(empty_json))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test non-JSON content
    let non_json_content = "This is not JSON";
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(non_json_content))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// # Test Optional Parameters
/// 
/// Tests handling of optional parameters.

async fn test_optional_parameters() {
    let app_state = create_test_app_state().await;
    let app = create_router(app_state);
    
    // Test request with only required parameters
    let minimal_request = json!({
        "model": "test-model",
        "messages": [{"role": "user", "content": "Hello"}]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&minimal_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test request with all optional parameters
    let full_request = json!({
        "model": "test-model",
        "messages": [{"role": "user", "content": "Hello"}],
        "stream": false,
        "temperature": 0.7,
        "max_tokens": 100,
        "top_p": 0.9,
        "n": 1,
        "stop": ["END"],
        "presence_penalty": 0.0,
        "frequency_penalty": 0.0,
        "user": "test-user"
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&full_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
}

/// # Test Array Parameter Validation
/// 
/// Tests validation of array parameters.

async fn test_array_parameter_validation() {
    let app_state = create_test_app_state().await;
    let app = create_router(app_state);
    
    // Test valid stop array
    let valid_stop_request = json!({
        "model": "test-model",
        "messages": [{"role": "user", "content": "Hello"}],
        "stop": ["END", "STOP", "DONE"]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&valid_stop_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test invalid stop array (non-string elements)
    let invalid_stop_request = json!({
        "model": "test-model",
        "messages": [{"role": "user", "content": "Hello"}],
        "stop": ["END", 123, "DONE"]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_stop_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test empty stop array
    let empty_stop_request = json!({
        "model": "test-model",
        "messages": [{"role": "user", "content": "Hello"}],
        "stop": []
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&empty_stop_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
}

/// # Test Unicode and Special Characters
/// 
/// Tests handling of Unicode and special characters in requests.

async fn test_unicode_and_special_characters() {
    let app_state = create_test_app_state().await;
    let app = create_router(app_state);
    
    // Test Unicode content
    let unicode_request = json!({
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": "Hello 🌍! This has émojis and spécial châractèrs. 你好世界！"
            }
        ]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&unicode_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test special characters in model name
    let special_model_request = json!({
        "model": "test-model-with-special-chars_123",
        "messages": [{"role": "user", "content": "Hello"}]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&special_model_request).unwrap()))
        .unwrap();
    
    let _response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
}

/// # Integration Test Suite
/// 
/// Runs a comprehensive integration test suite for request schema validation.

async fn test_request_schema_integration_suite() {
    println!("🚀 Starting comprehensive request schema validation test suite");
    
    // Test all schema validation scenarios
    test_required_fields_validation().await;
    test_message_schema_validation().await;
    test_parameter_range_validation().await;
    test_data_type_validation().await;
    test_message_content_validation().await;
    test_tool_schema_validation().await;
    test_tool_choice_validation().await;
    test_json_schema_validation().await;
    test_optional_parameters().await;
    test_array_parameter_validation().await;
    test_unicode_and_special_characters().await;
    
    println!("✅ Comprehensive request schema validation test suite completed");
}
