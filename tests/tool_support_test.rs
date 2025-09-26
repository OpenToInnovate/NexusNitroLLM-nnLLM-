//! # Comprehensive Tool Support Tests
//! 
//! This module provides comprehensive tests for tool calls and tool use message types,
//! including validation, execution, and response formatting.

use nexus_nitro_llm::{
    schemas::{ChatCompletionRequest, Message, Tool, ToolChoice, FunctionDefinition, ToolCall, FunctionCall},
    tool_support::{
        ToolRole, ToolUseMessage, ToolCallExecutor, ToolCallValidator, ToolCallMessageBuilder,
        ToolCallResponseFormatter, ToolCallHistoryEntry,
    },
    function_calling::{FunctionRegistry, CalculatorExecutor, WeatherExecutor, SystemInfoExecutor},
};
use serde_json::json;
use std::sync::Arc;

/// # Test Tool Role Functionality
/// 
/// Tests the ToolRole enum and its conversions.
#[test]
fn test_tool_role_functionality() {
    // Test string conversion
    assert_eq!(ToolRole::System.as_str(), "system");
    assert_eq!(ToolRole::User.as_str(), "user");
    assert_eq!(ToolRole::Assistant.as_str(), "assistant");
    assert_eq!(ToolRole::Tool.as_str(), "tool");
    
    // Test parsing from string
    assert_eq!(ToolRole::from_str("system").unwrap(), ToolRole::System);
    assert_eq!(ToolRole::from_str("user").unwrap(), ToolRole::User);
    assert_eq!(ToolRole::from_str("assistant").unwrap(), ToolRole::Assistant);
    assert_eq!(ToolRole::from_str("tool").unwrap(), ToolRole::Tool);
    
    // Test invalid role
    assert!(ToolRole::from_str("invalid").is_err());
    assert!(ToolRole::from_str("").is_err());
}

/// # Test Tool Use Message Creation
/// 
/// Tests creating and manipulating tool use messages.
#[test]
fn test_tool_use_message_creation() {
    // Test basic message creation
    let user_message = ToolUseMessage::new(ToolRole::User, Some("Hello, world!".to_string()));
    assert_eq!(user_message.role, ToolRole::User);
    assert_eq!(user_message.content, Some("Hello, world!".to_string()));
    assert!(user_message.tool_calls.is_none());
    assert!(user_message.tool_call_id.is_none());
    
    // Test tool result message creation
    let tool_result = ToolUseMessage::tool_result(
        "call-123".to_string(),
        "The result is 42".to_string()
    );
    assert_eq!(tool_result.role, ToolRole::Tool);
    assert_eq!(tool_result.tool_call_id, Some("call-123".to_string()));
    assert_eq!(tool_result.content, Some("The result is 42".to_string()));
    
    // Test assistant message with tool calls
    let tool_call = ToolCall {
        id: "call-456".to_string(),
        tool_type: "function".to_string(),
        function: FunctionCall {
            name: "add".to_string(),
            arguments: json!({"a": 1, "b": 2}).to_string(),
        },
    };
    
    let assistant_message = ToolUseMessage::assistant_with_tool_calls(vec![tool_call.clone()]);
    assert_eq!(assistant_message.role, ToolRole::Assistant);
    assert_eq!(assistant_message.tool_calls, Some(vec![tool_call]));
    assert!(assistant_message.content.is_none());
}

/// # Test Message Conversion
/// 
/// Tests conversion between ToolUseMessage and standard Message.
#[test]
fn test_message_conversion() {
    // Test ToolUseMessage to Message
    let tool_use_message = ToolUseMessage::new(
        ToolRole::Assistant,
        Some("I'll help you with that".to_string())
    );
    
    let message = tool_use_message.to_message();
    assert_eq!(message.role, "assistant");
    assert_eq!(message.content, Some("I'll help you with that".to_string()));
    assert!(message.tool_calls.is_none());
    assert!(message.tool_call_id.is_none());
    
    // Test Message to ToolUseMessage
    let standard_message = Message {
        role: "tool".to_string(),
        content: Some("Tool result".to_string()),
        name: None,
        function_call: None,
        tool_call_id: Some("call-789".to_string()),
        tool_calls: None,
    };
    
    let converted_tool_message = ToolUseMessage::from_message(standard_message).unwrap();
    assert_eq!(converted_tool_message.role, ToolRole::Tool);
    assert_eq!(converted_tool_message.content, Some("Tool result".to_string()));
    assert_eq!(converted_tool_message.tool_call_id, Some("call-789".to_string()));
    
    // Test invalid role conversion
    let invalid_message = Message {
        role: "invalid".to_string(),
        content: Some("Test".to_string()),
        name: None,
        function_call: None,
        tool_call_id: None,
        tool_calls: None,
    };
    
    assert!(ToolUseMessage::from_message(invalid_message).is_err());
}

/// # Test Tool Call Executor
/// 
/// Tests the tool call executor functionality.
#[tokio::test]
async fn test_tool_call_executor() {
    // Create function registry with test functions
    let mut registry = FunctionRegistry::new();
    registry.register_function(Box::new(CalculatorExecutor));
    registry.register_function(Box::new(WeatherExecutor));
    registry.register_function(Box::new(SystemInfoExecutor));
    
    let executor = ToolCallExecutor::new(Arc::new(registry));
    
    // Test successful tool call execution
    let tool_call = ToolCall {
        id: "test-call-1".to_string(),
        tool_type: "function".to_string(),
        function: FunctionCall {
            name: "add".to_string(),
            arguments: json!({"a": 5, "b": 3}).to_string(),
        },
    };
    
    let results = executor.execute_tool_calls(vec![tool_call]).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].tool_call_id, "test-call-1");
    assert!(!results[0].is_error);
    
    // Parse the result to verify it's correct
    let result_value: serde_json::Value = serde_json::from_str(&results[0].content).unwrap();
    assert_eq!(result_value["result"], 8);
    
    // Test multiple tool calls
    let tool_calls = vec![
        ToolCall {
            id: "test-call-2".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "multiply".to_string(),
                arguments: json!({"a": 4, "b": 6}).to_string(),
            },
        },
        ToolCall {
            id: "test-call-3".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "get_system_info".to_string(),
                arguments: json!({}).to_string(),
            },
        },
    ];
    
    let results = executor.execute_tool_calls(tool_calls).await.unwrap();
    assert_eq!(results.len(), 2);
    
    // Verify multiplication result
    let multiply_result: serde_json::Value = serde_json::from_str(&results[0].content).unwrap();
    assert_eq!(multiply_result["result"], 24);
    
    // Verify system info result
    let system_info_result: serde_json::Value = serde_json::from_str(&results[1].content).unwrap();
    assert!(system_info_result["os"].is_string());
    assert!(system_info_result["arch"].is_string());
}

/// # Test Tool Call Executor Error Handling
/// 
/// Tests error handling in tool call execution.
#[tokio::test]
async fn test_tool_call_executor_error_handling() {
    let registry = FunctionRegistry::new();
    let executor = ToolCallExecutor::new(Arc::new(registry));
    
    // Test function not found
    let tool_call = ToolCall {
        id: "test-call-error".to_string(),
        tool_type: "function".to_string(),
        function: FunctionCall {
            name: "nonexistent_function".to_string(),
            arguments: json!({}).to_string(),
        },
    };
    
    let results = executor.execute_tool_calls(vec![tool_call]).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].is_error);
    assert!(results[0].content.contains("Function not found"));
    
    // Test invalid arguments
    let tool_call_invalid_args = ToolCall {
        id: "test-call-invalid-args".to_string(),
        tool_type: "function".to_string(),
        function: FunctionCall {
            name: "add".to_string(),
            arguments: "invalid json".to_string(),
        },
    };
    
    let results = executor.execute_tool_calls(vec![tool_call_invalid_args]).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].is_error);
    assert!(results[0].content.contains("Invalid function arguments"));
}

/// # Test Tool Call Validator
/// 
/// Tests the tool call validator functionality.
#[test]
fn test_tool_call_validator() {
    // Create test tools
    let tools = vec![
        Tool {
            tool_type: "function".to_string(),
            function: Some(FunctionDefinition {
                name: "add".to_string(),
                description: Some("Add two numbers".to_string()),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number"},
                        "b": {"type": "number"}
                    },
                    "required": ["a", "b"]
                })),
            }),
        },
        Tool {
            tool_type: "function".to_string(),
            function: Some(FunctionDefinition {
                name: "get_weather".to_string(),
                description: Some("Get weather information".to_string()),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    },
                    "required": ["location"]
                })),
            }),
        },
    ];
    
    let validator = ToolCallValidator::new(tools);
    
    // Test valid tool call
    let valid_tool_call = ToolCall {
        id: "valid-call".to_string(),
        tool_type: "function".to_string(),
        function: FunctionCall {
            name: "add".to_string(),
            arguments: json!({"a": 1, "b": 2}).to_string(),
        },
    };
    
    assert!(validator.validate_tool_call(&valid_tool_call).is_ok());
    
    // Test invalid function name
    let invalid_function_call = ToolCall {
        id: "invalid-function".to_string(),
        tool_type: "function".to_string(),
        function: FunctionCall {
            name: "nonexistent".to_string(),
            arguments: json!({"a": 1, "b": 2}).to_string(),
        },
    };
    
    assert!(validator.validate_tool_call(&invalid_function_call).is_err());
    
    // Test invalid JSON arguments
    let invalid_args_call = ToolCall {
        id: "invalid-args".to_string(),
        tool_type: "function".to_string(),
        function: FunctionCall {
            name: "add".to_string(),
            arguments: "invalid json".to_string(),
        },
    };
    
    assert!(validator.validate_tool_call(&invalid_args_call).is_err());
}

/// # Test Tool Choice Validation
/// 
/// Tests tool choice validation.
#[test]
fn test_tool_choice_validation() {
    let tools = vec![
        Tool {
            tool_type: "function".to_string(),
            function: Some(FunctionDefinition {
                name: "add".to_string(),
                description: Some("Add two numbers".to_string()),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number"},
                        "b": {"type": "number"}
                    },
                    "required": ["a", "b"]
                })),
            }),
        },
    ];
    
    let validator = ToolCallValidator::new(tools);
    
    // Test valid tool choices
    assert!(validator.validate_tool_choice(&ToolChoice::None).is_ok());
    assert!(validator.validate_tool_choice(&ToolChoice::Auto).is_ok());
    assert!(validator.validate_tool_choice(&ToolChoice::Required).is_ok());
    assert!(validator.validate_tool_choice(&ToolChoice::FunctionChoice {
        function_name: "add".to_string()
    }).is_ok());
    
    // Test invalid function choice
    assert!(validator.validate_tool_choice(&ToolChoice::FunctionChoice {
        function_name: "nonexistent".to_string()
    }).is_err());
    
    // Test required tool choice with no tools
    let empty_validator = ToolCallValidator::new(vec![]);
    assert!(empty_validator.validate_tool_choice(&ToolChoice::Required).is_err());
}

/// # Test Tool Call Message Builder
/// 
/// Tests the tool call message builder functionality.
#[tokio::test]
async fn test_tool_call_message_builder() {
    let mut registry = FunctionRegistry::new();
    registry.register_function(Box::new(CalculatorExecutor));
    
    let builder = ToolCallMessageBuilder::new(Arc::new(registry));
    
    // Test creating tool use conversation
    let messages = vec![
        Message {
            role: "user".to_string(),
            content: Some("What is 2 + 3?".to_string()),
            name: None,
            function_call: None,
            tool_call_id: None,
            tool_calls: None,
        },
        Message {
            role: "assistant".to_string(),
            content: Some("I'll calculate that for you.".to_string()),
            name: None,
            function_call: None,
            tool_call_id: None,
            tool_calls: Some(vec![ToolCall {
                id: "call-1".to_string(),
                tool_type: "function".to_string(),
                function: FunctionCall {
                    name: "add".to_string(),
                    arguments: json!({"a": 2, "b": 3}).to_string(),
                },
            }]),
        },
    ];
    
    let tools = vec![
        Tool {
            tool_type: "function".to_string(),
            function: Some(FunctionDefinition {
                name: "add".to_string(),
                description: Some("Add two numbers".to_string()),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number"},
                        "b": {"type": "number"}
                    },
                    "required": ["a", "b"]
                })),
            }),
        },
    ];
    
    let conversation = builder.build_tool_use_conversation(
        messages,
        Some(tools),
        Some(ToolChoice::Auto),
    ).await.unwrap();
    
    // Should have original messages plus tool result
    assert_eq!(conversation.len(), 3);
    
    // Check tool result message
    let tool_result_message = &conversation[2];
    assert_eq!(tool_result_message.role, "tool");
    assert_eq!(tool_result_message.tool_call_id, Some("call-1".to_string()));
    assert!(tool_result_message.content.is_some());
    
    // Parse tool result to verify it's correct
    let result_content = tool_result_message.content.as_ref().unwrap();
    let result_value: serde_json::Value = serde_json::from_str(result_content).unwrap();
    assert_eq!(result_value["result"], 5);
}

/// # Test Tool Call Response Formatter
/// 
/// Tests the tool call response formatter.
#[test]
fn test_tool_call_response_formatter() {
    let formatter = ToolCallResponseFormatter;
    
    // Test formatting tool call results
    let results = vec![
        ToolCallResult {
            tool_call_id: "call-1".to_string(),
            content: "Result 1".to_string(),
            is_error: false,
        },
        ToolCallResult {
            tool_call_id: "call-2".to_string(),
            content: "Result 2".to_string(),
            is_error: false,
        },
    ];
    
    let messages = formatter.format_tool_call_results(results);
    assert_eq!(messages.len(), 2);
    
    // Check first message
    assert_eq!(messages[0].role, "tool");
    assert_eq!(messages[0].tool_call_id, Some("call-1".to_string()));
    assert_eq!(messages[0].content, Some("Result 1".to_string()));
    
    // Check second message
    assert_eq!(messages[1].role, "tool");
    assert_eq!(messages[1].tool_call_id, Some("call-2".to_string()));
    assert_eq!(messages[1].content, Some("Result 2".to_string()));
}

/// # Test Tool Call History
/// 
/// Tests tool call history tracking.
#[tokio::test]
async fn test_tool_call_history() {
    let mut registry = FunctionRegistry::new();
    registry.register_function(Box::new(CalculatorExecutor));
    
    let executor = ToolCallExecutor::new(Arc::new(registry));
    
    // Execute some tool calls
    let tool_calls = vec![
        ToolCall {
            id: "history-call-1".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "add".to_string(),
                arguments: json!({"a": 1, "b": 2}).to_string(),
            },
        },
        ToolCall {
            id: "history-call-2".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "multiply".to_string(),
                arguments: json!({"a": 3, "b": 4}).to_string(),
            },
        },
    ];
    
    let _results = executor.execute_tool_calls(tool_calls).await.unwrap();
    
    // Check history
    let history = executor.get_tool_call_history();
    assert_eq!(history.len(), 2);
    
    // Check first history entry
    assert_eq!(history[0].tool_call_id, "history-call-1");
    assert_eq!(history[0].function_name, "add");
    assert!(history[0].success);
    assert!(history[0].error_message.is_none());
    
    // Check second history entry
    assert_eq!(history[1].tool_call_id, "history-call-2");
    assert_eq!(history[1].function_name, "multiply");
    assert!(history[1].success);
    assert!(history[1].error_message.is_none());
    
    // Test clearing history
    executor.clear_tool_call_history();
    let cleared_history = executor.get_tool_call_history();
    assert_eq!(cleared_history.len(), 0);
}

/// # Test Complete Tool Use Workflow
/// 
/// Tests a complete workflow with tool calls and responses.
#[tokio::test]
async fn test_complete_tool_use_workflow() {
    // Setup
    let mut registry = FunctionRegistry::new();
    registry.register_function(Box::new(CalculatorExecutor));
    registry.register_function(Box::new(WeatherExecutor));
    
    let builder = ToolCallMessageBuilder::new(Arc::new(registry));
    
    // Create a conversation with tool calls
    let messages = vec![
        Message {
            role: "user".to_string(),
            content: Some("What's 5 * 6 and what's the weather like?".to_string()),
            name: None,
            function_call: None,
            tool_call_id: None,
            tool_calls: None,
        },
        Message {
            role: "assistant".to_string(),
            content: Some("I'll calculate that and get the weather for you.".to_string()),
            name: None,
            function_call: None,
            tool_call_id: None,
            tool_calls: Some(vec![
                ToolCall {
                    id: "calc-call".to_string(),
                    tool_type: "function".to_string(),
                    function: FunctionCall {
                        name: "multiply".to_string(),
                        arguments: json!({"a": 5, "b": 6}).to_string(),
                    },
                },
                ToolCall {
                    id: "weather-call".to_string(),
                    tool_type: "function".to_string(),
                    function: FunctionCall {
                        name: "get_weather".to_string(),
                        arguments: json!({"location": "New York"}).to_string(),
                    },
                },
            ]),
        },
    ];
    
    let tools = vec![
        Tool {
            tool_type: "function".to_string(),
            function: Some(FunctionDefinition {
                name: "multiply".to_string(),
                description: Some("Multiply two numbers".to_string()),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number"},
                        "b": {"type": "number"}
                    },
                    "required": ["a", "b"]
                })),
            }),
        },
        Tool {
            tool_type: "function".to_string(),
            function: Some(FunctionDefinition {
                name: "get_weather".to_string(),
                description: Some("Get weather information".to_string()),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    },
                    "required": ["location"]
                })),
            }),
        },
    ];
    
    // Build the conversation
    let conversation = builder.build_tool_use_conversation(
        messages,
        Some(tools),
        Some(ToolChoice::Auto),
    ).await.unwrap();
    
    // Should have original messages plus two tool results
    assert_eq!(conversation.len(), 4);
    
    // Check tool result messages
    let calc_result = &conversation[2];
    assert_eq!(calc_result.role, "tool");
    assert_eq!(calc_result.tool_call_id, Some("calc-call".to_string()));
    
    let weather_result = &conversation[3];
    assert_eq!(weather_result.role, "tool");
    assert_eq!(weather_result.tool_call_id, Some("weather-call".to_string()));
    
    // Verify calculation result
    let calc_content = calc_result.content.as_ref().unwrap();
    let calc_value: serde_json::Value = serde_json::from_str(calc_content).unwrap();
    assert_eq!(calc_value["result"], 30);
    
    // Verify weather result
    let weather_content = weather_result.content.as_ref().unwrap();
    let weather_value: serde_json::Value = serde_json::from_str(weather_content).unwrap();
    assert!(weather_value["location"].is_string());
    assert!(weather_value["temperature"].is_number());
}

/// # Integration Test Suite
/// 
/// Runs a comprehensive integration test suite for tool support.
#[tokio::test]
async fn test_tool_support_integration_suite() {
    println!("🚀 Starting comprehensive tool support integration test suite");
    
    // Test all components
    test_tool_role_functionality();
    test_tool_use_message_creation();
    test_message_conversion();
    test_tool_call_executor().await;
    test_tool_call_executor_error_handling().await;
    test_tool_call_validator();
    test_tool_choice_validation();
    test_tool_call_message_builder().await;
    test_tool_call_response_formatter();
    test_tool_call_history().await;
    test_complete_tool_use_workflow().await;
    
    println!("✅ Comprehensive tool support integration test suite completed");
}
