//! Brimstone JavaScript execution for dependency-free workflows

use crate::{OpenFlowError, OpenFlowSpec, Result};
use brimstone_core::Context;
use serde_json::Value;

/// Validate that workflow is pure JavaScript with no external dependencies
pub fn validate_pure_javascript(spec: &OpenFlowSpec) -> Result<()> {
    for module in &spec.value.modules {
        // Check language
        let language = module
            .value
            .as_script()
            .and_then(|s| s.language.as_deref())
            .ok_or_else(|| {
                OpenFlowError::InvalidWorkflow(format!(
                    "module '{}': missing language field",
                    module.id
                ))
            })?;

        if language != "node" {
            return Err(OpenFlowError::InvalidWorkflow(format!(
                "Run command requires pure JavaScript workflows (found language: {})",
                language
            )));
        }

        // Check for external dependencies
        if let Some(script) = module.value.as_script() {
            if let Some(deps) = &script.dependencies {
                if !deps.is_empty() {
                    return Err(OpenFlowError::InvalidWorkflow(format!(
                        "Run command does not support external dependencies (module '{}' has {} dependencies)",
                        module.id,
                        deps.len()
                    )));
                }
            }
        }
    }
    Ok(())
}

/// Execute pure JavaScript workflow using Brimstone interpreter
pub fn run_workflow(spec: &OpenFlowSpec, input: Value) -> Result<Value> {
    // Validate first
    validate_pure_javascript(spec)?;

    // Initialize Brimstone context
    let mut context = Context::new();
    let mut current_input = input;

    for module in &spec.value.modules {
        let script = module.value.as_script().ok_or_else(|| {
            OpenFlowError::InvalidWorkflow(format!("module '{}': not a script", module.id))
        })?;

        let code = script.code.as_ref().ok_or_else(|| {
            OpenFlowError::InvalidWorkflow(format!("module '{}': missing code", module.id))
        })?;

        let entry_point = script.entry_point.as_ref().ok_or_else(|| {
            OpenFlowError::InvalidWorkflow(format!("module '{}': missing entry_point", module.id))
        })?;

        // Wrap task code with input/output handling
        let input_json = serde_json::to_string(&current_input)
            .map_err(|e| OpenFlowError::InvalidWorkflow(format!("failed to serialize input: {}", e)))?;

        let wrapped_code = format!(
            r#"
{code}

// Execute task and return JSON result
(function() {{
    const input = {input_json};
    const result = {entry_point}(input);
    return JSON.stringify(result);
}})()
"#
        );

        // Execute with Brimstone
        let result = context.eval(&wrapped_code).map_err(|e| {
            OpenFlowError::ExecutionError(format!("Brimstone execution failed in module '{}': {:?}", module.id, e))
        })?;

        // Parse result as JSON
        let result_str = result.to_string();
        current_input = serde_json::from_str(&result_str).map_err(|e| {
            OpenFlowError::ExecutionError(format!(
                "Failed to parse task output as JSON: {}. Output: {}",
                e, result_str
            ))
        })?;
    }

    Ok(current_input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OpenFlowModule, OpenFlowModuleValue, OpenFlowValue};
    use serde_json::json;

    fn create_test_spec(language: &str, has_deps: bool) -> OpenFlowSpec {
        let dependencies = if has_deps {
            Some(serde_json::Map::from_iter(vec![(
                "lodash".to_string(),
                json!("4.17.21"),
            )]))
        } else {
            None
        };

        OpenFlowSpec {
            summary: "test".to_string(),
            value: OpenFlowValue {
                modules: vec![OpenFlowModule {
                    id: "task1".to_string(),
                    value: OpenFlowModuleValue::Script {
                        language: Some(language.to_string()),
                        code: Some("function task1(x) { return x; }".to_string()),
                        entry_point: Some("task1".to_string()),
                        dependencies,
                    },
                }],
            },
        }
    }

    #[test]
    fn test_validate_pure_javascript_success() {
        let spec = create_test_spec("node", false);
        assert!(validate_pure_javascript(&spec).is_ok());
    }

    #[test]
    fn test_validate_rejects_rust() {
        let spec = create_test_spec("rust", false);
        let result = validate_pure_javascript(&spec);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("pure JavaScript"));
    }

    #[test]
    fn test_validate_rejects_python() {
        let spec = create_test_spec("python", false);
        let result = validate_pure_javascript(&spec);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_rejects_dependencies() {
        let spec = create_test_spec("node", true);
        let result = validate_pure_javascript(&spec);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("external dependencies"));
    }

    #[test]
    fn test_run_simple_workflow() {
        let spec = OpenFlowSpec {
            summary: "test".to_string(),
            value: OpenFlowValue {
                modules: vec![OpenFlowModule {
                    id: "double".to_string(),
                    value: OpenFlowModuleValue::Script {
                        language: Some("node".to_string()),
                        code: Some("function double(x) { return x * 2; }".to_string()),
                        entry_point: Some("double".to_string()),
                        dependencies: None,
                    },
                }],
            },
        };

        let result = run_workflow(&spec, json!(5)).unwrap();
        assert_eq!(result, json!(10));
    }

    #[test]
    fn test_run_chained_workflow() {
        let spec = OpenFlowSpec {
            summary: "test".to_string(),
            value: OpenFlowValue {
                modules: vec![
                    OpenFlowModule {
                        id: "double".to_string(),
                        value: OpenFlowModuleValue::Script {
                            language: Some("node".to_string()),
                            code: Some("function double(x) { return x * 2; }".to_string()),
                            entry_point: Some("double".to_string()),
                            dependencies: None,
                        },
                    },
                    OpenFlowModule {
                        id: "add_ten".to_string(),
                        value: OpenFlowModuleValue::Script {
                            language: Some("node".to_string()),
                            code: Some("function addTen(x) { return x + 10; }".to_string()),
                            entry_point: Some("addTen".to_string()),
                            dependencies: None,
                        },
                    },
                ],
            },
        };

        // 5 → double → 10 → add_ten → 20
        let result = run_workflow(&spec, json!(5)).unwrap();
        assert_eq!(result, json!(20));
    }
}
