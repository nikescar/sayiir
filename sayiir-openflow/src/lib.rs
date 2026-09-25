//! OpenFlow JSON and Mermaid markdown import/export for Sayiir workflow engine.
//!
//! Supports importing and exporting workflows in OpenFlow JSON format (Windmill spec)
//! and Mermaid markdown flowcharts.
//!
//! # Features
//! - Import OpenFlow JSON to Sayiir workflows
//! - Export Sayiir workflows to OpenFlow JSON
//! - Import Mermaid markdown flowcharts
//! - Export workflows to Mermaid markdown
//!
//! # Example
//! ```no_run
//! use sayiir_openflow::{import_openflow_json, OpenFlowSpec};
//!
//! let json = r#"{"summary": "Example workflow", "value": {"modules": []}}"#;
//! // let workflow = import_openflow_json(json)?;
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs)]

use thiserror::Error;

/// OpenFlow import/export errors
#[derive(Error, Debug)]
pub enum OpenFlowError {
    /// JSON parsing error
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Invalid workflow structure
    #[error("Invalid workflow: {0}")]
    InvalidWorkflow(String),

    /// Unsupported feature
    #[error("Unsupported: {0}")]
    Unsupported(String),
}

/// Result type for OpenFlow operations
pub type Result<T> = std::result::Result<T, OpenFlowError>;

/// OpenFlow JSON specification (Windmill format)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenFlowSpec {
    /// Workflow summary/description
    pub summary: String,
    /// Workflow value (modules and flows)
    pub value: OpenFlowValue,
}

/// OpenFlow workflow value
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenFlowValue {
    /// Workflow modules (tasks)
    pub modules: Vec<OpenFlowModule>,
}

/// OpenFlow module (task)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenFlowModule {
    /// Module ID
    pub id: String,
    /// Module value
    pub value: OpenFlowModuleValue,
}

/// OpenFlow module value
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum OpenFlowModuleValue {
    /// Script execution
    #[serde(rename = "script")]
    Script {
        /// Script path/name
        path: String,
        /// Language (rust, node, python) - optional for backward compatibility
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<String>,
        /// Entry point function name - optional for backward compatibility
        #[serde(skip_serializing_if = "Option::is_none")]
        entry_point: Option<String>,
        /// Embedded source code - optional for backward compatibility
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
    },
}

/// Import OpenFlow JSON to Sayiir workflow
///
/// Parses OpenFlow JSON and returns the parsed specification.
/// Conversion to executable Sayiir workflow requires a TaskRegistry.
pub fn import_openflow_json(json: &str) -> Result<OpenFlowSpec> {
    let spec: OpenFlowSpec = serde_json::from_str(json)?;
    validate_spec(&spec)?;
    Ok(spec)
}

fn validate_spec(spec: &OpenFlowSpec) -> Result<()> {
    if spec.summary.is_empty() {
        return Err(OpenFlowError::InvalidWorkflow("summary cannot be empty".into()));
    }

    // Validate module IDs are unique
    let mut seen_ids = std::collections::HashSet::new();
    for module in &spec.value.modules {
        if !seen_ids.insert(&module.id) {
            return Err(OpenFlowError::InvalidWorkflow(
                format!("duplicate module ID: {}", module.id)
            ));
        }

        // Validate embedded code fields (if present, all three must be present)
        if let OpenFlowModuleValue::Script { language, entry_point, code, .. } = &module.value {
            let has_language = language.is_some();
            let has_entry_point = entry_point.is_some();
            let has_code = code.is_some();

            if has_language || has_entry_point || has_code {
                if !has_language {
                    return Err(OpenFlowError::InvalidWorkflow(
                        format!("module '{}': language required when code is embedded", module.id)
                    ));
                }
                if !has_entry_point {
                    return Err(OpenFlowError::InvalidWorkflow(
                        format!("module '{}': entry_point required when code is embedded", module.id)
                    ));
                }
                if !has_code {
                    return Err(OpenFlowError::InvalidWorkflow(
                        format!("module '{}': code required when language is specified", module.id)
                    ));
                }

                // Validate language is supported
                let lang = language.as_ref().unwrap();
                if !matches!(lang.as_str(), "rust" | "node" | "python") {
                    return Err(OpenFlowError::Unsupported(
                        format!("language '{}' not supported (use rust, node, or python)", lang)
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Export OpenFlowSpec to JSON
pub fn export_openflow_json(spec: &OpenFlowSpec) -> Result<String> {
    let json = serde_json::to_string_pretty(spec)?;
    Ok(json)
}

/// Import Mermaid markdown flowchart
///
/// Parses Mermaid flowchart syntax and converts to OpenFlowSpec.
/// Basic implementation - supports simple flowchart syntax.
pub fn import_mermaid(markdown: &str) -> Result<OpenFlowSpec> {
    // Simple parser for "flowchart TD" format
    let mut modules = Vec::new();

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("flowchart") || trimmed.is_empty() {
            continue;
        }

        // Parse "A[Task Name]" or "A --> B" syntax
        if let Some(node) = parse_mermaid_node(trimmed) {
            modules.push(node);
        }
    }

    Ok(OpenFlowSpec {
        summary: "Imported from Mermaid".to_string(),
        value: OpenFlowValue { modules },
    })
}

fn parse_mermaid_node(line: &str) -> Option<OpenFlowModule> {
    // Simple parser: "id[label]" format
    if let Some(bracket_pos) = line.find('[') {
        let id = line[..bracket_pos].trim();
        if let Some(end_bracket) = line.find(']') {
            let _label = &line[bracket_pos + 1..end_bracket];
            return Some(OpenFlowModule {
                id: id.to_string(),
                value: OpenFlowModuleValue::Script {
                    path: id.to_string(),
                    language: None,
                    entry_point: None,
                    code: None,
                },
            });
        }
    }
    None
}

/// Export OpenFlowSpec to Mermaid markdown
pub fn export_mermaid(spec: &OpenFlowSpec) -> Result<String> {
    let mut mermaid = String::from("flowchart TD\n");

    for module in &spec.value.modules {
        mermaid.push_str(&format!("    {}[{}]\n", module.id, module.id));
    }

    // Add arrows between sequential modules
    for i in 0..spec.value.modules.len().saturating_sub(1) {
        let current = &spec.value.modules[i];
        let next = &spec.value.modules[i + 1];
        mermaid.push_str(&format!("    {} --> {}\n", current.id, next.id));
    }

    Ok(mermaid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openflow_spec_parse() {
        let json = r#"{"summary": "Test", "value": {"modules": []}}"#;
        let spec = import_openflow_json(json).unwrap();
        assert_eq!(spec.summary, "Test");
        assert_eq!(spec.value.modules.len(), 0);
    }

    #[test]
    fn test_import_with_modules() {
        let json = r#"{
            "summary": "Test workflow",
            "value": {
                "modules": [
                    {
                        "id": "task1",
                        "value": {
                            "type": "script",
                            "path": "test_script"
                        }
                    }
                ]
            }
        }"#;
        let spec = import_openflow_json(json).unwrap();
        assert_eq!(spec.value.modules.len(), 1);
        assert_eq!(spec.value.modules[0].id, "task1");
    }

    #[test]
    fn test_duplicate_module_ids() {
        let json = r#"{
            "summary": "Test",
            "value": {
                "modules": [
                    {"id": "task1", "value": {"type": "script", "path": "a"}},
                    {"id": "task1", "value": {"type": "script", "path": "b"}}
                ]
            }
        }"#;
        let result = import_openflow_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_export_roundtrip() {
        let spec = OpenFlowSpec {
            summary: "Test workflow".to_string(),
            value: OpenFlowValue {
                modules: vec![
                    OpenFlowModule {
                        id: "task1".to_string(),
                        value: OpenFlowModuleValue::Script {
                            path: "test_script".to_string(),
                            language: None,
                            entry_point: None,
                            code: None,
                        },
                    },
                ],
            },
        };

        let json = export_openflow_json(&spec).unwrap();
        let imported = import_openflow_json(&json).unwrap();
        assert_eq!(imported.summary, spec.summary);
        assert_eq!(imported.value.modules.len(), 1);
    }

    #[test]
    fn test_mermaid_import() {
        let mermaid = r#"
flowchart TD
    A[Start]
    B[Process]
    A --> B
"#;
        let spec = import_mermaid(mermaid).unwrap();
        assert_eq!(spec.value.modules.len(), 2);
        assert_eq!(spec.value.modules[0].id, "A");
        assert_eq!(spec.value.modules[1].id, "B");
    }

    #[test]
    fn test_mermaid_export() {
        let spec = OpenFlowSpec {
            summary: "Test".to_string(),
            value: OpenFlowValue {
                modules: vec![
                    OpenFlowModule {
                        id: "A".to_string(),
                        value: OpenFlowModuleValue::Script {
                            path: "a".to_string(),
                            language: None,
                            entry_point: None,
                            code: None,
                        },
                    },
                    OpenFlowModule {
                        id: "B".to_string(),
                        value: OpenFlowModuleValue::Script {
                            path: "b".to_string(),
                            language: None,
                            entry_point: None,
                            code: None,
                        },
                    },
                ],
            },
        };

        let mermaid = export_mermaid(&spec).unwrap();
        assert!(mermaid.contains("flowchart TD"));
        assert!(mermaid.contains("A[A]"));
        assert!(mermaid.contains("B[B]"));
        assert!(mermaid.contains("A --> B"));
    }

    // Additional comprehensive tests for Task 2.6

    #[test]
    fn test_empty_summary_validation() {
        let json = r#"{"summary": "", "value": {"modules": []}}"#;
        let result = import_openflow_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("summary cannot be empty"));
    }

    #[test]
    fn test_invalid_json() {
        let json = r#"{"summary": "Test", "value": {"modules": [}"#; // Missing closing bracket
        let result = import_openflow_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_module_with_embedded_code() {
        let json = r#"{
            "summary": "Test",
            "value": {
                "modules": [{
                    "id": "task1",
                    "value": {
                        "type": "script",
                        "path": "task1",
                        "language": "rust",
                        "entry_point": "run",
                        "code": "fn run(input: serde_json::Value) -> Result<serde_json::Value, String> { Ok(input) }"
                    }
                }]
            }
        }"#;

        let spec = import_openflow_json(json).unwrap();
        let module = &spec.value.modules[0];

        if let OpenFlowModuleValue::Script { language, entry_point, code, .. } = &module.value {
            assert_eq!(language.as_ref().unwrap(), "rust");
            assert_eq!(entry_point.as_ref().unwrap(), "run");
            assert!(code.as_ref().unwrap().contains("fn run"));
        } else {
            panic!("Expected Script variant");
        }
    }

    #[test]
    fn test_export_module_with_embedded_code() {
        let spec = OpenFlowSpec {
            summary: "Test".to_string(),
            value: OpenFlowValue {
                modules: vec![OpenFlowModule {
                    id: "task1".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "task1".to_string(),
                        language: Some("python".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some("def run(input): return input".to_string()),
                    },
                }],
            },
        };

        let json = export_openflow_json(&spec).unwrap();
        assert!(json.contains("\"language\": \"python\""));
        assert!(json.contains("\"entry_point\": \"run\""));
        assert!(json.contains("def run(input)"));
    }

    #[test]
    fn test_validation_embedded_code_requires_all_fields() {
        // Missing language
        let json = r#"{
            "summary": "Test",
            "value": {
                "modules": [{
                    "id": "task1",
                    "value": {
                        "type": "script",
                        "path": "task1",
                        "entry_point": "run",
                        "code": "fn run() {}"
                    }
                }]
            }
        }"#;
        let result = import_openflow_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("language required"));

        // Unsupported language
        let json = r#"{
            "summary": "Test",
            "value": {
                "modules": [{
                    "id": "task1",
                    "value": {
                        "type": "script",
                        "path": "task1",
                        "language": "java",
                        "entry_point": "run",
                        "code": "public static void run() {}"
                    }
                }]
            }
        }"#;
        let result = import_openflow_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not supported"));
    }

    #[test]
    fn test_multiple_modules() {
        let json = r#"{
            "summary": "Multi-task workflow",
            "value": {
                "modules": [
                    {"id": "task1", "value": {"type": "script", "path": "step1"}},
                    {"id": "task2", "value": {"type": "script", "path": "step2"}},
                    {"id": "task3", "value": {"type": "script", "path": "step3"}}
                ]
            }
        }"#;
        let spec = import_openflow_json(json).unwrap();
        assert_eq!(spec.value.modules.len(), 3);
        assert_eq!(spec.value.modules[2].id, "task3");
    }

    #[test]
    fn test_mermaid_single_node() {
        let mermaid = "flowchart TD\n    A[Single Node]";
        let spec = import_mermaid(mermaid).unwrap();
        assert_eq!(spec.value.modules.len(), 1);
        assert_eq!(spec.value.modules[0].id, "A");
    }

    #[test]
    fn test_mermaid_empty() {
        let mermaid = "flowchart TD\n";
        let spec = import_mermaid(mermaid).unwrap();
        assert_eq!(spec.value.modules.len(), 0);
    }

    #[test]
    fn test_mermaid_with_comments() {
        let mermaid = r#"
flowchart TD
    %% This is a comment
    A[Start]
    B[End]
    A --> B
"#;
        let spec = import_mermaid(mermaid).unwrap();
        assert_eq!(spec.value.modules.len(), 2);
    }

    #[test]
    fn test_export_single_module() {
        let spec = OpenFlowSpec {
            summary: "Single task".to_string(),
            value: OpenFlowValue {
                modules: vec![
                    OpenFlowModule {
                        id: "only".to_string(),
                        value: OpenFlowModuleValue::Script {
                            path: "single".to_string(),
                            language: None,
                            entry_point: None,
                            code: None,
                        },
                    },
                ],
            },
        };

        let mermaid = export_mermaid(&spec).unwrap();
        assert!(mermaid.contains("only[only]"));
        assert!(!mermaid.contains("-->")); // No arrows for single node
    }

    #[test]
    fn test_export_empty_workflow() {
        let spec = OpenFlowSpec {
            summary: "Empty".to_string(),
            value: OpenFlowValue { modules: vec![] },
        };

        let mermaid = export_mermaid(&spec).unwrap();
        assert_eq!(mermaid, "flowchart TD\n");
    }

    #[test]
    fn test_json_pretty_formatting() {
        let spec = OpenFlowSpec {
            summary: "Test".to_string(),
            value: OpenFlowValue {
                modules: vec![
                    OpenFlowModule {
                        id: "task1".to_string(),
                        value: OpenFlowModuleValue::Script {
                            path: "test".to_string(),
                            language: None,
                            entry_point: None,
                            code: None,
                        },
                    },
                ],
            },
        };

        let json = export_openflow_json(&spec).unwrap();
        // Verify it's pretty-printed (contains newlines)
        assert!(json.contains('\n'));
        // Verify it can be re-imported
        let reimported = import_openflow_json(&json).unwrap();
        assert_eq!(reimported.summary, "Test");
    }
}
