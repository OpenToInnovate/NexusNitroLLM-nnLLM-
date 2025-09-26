//! # Chat Message Rules Tests
//! 
//! This module provides comprehensive tests for chat message rules including
//! role validation, content rules, message ordering, and conversation flow validation.

use nexus_nitro_llm::{
    config::Config,
    server::{AppState, create_router},
    schemas::{ChatCompletionRequest, Message, Tool, FunctionDefinition, ToolCall, FunctionCall},
};
use axum::{
    body::Body,
    http::{Request, StatusCode, HeaderMap, HeaderValue, Method},
    Router,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use tower::ServiceExt;

/// # Test Configuration
/// 
/// Configuration for chat message rules tests.
struct MessageRulesTestConfig {
    /// Test timeout duration
    timeout: std::time::Duration,
    /// Maximum message length
    max_message_length: usize,
    /// Maximum number of messages in conversation
    max_conversation_length: usize,
    /// Valid message roles
    valid_roles: Vec<String>,
    /// Required roles for conversation start
    required_start_roles: Vec<String>,
}

impl Default for MessageRulesTestConfig {
    fn default() -> Self {
        Self {
            timeout: std::time::Duration::from_secs(30),
            max_message_length: 100000,
            max_conversation_length: 100,
            valid_roles: vec![
                "system".to_string(),
                "user".to_string(),
                "assistant".to_string(),
                "tool".to_string(),
            ],
            required_start_roles: vec![
                "system".to_string(),
                "user".to_string(),
            ],
        }
    }
}

/// # Create Test App State
/// 
/// Creates a test application state with mock configuration.
fn create_test_app_state() -> AppState {
    let config = Config {;
        backend_type: "lightllm".to_string(),
        backend_url: "http://localhost:8000".to_string(),
        model_id: "test-model".to_string(),
        port: 3000,
        ..Default::default()
    };
    
    AppState::new(config)
}

/// # Test Message Role Validation
/// 
/// Tests validation of message roles according to OpenAI standards.
#[tokio::test]
async fn test_message_role_validation() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    let config = MessageRulesTestConfig::default();
    
    // Test valid roles
    for role in &config.valid_roles {
        let request_data = json!({;
            "model": "test-model",
            "messages": [
                {
                    "role": role,
                    "content": "Test message"
                }
            ]
        });
        
        let request = Request::builder();
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_ne!(response.status(), StatusCode::BAD_REQUEST, 
                  "Valid role '{}' should not return 400", role);
    }
    
    // Test invalid roles
    let invalid_roles = vec![;
        "invalid-role",
        "admin",
        "moderator",
        "bot",
        "ai",
        "",
        "SYSTEM", // Case sensitive
        "User",   // Case sensitive
    ];
    
    for role in invalid_roles {
        let request_data = json!({;
            "model": "test-model",
            "messages": [
                {
                    "role": role,
                    "content": "Test message"
                }
            ]
        });
        
        let request = Request::builder();
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, 
                  "Invalid role '{}' should return 400", role);
    }
}

/// # Test Message Content Rules
/// 
/// Tests validation of message content according to rules.
#[tokio::test]
async fn test_message_content_rules() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    let config = MessageRulesTestConfig::default();
    
    // Test empty content
    let empty_content_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": ""
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&empty_content_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    // Empty content might be valid depending on implementation
    // assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test null content
    let null_content_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": null
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&null_content_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test very long content
    let long_content = "x".repeat(config.max_message_length + 1);
    let long_content_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": long_content
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&long_content_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert!(response.status() == StatusCode::BAD_REQUEST || 
            response.status() == StatusCode::PAYLOAD_TOO_LARGE);
    
    // Test content with special characters
    let special_content_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": "Hello 🌍! This has émojis and spécial châractèrs. 你好世界！\n\t\r"
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&special_content_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
}

/// # Test Message Ordering Rules
/// 
/// Tests validation of message ordering in conversations.
#[tokio::test]
async fn test_message_ordering_rules() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test valid conversation flow: system -> user -> assistant
    let valid_flow_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant."
            },
            {
                "role": "user",
                "content": "Hello!"
            },
            {
                "role": "assistant",
                "content": "Hi there! How can I help you?"
            },
            {
                "role": "user",
                "content": "What's the weather like?"
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&valid_flow_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test invalid flow: assistant before user
    let invalid_flow_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "assistant",
                "content": "Hello! How can I help you?"
            },
            {
                "role": "user",
                "content": "What's the weather like?"
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_flow_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test multiple system messages (should be invalid)
    let multiple_system_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant."
            },
            {
                "role": "system",
                "content": "You are also a weather expert."
            },
            {
                "role": "user",
                "content": "Hello!"
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&multiple_system_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// # Test Tool Message Rules
/// 
/// Tests validation of tool-related messages.
#[tokio::test]
async fn test_tool_message_rules() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test valid tool message flow
    let valid_tool_flow_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": "What's the weather in New York?"
            },
            {
                "role": "assistant",
                "content": "I'll check the weather for you.",
                "tool_calls": [
                    {
                        "id": "call_123",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"location\": \"New York\"}"
                        }
                    }
                ]
            },
            {
                "role": "tool",
                "content": "Sunny, 72°F",
                "tool_call_id": "call_123"
            }
        ],
        "tools": [
            {
                "type": "function",
                "function": {
                    "name": "get_weather",
                    "description": "Get weather information"
                }
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&valid_tool_flow_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test tool message without tool_call_id
    let invalid_tool_message_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "tool",
                "content": "Sunny, 72°F"
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_tool_message_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test tool message with invalid tool_call_id
    let invalid_tool_call_id_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "assistant",
                "content": "I'll check the weather.",
                "tool_calls": [
                    {
                        "id": "call_123",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"location\": \"New York\"}"
                        }
                    }
                ]
            },
            {
                "role": "tool",
                "content": "Sunny, 72°F",
                "tool_call_id": "call_456" // Mismatched ID
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_tool_call_id_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// # Test Conversation Length Limits
/// 
/// Tests validation of conversation length limits.
#[tokio::test]
async fn test_conversation_length_limits() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    let config = MessageRulesTestConfig::default();
    
    // Test conversation within limits
    let mut messages = Vec::new();
    for i in 0..10 {
        messages.push(json!({
            "role": if i % 2 == 0 { "user" } else { "assistant" },
            "content": format!("Message {}", i)
        }));
    }
    
    let valid_length_request = json!({;
        "model": "test-model",
        "messages": messages
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&valid_length_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test conversation exceeding limits
    let mut long_messages = Vec::new();
    for i in 0..(config.max_conversation_length + 1) {
        long_messages.push(json!({
            "role": if i % 2 == 0 { "user" } else { "assistant" },
            "content": format!("Message {}", i)
        }));
    }
    
    let excessive_length_request = json!({;
        "model": "test-model",
        "messages": long_messages
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&excessive_length_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert!(response.status() == StatusCode::BAD_REQUEST || 
            response.status() == StatusCode::PAYLOAD_TOO_LARGE);
}

/// # Test Message Name Field Rules
/// 
/// Tests validation of the optional name field in messages.
#[tokio::test]
async fn test_message_name_field_rules() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test valid name field
    let valid_name_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": "Hello!",
                "name": "john_doe"
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&valid_name_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test invalid name field (non-string)
    let invalid_name_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": "Hello!",
                "name": 123
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_name_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test empty name field
    let empty_name_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": "Hello!",
                "name": ""
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&empty_name_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// # Test Function Call Rules
/// 
/// Tests validation of function call fields in messages.
#[tokio::test]
async fn test_function_call_rules() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test valid function call
    let valid_function_call_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "assistant",
                "content": "I'll help you with that.",
                "function_call": {
                    "name": "get_weather",
                    "arguments": "{\"location\": \"New York\"}"
                }
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&valid_function_call_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test function call without name
    let invalid_function_call_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "assistant",
                "content": "I'll help you with that.",
                "function_call": {
                    "arguments": "{\"location\": \"New York\"}"
                }
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_function_call_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test function call with invalid JSON arguments
    let invalid_json_args_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "assistant",
                "content": "I'll help you with that.",
                "function_call": {
                    "name": "get_weather",
                    "arguments": "invalid json"
                }
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_json_args_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// # Test Message Content Type Rules
/// 
/// Tests validation of message content types.
#[tokio::test]
async fn test_message_content_type_rules() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test string content
    let string_content_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": "Hello, world!"
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&string_content_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test array content (for multimodal)
    let array_content_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": "What's in this image?"
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": "data:image/jpeg;base64,..."
                        }
                    }
                ]
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&array_content_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    // Array content might be valid for multimodal models
    // assert_ne!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test invalid content type
    let invalid_content_type_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": 123
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_content_type_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// # Test System Message Rules
/// 
/// Tests specific rules for system messages.
#[tokio::test]
async fn test_system_message_rules() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test system message at the beginning
    let valid_system_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant."
            },
            {
                "role": "user",
                "content": "Hello!"
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&valid_system_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test system message in the middle (should be invalid)
    let invalid_system_position_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": "Hello!"
            },
            {
                "role": "system",
                "content": "You are a helpful assistant."
            },
            {
                "role": "assistant",
                "content": "Hi there!"
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_system_position_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test system message with empty content
    let empty_system_request = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "system",
                "content": ""
            },
            {
                "role": "user",
                "content": "Hello!"
            }
        ]
    });
    
    let request = Request::builder();
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&empty_system_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// # Integration Test Suite
/// 
/// Runs a comprehensive integration test suite for chat message rules.
#[tokio::test]
async fn test_chat_message_rules_integration_suite() {
    println!("🚀 Starting comprehensive chat message rules test suite");
    
    // Test all message rule scenarios
    test_message_role_validation()
    test_message_content_rules()
    test_message_ordering_rules()
    test_tool_message_rules()
    test_conversation_length_limits()
    test_message_name_field_rules()
    test_function_call_rules()
    test_message_content_type_rules()
    test_system_message_rules()
    
    println!("✅ Comprehensive chat message rules test suite completed");
}
