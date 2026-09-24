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
pub fn import_openflow_json(_json: &str) -> Result<()> {
    todo!("Implement in Task 2.2")
}

/// Export Sayiir workflow to OpenFlow JSON
pub fn export_openflow_json() -> Result<String> {
    todo!("Implement in Task 2.4")
}

/// Import Mermaid markdown flowchart
pub fn import_mermaid(_markdown: &str) -> Result<()> {
    todo!("Implement in Task 2.5")
}

/// Export workflow to Mermaid markdown
pub fn export_mermaid() -> Result<String> {
    todo!("Implement in Task 2.5")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openflow_spec_parse() {
        let json = r#"{"summary": "Test", "value": {"modules": []}}"#;
        let spec: OpenFlowSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.summary, "Test");
        assert_eq!(spec.modules.len(), 0);
    }
}
