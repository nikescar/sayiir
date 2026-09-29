//! Build OpenFlow specifications from extracted task metadata

use std::collections::HashMap;

/// Task metadata extracted from source code
#[derive(Debug, Clone)]
pub struct TaskMetadata {
    pub id: String,
    pub language: Language,
    pub source_code: String,
    pub entry_point: String,
    pub dependencies: HashMap<String, String>,
}

/// Programming language for task execution
#[derive(Debug, Clone, Copy)]
pub enum Language {
    Rust,
    Python,
    Node,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::Node => "node",
        }
    }
}

/// Build OpenFlow specification from task metadata
pub fn build_openflow_spec(
    workflow_name: String,
    tasks: Vec<TaskMetadata>,
) -> crate::OpenFlowSpec {
    let modules = tasks
        .into_iter()
        .map(|task| crate::OpenFlowModule {
            id: task.id,
            value: crate::OpenFlowModuleValue::Script {
                path: task.entry_point.clone(),
                language: Some(task.language.as_str().to_string()),
                entry_point: Some(task.entry_point),
                code: Some(task.source_code),
                dependencies: if task.dependencies.is_empty() {
                    None
                } else {
                    Some(
                        task.dependencies
                            .into_iter()
                            .map(|(k, v)| (k, serde_json::Value::String(v)))
                            .collect(),
                    )
                },
            },
        })
        .collect();

    crate::OpenFlowSpec {
        summary: workflow_name,
        value: crate::OpenFlowValue { modules },
    }
}
