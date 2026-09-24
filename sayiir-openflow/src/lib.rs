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
                        value: OpenFlowModuleValue::Script { path: "a".to_string() },
                    },
                    OpenFlowModule {
                        id: "B".to_string(),
                        value: OpenFlowModuleValue::Script { path: "b".to_string() },
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
}
