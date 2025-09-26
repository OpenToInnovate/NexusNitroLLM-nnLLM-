//! # Function Registry
//!
//! This module provides function registration and management functionality,
//! consolidating the function discovery and definition logic.

use crate::schemas::{Tool, FunctionDefinition as SchemaFunctionDefinition};
use serde_json::Value;
use std::collections::HashMap;

/// Function definition for registered functions
#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    /// Function name
    pub name: String,
    /// Function description
    pub description: Option<String>,
    /// Function parameters schema
    pub parameters: Option<Value>,
    /// Whether the function is required
    pub required: bool,
}

impl FunctionDefinition {
    /// Create a new function definition
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            parameters: None,
            required: false,
        }
    }

    /// Set function description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set function parameters schema
    pub fn with_parameters(mut self, parameters: Value) -> Self {
        self.parameters = Some(parameters);
        self
    }

    /// Mark function as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Convert to Tool schema
    pub fn to_tool(&self) -> Tool {
        Tool {
            tool_type: "function".to_string(),
            function: SchemaFunctionDefinition {
                name: self.name.clone(),
                description: self.description.clone(),
                parameters: self.parameters.clone(),
            },
        }
    }
}

/// Function registry for managing available functions
#[derive(Debug, Default)]
pub struct FunctionRegistry {
    /// Registered functions
    functions: HashMap<String, FunctionDefinition>,
}

impl FunctionRegistry {
    /// Create a new function registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a function
    pub fn register(&mut self, function: FunctionDefinition) {
        self.functions.insert(function.name.clone(), function);
    }

    /// Get a function by name
    pub fn get(&self, name: &str) -> Option<&FunctionDefinition> {
        self.functions.get(name)
    }

    /// Check if a function is registered
    pub fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get all registered function names
    pub fn function_names(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }

    /// Get all registered functions as Tools
    pub fn as_tools(&self) -> Vec<Tool> {
        self.functions.values().map(|f| f.to_tool()).collect()
    }

    /// Get required functions only
    pub fn required_functions(&self) -> Vec<&FunctionDefinition> {
        self.functions.values().filter(|f| f.required).collect()
    }

    /// Clear all registered functions
    pub fn clear(&mut self) {
        self.functions.clear();
    }

    /// Get the number of registered functions
    pub fn len(&self) -> usize {
        self.functions.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
    }
}

/// Builder for creating function registries with predefined functions
pub struct FunctionRegistryBuilder {
    registry: FunctionRegistry,
}

impl FunctionRegistryBuilder {
    /// Create a new registry builder
    pub fn new() -> Self {
        Self {
            registry: FunctionRegistry::new(),
        }
    }

    /// Add a function to the registry
    pub fn add_function(mut self, function: FunctionDefinition) -> Self {
        self.registry.register(function);
        self
    }

    /// Add a simple function with just name and description
    pub fn add_simple_function(self, name: String, description: String) -> Self {
        let function = FunctionDefinition::new(name).with_description(description);
        self.add_function(function)
    }

    /// Build the registry
    pub fn build(self) -> FunctionRegistry {
        self.registry
    }
}

impl Default for FunctionRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_definition_creation() {
        let func = FunctionDefinition::new("test_function".to_string())
            .with_description("A test function".to_string())
            .required();

        assert_eq!(func.name, "test_function");
        assert_eq!(func.description, Some("A test function".to_string()));
        assert!(func.required);
    }

    #[test]
    fn test_function_registry_operations() {
        let mut registry = FunctionRegistry::new();

        let func = FunctionDefinition::new("test_func".to_string());
        registry.register(func);

        assert!(registry.contains("test_func"));
        assert!(!registry.contains("non_existent"));
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());

        let retrieved = registry.get("test_func");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_func");
    }

    #[test]
    fn test_function_registry_builder() {
        let registry = FunctionRegistryBuilder::new()
            .add_simple_function("func1".to_string(), "First function".to_string())
            .add_simple_function("func2".to_string(), "Second function".to_string())
            .build();

        assert_eq!(registry.len(), 2);
        assert!(registry.contains("func1"));
        assert!(registry.contains("func2"));
    }

    #[test]
    fn test_registry_as_tools() {
        let mut registry = FunctionRegistry::new();
        registry.register(FunctionDefinition::new("test_func".to_string()));

        let tools = registry.as_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].function.name, "test_func");
    }

    #[test]
    fn test_required_functions() {
        let mut registry = FunctionRegistry::new();

        registry.register(FunctionDefinition::new("optional_func".to_string()));
        registry.register(FunctionDefinition::new("required_func".to_string()).required());

        let required = registry.required_functions();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0].name, "required_func");
    }
}