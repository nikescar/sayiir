use crate::extract::TaskSource;
use crate::error::ExportResult as Result;

pub fn extract_all_python_tasks(source: &str) -> Result<Vec<TaskSource>> {
    let mut tasks = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Look for @task decorator
        if line == "@task" {
            i += 1;
            if i >= lines.len() {
                break;
            }

            // Next line should be: def function_name(...)
            let def_line = lines[i].trim();
            if let Some(fn_name) = parse_function_name(def_line) {
                // Extract function body (indent-aware)
                let (fn_source, end_idx) = extract_python_function(&lines, i);

                tasks.push(TaskSource {
                    id: fn_name.clone(),
                    entry_point: fn_name,
                    source_code: format!("@task\n{}", fn_source),
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
