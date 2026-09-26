use crate::error::{ExportError, ExportResult as Result};
use crate::extract::TaskSource;
use syn::{File, Item, Attribute};
use quote::ToTokens;

pub fn extract_rust_task(source: &str, task_id: &str) -> Result<TaskSource> {
    let ast: File = syn::parse_str(source)
        .map_err(|e| ExportError::InvalidWorkflowSyntax(format!("Failed to parse Rust: {}", e)))?;

    for item in &ast.items {
        if let Item::Fn(func) = item {
            // Check for #[task(id = "task_id")] attribute
            if has_task_attribute(&func.attrs, task_id) {
                let entry_point = func.sig.ident.to_string();
                let source_code = extract_function_source(source, func)?;

                return Ok(TaskSource {
                    id: task_id.to_string(),
                    source_code,
                    entry_point,
                });
            }
        }
    }

    Err(ExportError::TaskNotFound {
        task_id: task_id.to_string(),
    })
}

fn has_task_attribute(attrs: &[Attribute], task_id: &str) -> bool {
    for attr in attrs {
        if attr.path().is_ident("task") {
            // Parse attribute arguments: #[task(id = "task_id", ...)]
            if let Ok(list) = attr.meta.require_list() {
                let tokens = list.tokens.to_string();
                // Simple check: look for id = "task_id"
                let pattern = format!(r#"id = "{}""#, task_id);
                if tokens.contains(&pattern) {
                    return true;
                }
            }
        }
    }
    false
}

fn extract_function_source(_source: &str, func: &syn::ItemFn) -> Result<String> {
    // Convert the function AST back to source code
    // Note: We ignore the _source parameter and use ToTokens to format the function
    // This ensures we get valid Rust code including all attributes
    let source_code = func.to_token_stream().to_string();
    Ok(source_code)
}

pub fn extract_all_rust_tasks(source: &str) -> Result<Vec<TaskSource>> {
    let ast: File = syn::parse_str(source)
        .map_err(|e| ExportError::InvalidWorkflowSyntax(format!("Failed to parse Rust: {}", e)))?;

    let mut tasks = Vec::new();

    for item in &ast.items {
        if let Item::Fn(func) = item {
            if let Some(task_id) = get_task_id(&func.attrs, &func.sig.ident) {
                let entry_point = func.sig.ident.to_string();

                // Store the full file source instead of just the function
                // This allows extract_clean to extract use declarations, types, and constants
                tasks.push(TaskSource {
                    id: task_id,
                    source_code: source.to_string(),
                    entry_point,
                });
            }
        }
    }

    Ok(tasks)
}

fn get_task_id(attrs: &[Attribute], fn_ident: &syn::Ident) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("task") {
            // If #[task(id = "explicit_id")], use explicit_id
            if let Ok(list) = attr.meta.require_list() {
                let tokens = list.tokens.to_string();
                // Simple extraction: look for id = "value"
                if let Some(start) = tokens.find(r#"id = ""#) {
                    let after = &tokens[start + 6..];
                    if let Some(end) = after.find('"') {
                        return Some(after[..end].to_string());
                    }
                }
            }
            // If just #[task], infer from function name
            return Some(fn_ident.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_task_attribute() {
        let source = r#"
#[task(id = "test_task")]
fn my_task() {}
        "#;
        let ast: File = syn::parse_str(source).unwrap();
        if let Item::Fn(func) = &ast.items[0] {
            assert!(has_task_attribute(&func.attrs, "test_task"));
            assert!(!has_task_attribute(&func.attrs, "other_task"));
        }
    }

    #[test]
    fn test_extract_all_rust_tasks() {
        let source = r#"
#[task(id = "task1")]
fn first_task() {}

#[task(id = "task2")]
async fn second_task(input: String) -> Result<String, BoxError> {
    Ok(input)
}

#[task]
fn third_task() {}

fn not_a_task() {}
"#;

        let tasks = extract_all_rust_tasks(source).unwrap();

        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].id, "task1");
        assert_eq!(tasks[1].id, "task2");
        assert_eq!(tasks[2].id, "third_task"); // Inferred from function name
    }
}
