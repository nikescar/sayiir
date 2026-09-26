use crate::extract::TaskSource;
use crate::error::ExportResult as Result;

pub fn extract_all_python_tasks(source: &str) -> Result<Vec<TaskSource>> {
    let mut tasks = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Look for @task decorator (with or without arguments)
        if line.starts_with("@task") {
            let decorator_start = i;
            i += 1;

            // Skip decorator arguments if multi-line (look for closing paren)
            while i < lines.len() {
                let current = lines[i].trim();
                // If we hit a def, break
                if current.starts_with("def ") {
                    break;
                }
                // If line doesn't look like decorator continuation, break
                if !current.is_empty() && !current.ends_with(',') && !current.ends_with('(') && !current.contains(')') {
                    break;
                }
                i += 1;
                // If we found closing paren, next should be def
                if current.contains(')') {
                    break;
                }
            }

            if i >= lines.len() {
                break;
            }

            // Now we should be at the def line
            let def_line = lines[i].trim();
            if let Some(fn_name) = parse_function_name(def_line) {
                // Extract function body (indent-aware)
                let (fn_source, end_idx) = extract_python_function(&lines, i);

                // Store the full file source instead of just the function
                // This allows extract_clean to extract imports, classes, and constants
                tasks.push(TaskSource {
                    id: fn_name.clone(),
                    entry_point: fn_name,
                    source_code: source.to_string(),
                });

                i = end_idx - 1; // -1 because we'll increment at the end of the loop
            }
        }

        i += 1;
    }

    Ok(tasks)
}

fn parse_function_name(line: &str) -> Option<String> {
    if !line.starts_with("def ") {
        return None;
    }

    let after_def = &line[4..];
    let paren_pos = after_def.find('(')?;
    let fn_name = after_def[..paren_pos].trim();
    Some(fn_name.to_string())
}

fn extract_python_function(lines: &[&str], start: usize) -> (String, usize) {
    let def_line = lines[start];
    let base_indent = count_leading_spaces(def_line);

    let mut fn_lines = vec![def_line];
    let mut i = start + 1;

    while i < lines.len() {
        let line = lines[i];

        // Empty lines are part of the function
        if line.trim().is_empty() {
            fn_lines.push(line);
            i += 1;
            continue;
        }

        let line_indent = count_leading_spaces(line);

        // If indentation <= base, function ended
        if line_indent <= base_indent {
            break;
        }

        fn_lines.push(line);
        i += 1;
    }

    (fn_lines.join("\n"), i)
}

fn count_leading_spaces(line: &str) -> usize {
    line.chars().take_while(|c| c.is_whitespace()).count()
}
