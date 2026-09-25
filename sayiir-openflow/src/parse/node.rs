// Node.js workflow parser - to be implemented in Task 9
use crate::error::{ExportError, ExportResult as Result};

#[derive(Debug, Clone)]
pub struct NodeWorkflow {
    pub name: String,
    pub task_names: Vec<String>,
}

pub fn parse_node_workflow(_source: &str) -> Result<NodeWorkflow> {
    Err(ExportError::InvalidWorkflowSyntax(
        "Node.js parser not yet implemented".into()
    ))
}
