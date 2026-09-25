use crate::error::{ExportError, ExportResult as Result};
use regex::Regex;

pub struct PythonWorkflow {
    pub name: String,
    pub task_names: Vec<String>,
}

pub fn parse_python_workflow(source: &str) -> Result<PythonWorkflow> {
    let name = extract_flow_name(source)
        .ok_or_else(|| ExportError::NoWorkflowFound)?;

    let task_names = extract_then_calls(source);

    if task_names.is_empty() {
        return Err(ExportError::NoWorkflowFound);
    }

    Ok(PythonWorkflow { name, task_names })
}

fn extract_flow_name(source: &str) -> Option<String> {
    let pattern = r#"Flow\s*\(\s*["']([^"']+)["']\s*\)"#;
    let re = Regex::new(pattern).ok()?;
    let caps = re.captures(source)?;
    Some(caps.get(1)?.as_str().to_string())
}

fn extract_then_calls(source: &str) -> Vec<String> {
    let pattern = r#"\.then\s*\(\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)"#;
    let re = Regex::new(pattern).unwrap();

    re.captures_iter(source)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}
