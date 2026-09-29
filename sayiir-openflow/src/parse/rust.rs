use crate::error::{ExportError, ExportResult as Result};
use syn::{File, Item, Macro, Expr};

#[derive(Debug, Clone)]
pub struct RustWorkflow {
    pub name: String,
    pub task_names: Vec<String>,
}

pub fn parse_rust_workflow(source: &str) -> Result<RustWorkflow> {
    let ast: File = syn::parse_str(source)
        .map_err(|e| ExportError::InvalidWorkflowSyntax(format!("Failed to parse Rust: {}", e)))?;

    // Find workflow! macro invocation
    for item in &ast.items {
        if let Item::Fn(func) = item {
            // Check function body for workflow! macro
            if let Some(workflow) = find_workflow_in_block(&func.block)? {
                return Ok(workflow);
            }
        }
    }

    // Also check top-level expressions
    for item in &ast.items {
        if let Item::Macro(mac) = item {
            if is_workflow_macro(&mac.mac) {
                return parse_workflow_macro(&mac.mac);
            }
        }
    }

    Err(ExportError::NoWorkflowFound)
}

fn find_workflow_in_block(block: &syn::Block) -> Result<Option<RustWorkflow>> {
    for stmt in &block.stmts {
        if let syn::Stmt::Local(local) = stmt {
            if let Some(init) = &local.init {
                // Handle direct macro: let workflow = workflow! { ... };
                if let Expr::Macro(expr_mac) = &*init.expr {
                    if is_workflow_macro(&expr_mac.mac) {
                        return Ok(Some(parse_workflow_macro(&expr_mac.mac)?));
                    }
                }
                // Handle macro with method call: let workflow = workflow! { ... }.unwrap();
                if let Expr::MethodCall(method_call) = &*init.expr {
                    if let Expr::Macro(expr_mac) = &*method_call.receiver {
                        if is_workflow_macro(&expr_mac.mac) {
                            return Ok(Some(parse_workflow_macro(&expr_mac.mac)?));
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}

fn is_workflow_macro(mac: &Macro) -> bool {
    mac.path.segments.last().map_or(false, |seg| seg.ident == "workflow")
}

fn parse_workflow_macro(mac: &Macro) -> Result<RustWorkflow> {
    let tokens = mac.tokens.to_string();

    // Parse name: "workflow-name"
    let name = extract_field_value(&tokens, "name")
        .ok_or_else(|| ExportError::InvalidWorkflowSyntax("Missing 'name' field".into()))?;

    // Parse steps: [task1, task2, ...]
    let task_names = extract_task_names(&tokens)?;

    Ok(RustWorkflow { name, task_names })
}

fn extract_field_value(tokens: &str, field: &str) -> Option<String> {
    // Extract field value handling optional whitespace: name : "value" or name: "value"
    let pattern = field;
    if let Some(start) = tokens.find(pattern) {
        let after = &tokens[start + pattern.len()..];
        // Skip whitespace and colon
        let after = after.trim_start();
        if !after.starts_with(':') {
            return None;
        }
        let after = &after[1..]; // Skip colon
        let after = after.trim_start();

        if let Some(quote_start) = after.find('"') {
            let after_quote = &after[quote_start + 1..];
            if let Some(quote_end) = after_quote.find('"') {
                return Some(after_quote[..quote_end].to_string());
            }
        }
    }
    None
}

fn extract_task_names(tokens: &str) -> Result<Vec<String>> {
    // Find steps: [...] (handling optional whitespace around colon)
    let pattern = "steps";
    let start = tokens.find(pattern)
        .ok_or_else(|| ExportError::InvalidWorkflowSyntax("Missing 'steps' field".into()))?;

    let after = &tokens[start + pattern.len()..];
    let after = after.trim_start();
    if !after.starts_with(':') {
        return Err(ExportError::InvalidWorkflowSyntax("Missing 'steps' field".into()));
    }
    let after = &after[1..].trim_start();

    let bracket_start = after.find('[')
        .ok_or_else(|| ExportError::InvalidWorkflowSyntax("Steps must be an array".into()))?;

    let after_bracket = &after[bracket_start + 1..];
    let bracket_end = after_bracket.find(']')
        .ok_or_else(|| ExportError::InvalidWorkflowSyntax("Unclosed steps array".into()))?;

    let steps_content = &after_bracket[..bracket_end];

    // Parse task names (handle identifiers and parallel syntax)
    let mut task_names = Vec::new();
    let mut current_token = String::new();

    for ch in steps_content.chars() {
        match ch {
            '(' | ')' => {
                // Parentheses group parallel tasks but don't affect parsing
            }
            ',' => {
                if !current_token.trim().is_empty() {
                    task_names.push(current_token.trim().to_string());
                    current_token.clear();
                }
            }
            '|' => {
                // Always treat | as separator (for parallel tasks)
                if !current_token.trim().is_empty() {
                    task_names.push(current_token.trim().to_string());
                    current_token.clear();
                }
            }
            c if c.is_alphanumeric() || c == '_' => {
                current_token.push(c);
            }
            ' ' | '\n' | '\t' => {
                // Whitespace - skip
            }
            _ => {}
        }
    }

    if !current_token.trim().is_empty() {
        task_names.push(current_token.trim().to_string());
    }

    Ok(task_names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_field_value() {
        let tokens = r#"name: "my-workflow", codec: JsonCodec"#;
        assert_eq!(extract_field_value(tokens, "name"), Some("my-workflow".to_string()));
    }

    #[test]
    fn test_extract_task_names_simple() {
        let tokens = r#"steps: [task1, task2, task3]"#;
        let names = extract_task_names(tokens).unwrap();
        assert_eq!(names, vec!["task1", "task2", "task3"]);
    }

    #[test]
    fn test_extract_task_names_parallel() {
        let tokens = r#"steps: [download, (transcode_720p || transcode_1080p), upload]"#;
        let names = extract_task_names(tokens).unwrap();
        assert_eq!(names, vec!["download", "transcode_720p", "transcode_1080p", "upload"]);
    }
}
