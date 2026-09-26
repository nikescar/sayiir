use crate::extract::TaskSource;
use crate::error::{ExportError, ExportResult as Result};
use regex::Regex;

pub fn extract_all_node_tasks(source: &str) -> Result<Vec<TaskSource>> {
    let mut tasks = Vec::new();

    // Match: const/let/var name = task("id", async? (args) => ... or (args): Type => ...
    let pattern = r#"(?:const|let|var)\s+(\w+)\s*=\s*task\s*\(\s*["']([^"']+)["']\s*,"#;
    let re = Regex::new(pattern).unwrap();

    for cap in re.captures_iter(source) {
        let var_name = cap.get(1).unwrap().as_str();
        let task_id = cap.get(2).unwrap().as_str();

        // Store the full file source instead of just the task body
        // This allows extract_clean to extract imports, types, and constants
        tasks.push(TaskSource {
            id: task_id.to_string(),
            entry_point: var_name.to_string(),
            source_code: source.to_string(),
        });
    }

    Ok(tasks)
}

fn extract_task_body(source: &str, start: usize) -> Result<String> {
    let after_start = &source[start..];
    let first_brace = after_start.find('{')
        .ok_or_else(|| ExportError::InvalidWorkflowSyntax("No opening brace".into()))?;

    let brace_start = start + first_brace;

    let mut depth = 0;
    let mut end_pos = brace_start;

    // Use byte iteration instead of char iteration to preserve byte offsets
    for (i, &byte) in source.as_bytes()[brace_start..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    end_pos = brace_start + i + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    // Add closing ); - skip whitespace, commas, and closing parens until we find semicolon
    let bytes = source.as_bytes();
    while end_pos < bytes.len() {
        let byte = bytes[end_pos];
        let ch = byte as char;
        if byte == b';' {
            end_pos += 1;
            break;
        } else if !ch.is_ascii_whitespace() && byte != b')' && byte != b',' {
            break;
        }
        end_pos += 1;
    }

    Ok(source[start..end_pos].to_string())
}
