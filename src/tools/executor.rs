//! # Tool Call Executor
//!
//! This module provides tool call execution functionality,
//! consolidating the function execution logic with proper error handling.

use crate::schemas::{ToolCall, FunctionCall};
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use super::{ToolError, ToolCallHistoryEntry, registry::FunctionRegistry};

/// Function execution result
pub type FunctionResult = Result<Value, ToolError>;

/// Async function handler type
pub type AsyncFunctionHandler = Box<
    dyn Fn(Value) -> Pin<Box<dyn Future<Output = FunctionResult> + Send>> + Send + Sync
>;

/// Tool call executor for managing and executing function calls
pub struct ToolCallExecutor {
    /// Registry of available functions
    registry: FunctionRegistry,
    /// Function handlers for execution
    handlers: HashMap<String, AsyncFunctionHandler>,
    /// Call history for tracking
    history: Vec<ToolCallHistoryEntry>,
    /// Maximum history size
    max_history_size: usize,
}

impl ToolCallExecutor {
    /// Create a new tool call executor
    pub fn new(registry: FunctionRegistry) -> Self {
        Self {
            registry,
            handlers: HashMap::new(),
            history: Vec::new(),
            max_history_size: 1000,
        }
    }

    /// Register a function handler
    pub fn register_handler<F, Fut>(&mut self, name: String, handler: F) -> Result<(), ToolError>
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = FunctionResult> + Send + 'static,
    {
        if !self.registry.contains(&name) {
            return Err(ToolError::FunctionNotFound { name });
        }

        let handler = Box::new(move |args: Value| {
            let fut = handler(args);
            Box::pin(fut) as Pin<Box<dyn Future<Output = FunctionResult> + Send>>
        });

        self.handlers.insert(name, handler);
        Ok(())
    }

    /// Execute a single tool call
    pub async fn execute_tool_call(&mut self, tool_call: ToolCall) -> Result<Value, ToolError> {
        let function_name = tool_call.function.name.clone();
        let arguments: serde_json::Value = serde_json::from_str(&tool_call.function.arguments).unwrap_or_default();

        // Create history entry
        let mut history_entry = ToolCallHistoryEntry::new(
            tool_call.id.clone(),
            function_name.clone(),
            arguments.clone(),
        );

        // Check if function is registered
        if !self.registry.contains(&function_name) {
            let error = ToolError::FunctionNotFound {
                name: function_name.clone(),
            };
            history_entry = history_entry.with_error(error.to_string());
            self.add_to_history(history_entry);
            return Err(error);
        }

        // Check if handler is available
        let handler = match self.handlers.get(&function_name) {
            Some(handler) => handler,
            None => {
                let error = ToolError::ExecutionFailed {
                    message: format!("No handler registered for function: {}", function_name),
                };
                history_entry = history_entry.with_error(error.to_string());
                self.add_to_history(history_entry);
                return Err(error);
            }
        };

        // Execute the function
        match handler(arguments).await {
            Ok(result) => {
                history_entry = history_entry.with_result(result.clone());
                self.add_to_history(history_entry);
                Ok(result)
            }
            Err(error) => {
                history_entry = history_entry.with_error(error.to_string());
                self.add_to_history(history_entry);
                Err(error)
            }
        }
    }

    /// Execute multiple tool calls
    pub async fn execute_tool_calls(
        &mut self,
        tool_calls: Vec<ToolCall>,
    ) -> Vec<Result<Value, ToolError>> {
        let mut results = Vec::with_capacity(tool_calls.len());

        for tool_call in tool_calls {
            let result = self.execute_tool_call(tool_call).await;
            results.push(result);
        }

        results
    }

    /// Get call history
    pub fn history(&self) -> &[ToolCallHistoryEntry] {
        &self.history
    }

    /// Clear call history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Set maximum history size
    pub fn set_max_history_size(&mut self, size: usize) {
        self.max_history_size = size;
        self.trim_history();
    }

    /// Get the function registry
    pub fn registry(&self) -> &FunctionRegistry {
        &self.registry
    }

    /// Check if a function handler is registered
    pub fn has_handler(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    /// Get list of available function handlers
    pub fn available_handlers(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }

    /// Add entry to history and manage size
    fn add_to_history(&mut self, entry: ToolCallHistoryEntry) {
        self.history.push(entry);
        self.trim_history();
    }

    /// Trim history to maximum size
    fn trim_history(&mut self) {
        if self.history.len() > self.max_history_size {
            let excess = self.history.len() - self.max_history_size;
            self.history.drain(0..excess);
        }
    }
}

/// Function executor for simpler function-based execution (legacy support)
pub struct FunctionExecutor {
    /// Underlying tool call executor
    tool_executor: ToolCallExecutor,
}

impl FunctionExecutor {
    /// Create a new function executor
    pub fn new(registry: FunctionRegistry) -> Self {
        Self {
            tool_executor: ToolCallExecutor::new(registry),
        }
    }

    /// Execute a function call
    pub async fn execute_function(&mut self, function_call: FunctionCall) -> Result<Value, ToolError> {
        // Convert function call to tool call
        let tool_call = ToolCall {
            id: format!("call_{}", uuid::Uuid::new_v4()),
            tool_type: "function".to_string(),
            function: function_call,
        };

        self.tool_executor.execute_tool_call(tool_call).await
    }

    /// Register a function handler
    pub fn register_handler<F, Fut>(&mut self, name: String, handler: F) -> Result<(), ToolError>
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = FunctionResult> + Send + 'static,
    {
        self.tool_executor.register_handler(name, handler)
    }

    /// Get the underlying tool executor
    pub fn tool_executor(&mut self) -> &mut ToolCallExecutor {
        &mut self.tool_executor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::registry::{FunctionRegistry, FunctionDefinition};

    async fn sample_function(args: Value) -> FunctionResult {
        Ok(serde_json::json!({
            "result": "success",
            "input": args
        }))
    }

    async fn failing_function(_args: Value) -> FunctionResult {
        Err(ToolError::ExecutionFailed {
            message: "Function failed intentionally".to_string(),
        })
    }

    #[tokio::test]
    async fn test_tool_call_executor_creation() {
        let registry = FunctionRegistry::new();
        let executor = ToolCallExecutor::new(registry);

        assert_eq!(executor.history().len(), 0);
        assert_eq!(executor.available_handlers().len(), 0);
    }

    #[tokio::test]
    async fn test_successful_tool_execution() {
        let mut registry = FunctionRegistry::new();
        registry.register(FunctionDefinition::new("test_func".to_string()));

        let mut executor = ToolCallExecutor::new(registry);
        executor.register_handler("test_func".to_string(), sample_function).unwrap();

        let tool_call = ToolCall {
            id: "call_123".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "test_func".to_string(),
                arguments: serde_json::to_string(&serde_json::json!({"key": "value"})).unwrap(),
            },
        };

        let result = executor.execute_tool_call(tool_call).await;
        assert!(result.is_ok());

        let history = executor.history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].function_name, "test_func");
        assert!(history[0].result.is_some());
        assert!(history[0].error.is_none());
    }

    #[tokio::test]
    async fn test_failed_tool_execution() {
        let mut registry = FunctionRegistry::new();
        registry.register(FunctionDefinition::new("failing_func".to_string()));

        let mut executor = ToolCallExecutor::new(registry);
        executor.register_handler("failing_func".to_string(), failing_function).unwrap();

        let tool_call = ToolCall {
            id: "call_456".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "failing_func".to_string(),
                arguments: serde_json::to_string(&serde_json::json!({})).unwrap(),
            },
        };

        let result = executor.execute_tool_call(tool_call).await;
        assert!(result.is_err());

        let history = executor.history();
        assert_eq!(history.len(), 1);
        assert!(history[0].result.is_none());
        assert!(history[0].error.is_some());
    }

    #[tokio::test]
    async fn test_unregistered_function() {
        let registry = FunctionRegistry::new();
        let mut executor = ToolCallExecutor::new(registry);

        let tool_call = ToolCall {
            id: "call_789".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "nonexistent_func".to_string(),
                arguments: serde_json::to_string(&serde_json::json!({})).unwrap(),
            },
        };

        let result = executor.execute_tool_call(tool_call).await;
        assert!(matches!(result, Err(ToolError::FunctionNotFound { .. })));
    }

    #[tokio::test]
    async fn test_history_management() {
        let mut registry = FunctionRegistry::new();
        registry.register(FunctionDefinition::new("test_func".to_string()));

        let mut executor = ToolCallExecutor::new(registry);
        executor.set_max_history_size(2);
        executor.register_handler("test_func".to_string(), sample_function).unwrap();

        // Execute multiple calls
        for i in 0..5 {
            let tool_call = ToolCall {
                id: format!("call_{}", i),
                tool_type: "function".to_string(),
                function: FunctionCall {
                    name: "test_func".to_string(),
                    arguments: serde_json::to_string(&serde_json::json!({"index": i})).unwrap(),
                },
            };
            let _ = executor.execute_tool_call(tool_call).await;
        }

        // History should be trimmed to max size
        assert_eq!(executor.history().len(), 2);
    }
}