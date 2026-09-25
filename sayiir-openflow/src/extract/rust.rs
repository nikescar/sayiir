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
}
