// Node.js task extractor - to be implemented later
use crate::error::{ExportError, ExportResult as Result};
use crate::extract::TaskSource;

pub fn extract_node_task(_source: &str, _task_id: &str) -> Result<TaskSource> {
    Err(ExportError::InvalidWorkflowSyntax(
        "Node.js task extractor not yet implemented".into()
    ))
}
