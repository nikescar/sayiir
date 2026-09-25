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

pub mod builder;
mod compile;
pub mod deps;
pub mod detect;
pub mod error;
pub mod extract;
pub mod extract_clean;
mod execute;
mod import;
pub mod parse;
mod run;
mod runtime;
pub mod scan;

pub use builder::{build_openflow_spec, TaskMetadata, Language};
pub use compile::{CachedModule, cleanup_stale_cache, compile_module};
pub use deps::{parse_cargo_deps, parse_python_deps, parse_node_deps};
pub use detect::{detect_language, ProjectLanguage};
pub use error::{ExportError, OpenFlowError, Result};
pub use execute::{execute_task, execute_task_with_timeout};
pub use extract::TaskSource;
pub use import::import_workflow;
pub use run::{run_workflow, run_workflow_with_timeout};
pub use runtime::check_runtimes;

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
        /// External dependencies (package_name -> version) - optional
        #[serde(skip_serializing_if = "Option::is_none")]
        dependencies: Option<serde_json::Map<String, serde_json::Value>>,
    },
}

/// Import preview summary
#[derive(Debug, Clone)]
pub struct ImportPreview {
    /// Workflow summary
    pub summary: String,
    /// Total modules count
    pub total_modules: usize,
    /// Module details
    pub modules: Vec<ModulePreview>,
}

/// Module preview details
#[derive(Debug, Clone)]
pub struct ModulePreview {
    /// Module ID
    pub id: String,
    /// Language (if embedded code)
    pub language: Option<String>,
    /// Entry point (if embedded code)
    pub entry_point: Option<String>,
    /// Code size in lines (if embedded code)
    pub code_lines: Option<usize>,
    /// Dependencies count (if embedded code with dependencies)
    pub dependencies_count: usize,
}

impl std::fmt::Display for ImportPreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Workflow: {}", self.summary)?;
        writeln!(f, "Modules: {}", self.total_modules)?;
        writeln!(f)?;
        for module in &self.modules {
            write!(f, "  - {}", module.id)?;
            if let Some(lang) = &module.language {
                write!(f, " ({})", lang)?;
                if let Some(lines) = module.code_lines {
                    write!(f, " - {} lines", lines)?;
                }
                if module.dependencies_count > 0 {
                    write!(f, " - {} deps", module.dependencies_count)?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Preview import from OpenFlow JSON
///
/// Shows a summary of what would be imported without actually parsing embedded code.
pub fn preview_import_json(json: &str) -> Result<ImportPreview> {
    let spec: OpenFlowSpec = serde_json::from_str(json)?;
    Ok(preview_spec(&spec))
}

/// Preview import from Mermaid markdown
///
/// Shows a summary of what would be imported without actually parsing embedded code.
pub fn preview_import_mermaid(markdown: &str) -> Result<ImportPreview> {
    let spec = import_mermaid(markdown)?;
    Ok(preview_spec(&spec))
}

fn preview_spec(spec: &OpenFlowSpec) -> ImportPreview {
    let modules: Vec<ModulePreview> = spec
        .value
        .modules
        .iter()
        .map(|module| {
            let OpenFlowModuleValue::Script {
                language,
                entry_point,
                code,
                dependencies,
                ..
            } = &module.value;

            let code_lines = code.as_ref().map(|c| c.lines().count());
            let dependencies_count = dependencies.as_ref().map(|d| d.len()).unwrap_or(0);

            ModulePreview {
                id: module.id.clone(),
                language: language.clone(),
                entry_point: entry_point.clone(),
                code_lines,
                dependencies_count,
            }
        })
        .collect();

    ImportPreview {
        summary: spec.summary.clone(),
        total_modules: modules.len(),
        modules,
    }
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
        return Err(OpenFlowError::InvalidWorkflow(
            "summary cannot be empty".into(),
        ));
    }

    // Validate module IDs are unique
    let mut seen_ids = std::collections::HashSet::new();
    for module in &spec.value.modules {
        if !seen_ids.insert(&module.id) {
            return Err(OpenFlowError::InvalidWorkflow(format!(
                "duplicate module ID: {}",
                module.id
            )));
        }

        // Validate embedded code fields (if present, all three must be present)
        let OpenFlowModuleValue::Script {
            language,
            entry_point,
            code,
            ..
        } = &module.value;

        let has_language = language.is_some();
        let has_entry_point = entry_point.is_some();
        let has_code = code.is_some();

        if has_language || has_entry_point || has_code {
            if !has_language {
                return Err(OpenFlowError::InvalidWorkflow(format!(
                    "module '{}': language required when code is embedded",
                    module.id
                )));
            }
            if !has_entry_point {
                return Err(OpenFlowError::InvalidWorkflow(format!(
                    "module '{}': entry_point required when code is embedded",
                    module.id
                )));
            }
            if !has_code {
                return Err(OpenFlowError::InvalidWorkflow(format!(
                    "module '{}': code required when language is specified",
                    module.id
                )));
            }

            // Validate language is supported
            let lang = language.as_ref().unwrap();
            if !matches!(lang.as_str(), "rust" | "node" | "python") {
                return Err(OpenFlowError::Unsupported(format!(
                    "language '{}' not supported (use rust, node, or python)",
                    lang
                )));
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
    use std::collections::HashMap;

    // Parse flowchart nodes first (preserving order)
    let mut modules: Vec<OpenFlowModule> = Vec::new();
    let mut module_indices: HashMap<String, usize> = HashMap::new();

    let mut in_code_block = false;
    for line in markdown.lines() {
        let trimmed = line.trim();

        // Track code block state
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        // Skip non-flowchart lines and code blocks
        if trimmed.starts_with("flowchart")
            || trimmed.is_empty()
            || trimmed.starts_with("%%%")
            || in_code_block
        {
            continue;
        }

        // Parse "A[Task Name]" or "A --> B" syntax
        if let Some(node) = parse_mermaid_node(trimmed) {
            let id = node.id.clone();
            if !module_indices.contains_key(&id) {
                module_indices.insert(id, modules.len());
                modules.push(node);
            }
        }
    }

    // Parse code blocks and metadata
    let lines: Vec<&str> = markdown.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Look for metadata comment: "%%% task_id (language)"
        if line.starts_with("%%%") && line.contains('(') && line.contains(')') {
            if let Some((task_id, lang)) = parse_task_metadata(line) {
                // Parse entry point
                i += 1;
                let entry_point = if i < lines.len() && lines[i].trim().starts_with("%%% Entry:") {
                    Some(
                        lines[i]
                            .trim()
                            .strip_prefix("%%% Entry:")
                            .unwrap()
                            .trim()
                            .to_string(),
                    )
                } else {
                    None
                };

                // Parse dependencies
                i += 1;
                let dependencies = if i < lines.len()
                    && lines[i].trim().starts_with("%%% Dependencies:")
                {
                    let deps_str = lines[i]
                        .trim()
                        .strip_prefix("%%% Dependencies:")
                        .unwrap()
                        .trim();
                    if deps_str == "{}" {
                        None
                    } else {
                        match serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(
                            deps_str,
                        ) {
                            Ok(deps) => Some(deps),
                            Err(_) => None,
                        }
                    }
                } else {
                    None
                };

                // Parse code block
                i += 1;
                let code = if i < lines.len() && lines[i].trim().starts_with("```") {
                    let mut code_lines = Vec::new();
                    i += 1; // Skip opening fence

                    while i < lines.len() {
                        let code_line = lines[i];
                        if code_line.trim() == "```" {
                            break;
                        }
                        code_lines.push(code_line);
                        i += 1;
                    }

                    Some(code_lines.join("\n"))
                } else {
                    None
                };

                // Update module if it exists
                if let Some(&idx) = module_indices.get(&task_id) {
                    modules[idx].value = OpenFlowModuleValue::Script {
                        path: task_id.clone(),
                        language: Some(lang),
                        entry_point,
                        code,
                        dependencies,
                    };
                }
            }
        }

        i += 1;
    }

    Ok(OpenFlowSpec {
        summary: "Imported from Mermaid".to_string(),
        value: OpenFlowValue { modules },
    })
}

fn parse_mermaid_node(line: &str) -> Option<OpenFlowModule> {
    // Skip arrow lines
    if line.contains("-->") {
        return None;
    }

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
                    dependencies: None,
                },
            });
        }
    }
    None
}

fn parse_task_metadata(line: &str) -> Option<(String, String)> {
    // Parse "%%% task_id (language)"
    let line = line.trim().strip_prefix("%%%")?.trim();

    if let Some(paren_start) = line.find('(') {
        if let Some(paren_end) = line.find(')') {
            let task_id = line[..paren_start].trim().to_string();
            let language = line[paren_start + 1..paren_end].trim().to_string();
            return Some((task_id, language));
        }
    }

    None
}

/// Export OpenFlowSpec to Mermaid markdown
pub fn export_mermaid(spec: &OpenFlowSpec) -> Result<String> {
    let mut mermaid = String::from("flowchart TD\n");

    // Generate flowchart nodes
    for module in &spec.value.modules {
        mermaid.push_str(&format!("    {}[{}]\n", module.id, module.id));
    }

    // Add arrows between sequential modules
    for i in 0..spec.value.modules.len().saturating_sub(1) {
        let current = &spec.value.modules[i];
        let next = &spec.value.modules[i + 1];
        mermaid.push_str(&format!("    {} --> {}\n", current.id, next.id));
    }

    // Append code blocks for modules with embedded code
    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            language: Some(lang),
            entry_point: Some(entry),
            code: Some(code),
            dependencies,
            ..
        } = &module.value
        {
            mermaid.push_str("\n");

            // Metadata comments
            mermaid.push_str(&format!("%%% {} ({})\n", module.id, lang));
            mermaid.push_str(&format!("%%% Entry: {}\n", entry));

            // Dependencies as JSON
            let deps_json = if let Some(deps) = dependencies {
                serde_json::to_string(deps)?
            } else {
                "{}".to_string()
            };
            mermaid.push_str(&format!("%%% Dependencies: {}\n", deps_json));

            // Code block with language tag
            mermaid.push_str(&format!("```{}\n", lang));
            mermaid.push_str(code);
            if !code.ends_with('\n') {
                mermaid.push('\n');
            }
            mermaid.push_str("```\n");
        }
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
                modules: vec![OpenFlowModule {
                    id: "task1".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "test_script".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                }],
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
                            dependencies: None,
                        },
                    },
                    OpenFlowModule {
                        id: "B".to_string(),
                        value: OpenFlowModuleValue::Script {
                            path: "b".to_string(),
                            language: None,
                            entry_point: None,
                            code: None,
                            dependencies: None,
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("summary cannot be empty")
        );
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

        if let OpenFlowModuleValue::Script {
            language,
            entry_point,
            code,
            ..
        } = &module.value
        {
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
                        dependencies: None,
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
    fn test_runtime_detection() {
        use crate::runtime::check_runtimes;

        let spec = OpenFlowSpec {
            summary: "Test".to_string(),
            value: OpenFlowValue {
                modules: vec![OpenFlowModule {
                    id: "task1".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "task1".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some("fn run() {}".to_string()),
                        dependencies: None,
                    },
                }],
            },
        };

        // This test assumes cargo is installed (required to build sayiir-openflow itself)
        let result = check_runtimes(&spec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_runtime_detection_missing_runtime() {
        use crate::runtime::check_runtimes;

        let spec = OpenFlowSpec {
            summary: "Test".to_string(),
            value: OpenFlowValue {
                modules: vec![OpenFlowModule {
                    id: "task1".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "task1".to_string(),
                        language: Some("nonexistent_language_xyz".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some("code".to_string()),
                        dependencies: None,
                    },
                }],
            },
        };

        let result = check_runtimes(&spec);
        assert!(result.is_err());
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("language required")
        );

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
                modules: vec![OpenFlowModule {
                    id: "only".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "single".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                }],
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
                modules: vec![OpenFlowModule {
                    id: "task1".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "test".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                }],
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
