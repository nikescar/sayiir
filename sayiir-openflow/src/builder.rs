//! Builder API for creating OpenFlow workflows with less boilerplate

use crate::{OpenFlowModule, OpenFlowModuleValue, OpenFlowSpec, OpenFlowValue};
use serde_json::json;

/// Fluent builder for OpenFlow workflows
pub struct WorkflowBuilder {
    summary: String,
    modules: Vec<OpenFlowModule>,
}

impl WorkflowBuilder {
    /// Create a new workflow builder
    pub fn new(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            modules: Vec::new(),
        }
    }

    /// Add a Rust task with embedded code
    pub fn rust_task(
        mut self,
        id: impl Into<String>,
        entry_point: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        let id_str = id.into();
        let mut deps = serde_json::Map::new();
        deps.insert("serde_json".to_string(), json!("1.0"));

        self.modules.push(OpenFlowModule {
            id: id_str.clone(),
            value: OpenFlowModuleValue::Script {
                path: id_str,
                language: Some("rust".to_string()),
                entry_point: Some(entry_point.into()),
                code: Some(code.into()),
                dependencies: Some(deps),
            },
        });
        self
    }

    /// Add a Rust task without dependencies
    pub fn rust_task_no_deps(
        mut self,
        id: impl Into<String>,
        entry_point: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        let id_str = id.into();
        self.modules.push(OpenFlowModule {
            id: id_str.clone(),
            value: OpenFlowModuleValue::Script {
                path: id_str,
                language: Some("rust".to_string()),
                entry_point: Some(entry_point.into()),
                code: Some(code.into()),
                dependencies: None,
            },
        });
        self
    }

    /// Add a Python task
    pub fn python_task(
        mut self,
        id: impl Into<String>,
        entry_point: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        let id_str = id.into();
        self.modules.push(OpenFlowModule {
            id: id_str.clone(),
            value: OpenFlowModuleValue::Script {
                path: id_str,
                language: Some("python".to_string()),
                entry_point: Some(entry_point.into()),
                code: Some(code.into()),
                dependencies: None,
            },
        });
        self
    }

    /// Add a Python task with dependencies
    pub fn python_task_with_deps(
        mut self,
        id: impl Into<String>,
        entry_point: impl Into<String>,
        code: impl Into<String>,
        deps: &[(&str, &str)],
    ) -> Self {
        let id_str = id.into();
        let mut dependencies = serde_json::Map::new();
        for (pkg, version) in deps {
            dependencies.insert(pkg.to_string(), json!(version));
        }

        self.modules.push(OpenFlowModule {
            id: id_str.clone(),
            value: OpenFlowModuleValue::Script {
                path: id_str,
                language: Some("python".to_string()),
                entry_point: Some(entry_point.into()),
                code: Some(code.into()),
                dependencies: Some(dependencies),
            },
        });
        self
    }

    /// Add a Node.js task
    pub fn node_task(
        mut self,
        id: impl Into<String>,
        entry_point: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        let id_str = id.into();
        self.modules.push(OpenFlowModule {
            id: id_str.clone(),
            value: OpenFlowModuleValue::Script {
                path: id_str,
                language: Some("node".to_string()),
                entry_point: Some(entry_point.into()),
                code: Some(code.into()),
                dependencies: None,
            },
        });
        self
    }

    /// Add a Node.js task with dependencies
    pub fn node_task_with_deps(
        mut self,
        id: impl Into<String>,
        entry_point: impl Into<String>,
        code: impl Into<String>,
        deps: &[(&str, &str)],
    ) -> Self {
        let id_str = id.into();
        let mut dependencies = serde_json::Map::new();
        for (pkg, version) in deps {
            dependencies.insert(pkg.to_string(), json!(version));
        }

        self.modules.push(OpenFlowModule {
            id: id_str.clone(),
            value: OpenFlowModuleValue::Script {
                path: id_str,
                language: Some("node".to_string()),
                entry_point: Some(entry_point.into()),
                code: Some(code.into()),
                dependencies: Some(dependencies),
            },
        });
        self
    }

    /// Build the final OpenFlow spec
    pub fn build(self) -> OpenFlowSpec {
        OpenFlowSpec {
            summary: self.summary,
            value: OpenFlowValue {
                modules: self.modules,
            },
        }
    }
}

/// Macro to define a workflow with less boilerplate
#[macro_export]
macro_rules! openflow_workflow {
    (
        summary: $summary:expr,
        tasks: [
            $( $task:expr ),* $(,)?
        ]
    ) => {{
        use $crate::builder::WorkflowBuilder;
        WorkflowBuilder::new($summary)
            $( .add_module($task) )*
            .build()
    }};
}

impl WorkflowBuilder {
    /// Add a pre-built module (for macro support)
    pub fn add_module(mut self, module: OpenFlowModule) -> Self {
        self.modules.push(module);
        self
    }
}
