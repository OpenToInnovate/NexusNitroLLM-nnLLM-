//! # Tool Call Validation
//!
//! This module provides validation functionality for tool calls,
//! tool choices, and function parameters.

use crate::schemas::{ToolCall, ToolChoice, FunctionCall};
use serde_json::{Value, Map};
use super::{ToolError, registry::FunctionRegistry};

/// Tool call validator for validating function calls and tool choices
pub struct ToolCallValidator<'a> {
    /// Function registry for validation
    registry: &'a FunctionRegistry,
    /// Whether to enforce strict parameter validation
    strict_validation: bool,
}

impl<'a> ToolCallValidator<'a> {
    /// Create a new tool call validator
    pub fn new(registry: &'a FunctionRegistry) -> Self {
        Self {
            registry,
            strict_validation: true,
        }
    }

    /// Set strict validation mode
    pub fn set_strict_validation(&mut self, strict: bool) {
        self.strict_validation = strict;
    }

    /// Validate a tool choice
    pub fn validate_tool_choice(&self, tool_choice: &ToolChoice) -> Result<(), ToolError> {
        match tool_choice {
            ToolChoice::None => Ok(()),
            ToolChoice::Auto => {
                if self.registry.is_empty() {
                    Err(ToolError::InvalidToolChoice {
                        message: "Auto tool choice specified but no functions available".to_string(),
                    })
                } else {
                    Ok(())
                }
            }
            ToolChoice::Required => {
                let required_functions = self.registry.required_functions();
                if required_functions.is_empty() {
                    Err(ToolError::InvalidToolChoice {
                        message: "Required tool choice specified but no required functions available".to_string(),
                    })
                } else {
                    Ok(())
                }
            }
            ToolChoice::Specific { tool_type, function } => {
                if tool_type == "function" {
                    if self.registry.contains(&function.name) {
                        Ok(())
                    } else {
                        Err(ToolError::InvalidToolChoice {
                            message: format!("Specified function '{}' not found in registry", function.name),
                        })
                    }
                } else {
                    Err(ToolError::InvalidToolChoice {
                        message: format!("Unsupported tool type: {}", tool_type),
                    })
                }
            }
        }
    }

    /// Validate a tool call
    pub fn validate_tool_call(&self, tool_call: &ToolCall) -> Result<(), ToolError> {
        self.validate_function_call(&tool_call.function)
    }

    /// Validate a function call
    pub fn validate_function_call(&self, function_call: &FunctionCall) -> Result<(), ToolError> {
        let function_name = &function_call.name;

        // Check if function exists in registry
        let function_def = self.registry.get(function_name)
            .ok_or_else(|| ToolError::FunctionNotFound {
                name: function_name.clone(),
            })?;

        // Validate parameters if strict validation is enabled
        if self.strict_validation {
            if let Some(ref schema) = function_def.parameters {
                let arguments: serde_json::Value = serde_json::from_str(&function_call.arguments)
                    .map_err(|e| ToolError::ValidationFailed {
                        message: format!("Invalid JSON arguments: {}", e),
                    })?;
                self.validate_parameters(&arguments, schema)?;
            }
        }

        Ok(())
    }

    /// Validate multiple tool calls
    pub fn validate_tool_calls(&self, tool_calls: &[ToolCall]) -> Result<(), ToolError> {
        for tool_call in tool_calls {
            self.validate_tool_call(tool_call)?;
        }
        Ok(())
    }

    /// Validate function parameters against schema
    fn validate_parameters(&self, arguments: &Value, schema: &Value) -> Result<(), ToolError> {
        // Basic JSON schema validation
        match schema {
            Value::Object(schema_obj) => {
                if let Some(Value::Object(props)) = schema_obj.get("properties") {
                    self.validate_object_properties(arguments, props)?;
                }

                if let Some(Value::Array(required_props)) = schema_obj.get("required") {
                    self.validate_required_properties(arguments, required_props)?;
                }
            }
            _ => {
                return Err(ToolError::ValidationFailed {
                    message: "Invalid parameter schema format".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate object properties
    fn validate_object_properties(
        &self,
        arguments: &Value,
        properties: &Map<String, Value>,
    ) -> Result<(), ToolError> {
        let Value::Object(args_obj) = arguments else {
            return Err(ToolError::ValidationFailed {
                message: "Arguments must be an object when properties are specified".to_string(),
            });
        };

        // Check each provided argument against schema
        for (arg_name, arg_value) in args_obj {
            if let Some(prop_schema) = properties.get(arg_name) {
                self.validate_property_type(arg_name, arg_value, prop_schema)?;
            } else if self.strict_validation {
                return Err(ToolError::ValidationFailed {
                    message: format!("Unknown property: {}", arg_name),
                });
            }
        }

        Ok(())
    }

    /// Validate required properties
    fn validate_required_properties(
        &self,
        arguments: &Value,
        required_props: &[Value],
    ) -> Result<(), ToolError> {
        let Value::Object(args_obj) = arguments else {
            return Err(ToolError::ValidationFailed {
                message: "Arguments must be an object when required properties are specified".to_string(),
            });
        };

        for required_prop in required_props {
            if let Value::String(prop_name) = required_prop {
                if !args_obj.contains_key(prop_name) {
                    return Err(ToolError::ValidationFailed {
                        message: format!("Missing required property: {}", prop_name),
                    });
                }
            }
        }

        Ok(())
    }

    /// Validate property type
    fn validate_property_type(
        &self,
        _name: &str,
        value: &Value,
        schema: &Value,
    ) -> Result<(), ToolError> {
        if let Value::Object(schema_obj) = schema {
            if let Some(Value::String(expected_type)) = schema_obj.get("type") {
                let actual_type = match value {
                    Value::String(_) => "string",
                    Value::Number(_) => "number",
                    Value::Bool(_) => "boolean",
                    Value::Array(_) => "array",
                    Value::Object(_) => "object",
                    Value::Null => "null",
                };

                if expected_type != actual_type && expected_type != "any" {
                    return Err(ToolError::ValidationFailed {
                        message: format!(
                            "Type mismatch: expected {}, got {}",
                            expected_type, actual_type
                        ),
                    });
                }
            }
        }

        Ok(())
    }

    /// Get the function registry
    pub fn registry(&self) -> &FunctionRegistry {
        self.registry
    }
}

/// Validation utilities
pub mod utils {
    use super::*;

    /// Check if a tool choice is valid for a given registry
    pub fn is_valid_tool_choice(
        tool_choice: &ToolChoice,
        registry: &FunctionRegistry,
    ) -> bool {
        let validator = ToolCallValidator::new(&registry);
        validator.validate_tool_choice(tool_choice).is_ok()
    }

    /// Check if a tool call is valid for a given registry
    pub fn is_valid_tool_call(
        tool_call: &ToolCall,
        registry: &FunctionRegistry,
    ) -> bool {
        let validator = ToolCallValidator::new(&registry);
        validator.validate_tool_call(tool_call).is_ok()
    }

    /// Validate and extract function arguments
    pub fn extract_validated_arguments(
        function_call: &FunctionCall,
        registry: &FunctionRegistry,
    ) -> Result<Value, ToolError> {
        let validator = ToolCallValidator::new(&registry);
        validator.validate_function_call(function_call)?;

        Ok(serde_json::from_str(&function_call.arguments).unwrap_or(Value::Null))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::registry::{FunctionRegistry, FunctionDefinition};
    use serde_json::json;

    fn create_test_registry() -> FunctionRegistry {
        let mut registry = FunctionRegistry::new();

        let func = FunctionDefinition::new("test_function".to_string())
            .with_parameters(json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "age": {"type": "number"}
                },
                "required": ["name"]
            }));

        registry.register(func);
        registry
    }

    #[test]
    fn test_valid_tool_choice_validation() {
        let registry = create_test_registry();
        let validator = ToolCallValidator::new(&registry);

        assert!(validator.validate_tool_choice(&ToolChoice::None).is_ok());
        assert!(validator.validate_tool_choice(&ToolChoice::Auto).is_ok());
        assert!(validator.validate_tool_choice(&ToolChoice::Specific {
            tool_type: "function".to_string(),
            function: crate::schemas::FunctionChoice { name: "test_function".to_string() }
        }).is_ok());
    }

    #[test]
    fn test_invalid_tool_choice_validation() {
        let registry = create_test_registry();
        let validator = ToolCallValidator::new(&registry);

        let result = validator.validate_tool_choice(&ToolChoice::Specific {
            tool_type: "function".to_string(),
            function: crate::schemas::FunctionChoice { name: "nonexistent_function".to_string() }
        });
        assert!(matches!(result, Err(ToolError::InvalidToolChoice { .. })));
    }

    #[test]
    fn test_valid_function_call_validation() {
        let registry = create_test_registry();
        let validator = ToolCallValidator::new(&registry);

        let function_call = FunctionCall {
            name: "test_function".to_string(),
            arguments: serde_json::to_string(&json!({
                "name": "John",
                "age": 30
            })).unwrap(),
        };

        assert!(validator.validate_function_call(&function_call).is_ok());
    }

    #[test]
    fn test_missing_required_parameter() {
        let registry = create_test_registry();
        let validator = ToolCallValidator::new(&registry);

        let function_call = FunctionCall {
            name: "test_function".to_string(),
            arguments: serde_json::to_string(&json!({
                "age": 30
                // missing required "name" parameter
            })).unwrap(),
        };

        let result = validator.validate_function_call(&function_call);
        assert!(matches!(result, Err(ToolError::ValidationFailed { .. })));
    }

    #[test]
    fn test_type_mismatch_validation() {
        let registry = create_test_registry();
        let validator = ToolCallValidator::new(&registry);

        let function_call = FunctionCall {
            name: "test_function".to_string(),
            arguments: serde_json::to_string(&json!({
                "name": "John",
                "age": "thirty" // should be number, not string
            })).unwrap(),
        };

        let result = validator.validate_function_call(&function_call);
        assert!(matches!(result, Err(ToolError::ValidationFailed { .. })));
    }

    #[test]
    fn test_non_strict_validation() {
        let registry = create_test_registry();
        let mut validator = ToolCallValidator::new(&registry);
        validator.set_strict_validation(false);

        let function_call = FunctionCall {
            name: "test_function".to_string(),
            arguments: serde_json::to_string(&json!({
                "name": "John",
                "extra_field": "allowed in non-strict mode"
            })).unwrap(),
        };

        assert!(validator.validate_function_call(&function_call).is_ok());
    }

    #[test]
    fn test_utility_functions() {
        let registry = create_test_registry();

        let valid_choice = ToolChoice::Specific {
            tool_type: "function".to_string(),
            function: crate::schemas::FunctionChoice { name: "test_function".to_string() }
        };
        assert!(utils::is_valid_tool_choice(&valid_choice, &registry));

        let invalid_choice = ToolChoice::Specific {
            tool_type: "function".to_string(),
            function: crate::schemas::FunctionChoice { name: "nonexistent".to_string() }
        };
        assert!(!utils::is_valid_tool_choice(&invalid_choice, &registry));
    }
}