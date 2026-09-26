//! Extract clean function code using Tree-sitter (no decorators/attributes/wrappers)

use tree_sitter::Parser;

/// Extract Rust function without #[task] attributes
pub fn extract_rust_function(source: &str, function_name: &str) -> Option<String> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).ok()?;

    let tree = parser.parse(source, None)?;
    let root = tree.root_node();

    find_rust_function(&root, source, function_name)
}

fn find_rust_function(node: &tree_sitter::Node, source: &str, target_name: &str) -> Option<String> {
    if node.kind() == "function_item" {
        for child in node.children(&mut node.walk()) {
            if child.kind() == "identifier" {
                let name = child.utf8_text(source.as_bytes()).ok()?;
                if name == target_name {
                    let func_start = node.start_byte();
                    let func_end = node.end_byte();
                    let func_text = &source[func_start..func_end];
                    return Some(remove_rust_attributes(func_text));
                }
            }
        }
    }

    for child in node.children(&mut node.walk()) {
        if let Some(result) = find_rust_function(&child, source, target_name) {
            return Some(result);
        }
    }

    None
}

fn remove_rust_attributes(func_text: &str) -> String {
    let lines: Vec<&str> = func_text.lines().collect();
    let mut result = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        // Skip lines starting with #[
        if trimmed.starts_with("#[") || trimmed.starts_with("# [") {
            continue;
        }
        result.push(line);
    }

    result.join("\n")
}

/// Extract Python function without @task decorator
pub fn extract_python_function(source: &str, function_name: &str) -> Option<String> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_python::LANGUAGE.into()).ok()?;

    let tree = parser.parse(source, None)?;
    let root = tree.root_node();

    // For simple extraction without dependencies, just find the function
    // This matches the original behavior for backwards compatibility
    if root.kind() != "module" {
        return None;
    }

    // Extract imports, constants, classes, and the function
    let mut imports = Vec::new();
    let mut classes = Vec::new();
    let mut constants = Vec::new();
    let mut function_text = None;

    for child in root.children(&mut root.walk()) {
        match child.kind() {
            "import_statement" | "import_from_statement" => {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    imports.push(text.to_string());
                }
            }
            "class_definition" => {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    classes.push(text.to_string());
                }
            }
            "expression_statement" => {
                // Module-level assignment (constant)
                if let Some(assignment) = child.child(0) {
                    if assignment.kind() == "assignment" {
                        if let Ok(text) = child.utf8_text(source.as_bytes()) {
                            // Include constants (simple assignments at module level)
                            constants.push(text.to_string());
                        }
                    }
                }
            }
            "decorated_definition" => {
                // Function with decorator - extract the inner function_definition
                for dec_child in child.children(&mut child.walk()) {
                    if dec_child.kind() == "function_definition" {
                        // Check if this is the target function
                        for func_child in dec_child.children(&mut dec_child.walk()) {
                            if func_child.kind() == "identifier" {
                                if let Ok(name) = func_child.utf8_text(source.as_bytes()) {
                                    if name == function_name {
                                        if let Ok(text) = dec_child.utf8_text(source.as_bytes()) {
                                            function_text = Some(text.to_string());
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "function_definition" => {
                // Function without decorator
                for func_child in child.children(&mut child.walk()) {
                    if func_child.kind() == "identifier" {
                        if let Ok(name) = func_child.utf8_text(source.as_bytes()) {
                            if name == function_name {
                                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                                    function_text = Some(text.to_string());
                                }
                                break;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Return just the function if no dependencies needed
    let func = function_text?;
    if imports.is_empty() && classes.is_empty() && constants.is_empty() {
        return Some(func);
    }

    // Build complete code with dependencies
    let mut result = String::new();

    if !imports.is_empty() {
        result.push_str(&imports.join("\n"));
        result.push_str("\n\n");
    }

    if !constants.is_empty() {
        result.push_str(&constants.join("\n"));
        result.push_str("\n\n");
    }

    if !classes.is_empty() {
        result.push_str(&classes.join("\n\n"));
        result.push_str("\n\n");
    }

    result.push_str(&func);

    Some(result)
}


fn remove_python_decorators(func_text: &str) -> String {
    let lines: Vec<&str> = func_text.lines().collect();
    let mut result = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        // Skip lines starting with @
        if trimmed.starts_with('@') {
            continue;
        }
        result.push(line);
    }

    result.join("\n")
}

/// Extract JavaScript/TypeScript function from task() wrapper
pub fn extract_javascript_function(source: &str, function_name: &str) -> Option<String> {
    let mut parser = Parser::new();
    // Use TypeScript parser for both TS and JS (TS is superset of JS)
    parser.set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()).ok()?;

    let tree = parser.parse(source, None)?;
    let root = tree.root_node();

    find_javascript_function(&root, source, function_name)
}

fn find_javascript_function(node: &tree_sitter::Node, source: &str, target_name: &str) -> Option<String> {
    // Look for: const NAME = task("id", arrow_function)
    if node.kind() == "variable_declarator" {
        let mut found_name = None;
        let mut arrow_function = None;
        let mut is_task_call = false;

        for child in node.children(&mut node.walk()) {
            if child.kind() == "identifier" && found_name.is_none() {
                found_name = child.utf8_text(source.as_bytes()).ok();
            } else if child.kind() == "call_expression" {
                // Check if it's task(...)
                for call_child in child.children(&mut child.walk()) {
                    if call_child.kind() == "identifier" {
                        if let Ok(func_name) = call_child.utf8_text(source.as_bytes()) {
                            if func_name == "task" {
                                is_task_call = true;
                            }
                        }
                    } else if call_child.kind() == "arguments" {
                        // Find arrow_function in arguments
                        for arg in call_child.children(&mut call_child.walk()) {
                            if arg.kind() == "arrow_function" {
                                arrow_function = arg.utf8_text(source.as_bytes()).ok();
                            }
                        }
                    }
                }
            }
        }

        if let Some(name) = found_name {
            if name == target_name && is_task_call {
                if let Some(arrow_text) = arrow_function {
                    return Some(convert_arrow_to_function(name, arrow_text));
                }
            }
        }
    }

    for child in node.children(&mut node.walk()) {
        if let Some(result) = find_javascript_function(&child, source, target_name) {
            return Some(result);
        }
    }

    None
}

fn convert_arrow_to_function(name: &str, arrow_text: &str) -> String {
    // Simple conversion: (params) => { body } -> function name(params) { body }

    // Extract params and body
    if let Some(arrow_idx) = arrow_text.find("=>") {
        let params_part = &arrow_text[..arrow_idx].trim();
        let body_part = &arrow_text[arrow_idx + 2..].trim();

        // Remove outer parens from params if present
        let params = params_part.trim_start_matches('(').trim_end_matches(')');

        // Strip TypeScript type annotations from params: (x: Type) -> (x)
        let clean_params = strip_typescript_types(params);

        // Strip 'as const' from body
        let clean_body = body_part.replace(" as const", "");

        format!("function {}({}) {}", name, clean_params, clean_body)
    } else {
        // Fallback
        format!("function {}() {{}}", name)
    }
}

fn strip_typescript_types(params: &str) -> String {
    // Simple regex-like replacement: remove ": Type" patterns
    let mut result = String::new();
    let mut chars = params.chars().peekable();
    let mut depth = 0;

    while let Some(c) = chars.next() {
        if c == ':' && depth == 0 {
            // Skip until comma or end
            while let Some(&next) = chars.peek() {
                if next == ',' {
                    break;
                }
                chars.next();
            }
        } else {
            if c == '(' || c == '<' {
                depth += 1;
            } else if c == ')' || c == '>' {
                depth -= 1;
            }
            result.push(c);
        }
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_function() {
        let source = r#"
#[task(id = "download", retries = 2)]
pub async fn download_video(req: Request) -> Result<Video> {
    Ok(Video {})
}
"#;
        let result = extract_rust_function(source, "download_video");
        assert!(result.is_some());
        let clean = result.unwrap();
        assert!(!clean.contains("#[task"));
        assert!(clean.contains("pub async fn download_video"));
    }

    #[test]
    fn test_extract_python_function() {
        let source = r#"
@task(description="Parse query")
def parse_query(raw: dict) -> dict:
    return raw
"#;
        let result = extract_python_function(source, "parse_query");
        assert!(result.is_some());
        let clean = result.unwrap();
        assert!(!clean.contains("@task"));
        assert!(clean.contains("def parse_query"));
    }

    #[test]
    fn test_extract_python_with_dependencies() {
        let source = r#"from models import ResearchQuery
import re

DRAFTS_DIR = "reports/drafts"

class Helper:
    pass

@task(description="Parse query")
def parse_query(raw: dict) -> dict:
    query = ResearchQuery.model_validate(raw)
    return query.model_dump()
"#;
        let result = extract_python_function(source, "parse_query");
        assert!(result.is_some());
        let clean = result.unwrap();

        // Should include imports
        assert!(clean.contains("from models import ResearchQuery"));
        assert!(clean.contains("import re"));

        // Should include constants
        assert!(clean.contains("DRAFTS_DIR"));

        // Should include classes
        assert!(clean.contains("class Helper"));

        // Should include function without decorator
        assert!(!clean.contains("@task"));
        assert!(clean.contains("def parse_query"));
    }

    #[test]
    fn test_extract_python_with_class_in_same_file() {
        let source = r#"class MyModel:
    def __init__(self, value):
        self.value = value

@task(description="Process data")
def process_data(raw: dict) -> dict:
    model = MyModel(raw["value"])
    return {"result": model.value}
"#;
        let result = extract_python_function(source, "process_data");
        assert!(result.is_some());
        let clean = result.unwrap();

        // Should include class
        assert!(clean.contains("class MyModel"));

        // Should include function without decorator
        assert!(!clean.contains("@task"));
        assert!(clean.contains("def process_data"));
    }

    #[test]
    fn test_extract_javascript_function() {
        let source = r#"
const validateOrder = task("validate-order", (order: Order) => {
  return { validated: true as const };
});
"#;
        let result = extract_javascript_function(source, "validateOrder");
        assert!(result.is_some());
        let clean = result.unwrap();
        assert!(!clean.contains("task("));
        assert!(clean.contains("function validateOrder"));
        assert!(!clean.contains(": Order"));
        assert!(!clean.contains("as const"));
    }
}
