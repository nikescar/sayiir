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
        let task_start = cap.get(0).unwrap().start();

        if let Ok(task_source) = extract_task_body(source, task_start) {
            tasks.push(TaskSource {
                id: task_id.to_string(),
                entry_point: var_name.to_string(),
                source_code: task_source,
            });
        }
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

    for (i, ch) in source[brace_start..].chars().enumerate() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end_pos = brace_start + i + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    // Add closing );
    while end_pos < source.len() {
        let ch = source.chars().nth(end_pos).unwrap();
        if ch == ';' {
            end_pos += 1;
            break;
        } else if !ch.is_whitespace() && ch != ')' {
            break;
        }
        end_pos += 1;
    }

    Ok(source[start..end_pos].to_string())
}
