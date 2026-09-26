//! Import OpenFlow workflows and generate standalone executable projects.

use crate::{OpenFlowError, OpenFlowModule, OpenFlowModuleValue, OpenFlowSpec, Result};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tree_sitter::Parser;

/// Import workflow and generate standalone project
pub async fn import_workflow(spec: &OpenFlowSpec, output_dir: &Path) -> Result<()> {
    // Detect if mono-language or multi-language
    let languages = detect_languages(spec);

    if languages.len() == 1 {
        let lang = languages.iter().next()
            .ok_or_else(|| OpenFlowError::InvalidWorkflow("no language detected".into()))?;
        generate_mono_project(spec, output_dir, lang).await
    } else {
        generate_multi_project(spec, output_dir, &languages).await
    }
}

fn detect_languages(spec: &OpenFlowSpec) -> HashSet<String> {
    let mut languages = HashSet::new();
    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            language: Some(lang),
            ..
        } = &module.value
        {
            languages.insert(lang.clone());
        }
    }
    languages
}

async fn generate_mono_project(
    spec: &OpenFlowSpec,
    output_dir: &Path,
    language: &str,
) -> Result<()> {
    match language {
        "rust" => generate_rust_project(spec, output_dir).await,
        "python" => generate_python_project(spec, output_dir).await,
        "node" => generate_node_project(spec, output_dir).await,
        _ => Err(OpenFlowError::Unsupported(format!(
            "language: {}",
            language
        ))),
    }
}

async fn generate_multi_project(
    spec: &OpenFlowSpec,
    output_dir: &Path,
    languages: &HashSet<String>,
) -> Result<()> {
    std::fs::create_dir_all(output_dir)?;

    // Group modules by language
    let mut modules_by_lang: HashMap<String, Vec<&OpenFlowModule>> = HashMap::new();
    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            language: Some(lang),
            ..
        } = &module.value
        {
            modules_by_lang
                .entry(lang.clone())
                .or_insert_with(Vec::new)
                .push(module);
        }
    }

    // Generate subdirectories for each language
    for lang in languages {
        let lang_dir = output_dir.join(format!("{}_tasks", lang));
        std::fs::create_dir_all(&lang_dir)?;

        let lang_modules: Vec<OpenFlowModule> = modules_by_lang
            .get(lang)
            .unwrap_or(&vec![])
            .iter()
            .map(|&m| m.clone())
            .collect();

        let lang_spec = OpenFlowSpec {
            summary: format!("{} - {} tasks", spec.summary, lang),
            value: crate::OpenFlowValue {
                modules: lang_modules,
            },
        };

        generate_mono_project(&lang_spec, &lang_dir, lang).await?;
    }

    // Generate orchestrator README
    let readme = generate_multi_readme(spec, languages);
    std::fs::write(output_dir.join("README.md"), readme)?;

    Ok(())
}

async fn generate_rust_project(spec: &OpenFlowSpec, output_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(output_dir)?;
    std::fs::create_dir_all(output_dir.join("src"))?;

    // Collect all dependencies
    let mut all_deps = HashMap::new();
    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            dependencies: Some(deps),
            ..
        } = &module.value
        {
            for (k, v) in deps {
                all_deps.insert(k.clone(), v.clone());
            }
        }
    }

    // Generate Cargo.toml
    let cargo_toml = generate_rust_cargo_toml(&spec.summary, &all_deps);
    std::fs::write(output_dir.join("Cargo.toml"), cargo_toml)?;

    // Generate src/main.rs
    let main_rs = generate_rust_main(spec)?;
    std::fs::write(output_dir.join("src/main.rs"), main_rs)?;

    // Generate README.md
    let readme = generate_rust_readme(spec);
    std::fs::write(output_dir.join("README.md"), readme)?;

    Ok(())
}

async fn generate_python_project(spec: &OpenFlowSpec, output_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(output_dir)?;

    // Collect all dependencies
    let mut all_deps = HashMap::new();
    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            dependencies: Some(deps),
            ..
        } = &module.value
        {
            for (k, v) in deps {
                all_deps.insert(k.clone(), v.clone());
            }
        }
    }

    // Generate pyproject.toml
    let pyproject = generate_python_pyproject(&spec.summary, &all_deps);
    std::fs::write(output_dir.join("pyproject.toml"), pyproject)?;

    // Generate main.py
    let main_py = generate_python_main(spec)?;
    std::fs::write(output_dir.join("main.py"), main_py)?;

    // Generate README.md
    let readme = generate_python_readme(spec);
    std::fs::write(output_dir.join("README.md"), readme)?;

    Ok(())
}

async fn generate_node_project(spec: &OpenFlowSpec, output_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(output_dir)?;

    // Collect all dependencies
    let mut all_deps = serde_json::Map::new();
    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            dependencies: Some(deps),
            ..
        } = &module.value
        {
            for (k, v) in deps {
                all_deps.insert(k.clone(), v.clone());
            }
        }
    }

    // Generate package.json
    let package_json = generate_node_package_json(&spec.summary, &all_deps);
    std::fs::write(output_dir.join("package.json"), package_json)?;

    // Generate index.js
    let index_js = generate_node_main(spec)?;
    std::fs::write(output_dir.join("index.js"), index_js)?;

    // Generate README.md
    let readme = generate_node_readme(spec);
    std::fs::write(output_dir.join("README.md"), readme)?;

    Ok(())
}

fn generate_rust_cargo_toml(
    summary: &str,
    deps: &HashMap<String, serde_json::Value>,
) -> String {
    let mut cargo_toml = format!(
        r#"[workspace]
# Prevent cargo from detecting parent workspace

[package]
name = "workflow"
version = "0.1.0"
edition = "2021"

# {}

[dependencies]
serde_json = "1.0"
tokio = {{ version = "1", features = ["full"] }}
"#,
        summary
    );

    for (pkg, version) in deps {
        if pkg == "serde_json" || pkg == "tokio" {
            continue;
        }
        if let Some(ver_str) = version.as_str() {
            cargo_toml.push_str(&format!("{} = \"{}\"\n", pkg, ver_str));
        }
    }

    cargo_toml
}

fn generate_rust_main(spec: &OpenFlowSpec) -> Result<String> {
    use std::collections::HashSet;

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into())
        .map_err(|e| OpenFlowError::InvalidWorkflow(format!("tree-sitter init failed: {e}")))?;

    let mut uses = HashSet::new();
    let mut types = HashSet::new();        // Deduplicate types
    let mut constants = HashSet::new();    // Deduplicate constants
    let mut helpers = HashSet::new();      // Deduplicate helper functions
    let mut task_functions = HashSet::new();   // Task functions (entry points, deduplicated)

    // Collect all entry points first to distinguish tasks from helpers
    let all_entry_points: HashSet<String> = spec
        .value
        .modules
        .iter()
        .filter_map(|m| {
            if let OpenFlowModuleValue::Script {
                entry_point: Some(ep),
                ..
            } = &m.value
            {
                Some(ep.clone())
            } else {
                None
            }
        })
        .collect();

    // Parse each task's code with tree-sitter to extract declarations
    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            code: Some(task_code),
            entry_point,
            ..
        } = &module.value
        {
            let tree = parser.parse(task_code, None)
                .ok_or_else(|| OpenFlowError::InvalidWorkflow("tree-sitter parse failed".into()))?;
            let root = tree.root_node();

            if root.kind() != "source_file" {
                continue;
            }

            // Extract top-level items using AST traversal
            for child in root.children(&mut root.walk()) {
                if let Ok(text) = child.utf8_text(task_code.as_bytes()) {
                    match child.kind() {
                        "use_declaration" => {
                            uses.insert(text.to_string());
                        }
                        "struct_item" | "enum_item" | "type_item" => {
                            types.insert(text.to_string());  // Deduplicate types
                        }
                        "const_item" | "static_item" => {
                            constants.insert(text.to_string());  // Deduplicate constants
                        }
                        "function_item" => {
                            // Check if this is a task function (any entry point) or a helper
                            let fn_name = extract_fn_name(&child, task_code);
                            if let Some(name) = fn_name {
                                if all_entry_points.contains(&name) {
                                    // Only add to task_functions if it matches THIS module's entry point
                                    if let Some(ep) = entry_point {
                                        if name == *ep {
                                            task_functions.insert(text.to_string());
                                        }
                                    }
                                    // Skip adding to helpers - it's a task function
                                } else {
                                    helpers.insert(text.to_string());
                                }
                            } else {
                                helpers.insert(text.to_string());
                            }
                        }
                        "attribute_item" => {
                            // Function with attributes (e.g., #[task(...)])
                            for attr_child in child.children(&mut child.walk()) {
                                if attr_child.kind() == "function_item" {
                                    if let Ok(fn_text) = attr_child.utf8_text(task_code.as_bytes()) {
                                        let fn_name = extract_fn_name(&attr_child, task_code);
                                        if let Some(name) = fn_name {
                                            if all_entry_points.contains(&name) {
                                                // Only add to task_functions if it matches THIS module's entry point
                                                if let Some(ep) = entry_point {
                                                    if name == *ep {
                                                        task_functions.insert(fn_text.to_string());
                                                    }
                                                }
                                                // Skip adding to helpers - it's a task function
                                            } else {
                                                helpers.insert(fn_text.to_string());
                                            }
                                        } else {
                                            helpers.insert(fn_text.to_string());
                                        }
                                    }
                                }
                            }
                        }
                        "impl_item" => {
                            // Trait implementations and inherent impls
                            // Skip impl blocks that reference sayiir-specific types (like Task types, JsonCodec)
                            if !text.contains("Task::task_id()") &&
                               !text.contains("JsonCodec") &&
                               !text.contains("sayiir") {
                                types.insert(text.to_string());  // Add to types (will be deduplicated)
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Build final code with deduplicated use declarations
    let mut code = String::from(
        r#"use serde_json::Value;
use serde::{Serialize, Deserialize};

// Common type alias for error handling
type BoxError = Box<dyn std::error::Error + Send + Sync>;

// Stub for sayiir runtime types (used in fork/join workflows)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedBranchResults(pub std::collections::HashMap<String, bytes::Bytes>);

impl NamedBranchResults {
    pub fn into_map(self) -> std::collections::HashMap<String, bytes::Bytes> {
        self.0
    }
}

impl From<std::collections::HashMap<String, bytes::Bytes>> for NamedBranchResults {
    fn from(map: std::collections::HashMap<String, bytes::Bytes>) -> Self {
        NamedBranchResults(map)
    }
}

// Note: TryFrom impl for ForkResults is added dynamically if needed

"#,
    );

    // Add use declarations (deduplicated, sorted, excluding imports already in header)
    let mut use_vec: Vec<_> = uses.into_iter()
        .filter(|u| {
            // Skip imports already in the header
            !u.contains("use serde_json::") &&
            !(u.contains("use serde::") && u.contains("Serialize") && u.contains("Deserialize"))
        })
        .collect();
    use_vec.sort();
    for use_decl in use_vec {
        code.push_str(&use_decl);
        code.push('\n');
    }
    code.push('\n');

    // Add constants (deduplicated, sorted)
    if !constants.is_empty() {
        let mut const_vec: Vec<_> = constants.into_iter().collect();
        const_vec.sort();
        for constant in const_vec {
            code.push_str(&constant);
            code.push('\n');
        }
        code.push('\n');
    }

    // Add types (deduplicated, sorted, with derives added if missing)
    if !types.is_empty() {
        let mut type_vec: Vec<_> = types.into_iter().collect();
        type_vec.sort();
        for type_def in type_vec {
            // Add Serialize/Deserialize derives if not present
            let needs_derives = (type_def.contains("pub struct") || type_def.contains("pub enum"))
                && !type_def.contains("#[derive");

            if needs_derives {
                code.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
            }
            code.push_str(&type_def);
            code.push_str("\n\n");
        }
    }

    // Add helper functions (deduplicated, excluding any that are in task_functions)
    if !helpers.is_empty() {
        let helpers_only: Vec<_> = helpers.difference(&task_functions).cloned().collect();
        let mut helper_vec = helpers_only;
        helper_vec.sort();
        for helper in helper_vec {
            code.push_str(&helper);
            code.push_str("\n\n");
        }
    }

    // Add task functions (deduplicated, sorted)
    if !task_functions.is_empty() {
        let mut task_vec: Vec<_> = task_functions.into_iter().collect();
        task_vec.sort();
        for func in task_vec {
            code.push_str(&func);
            code.push_str("\n\n");
        }
    }

    // Generate async main function with tokio runtime
    code.push_str(
        r#"#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let args: Vec<String> = std::env::args().collect();
    let mut value: Value = if args.len() > 1 {
        serde_json::from_str(&args[1])?
    } else {
        serde_json::json!({})
    };

"#,
    );

    // Chain task calls with .await for async functions and proper type conversion
    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            entry_point: Some(entry),
            ..
        } = &module.value
        {
            code.push_str(&format!(
                r#"    value = {{
        let input = serde_json::from_value(value)?;
        let output = {}(input).await?;
        serde_json::to_value(output)?
    }};
"#,
                entry
            ));
        }
    }

    code.push_str(
        r#"
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
"#,
    );

    // Apply compatibility fixes for standalone execution
    code = apply_rust_compatibility_fixes(code);

    Ok(code)
}

fn apply_rust_compatibility_fixes(mut code: String) -> String {
    // Fix 1: Replace streaming download with simpler bytes() for standalone workflows
    // This is a multi-line replacement for the download_video function
    if code.contains("let mut stream = response.bytes_stream();") {
        // Replace the streaming pattern with a simpler direct bytes download
        code = code.replace(
            "let mut stream = response.bytes_stream();",
            "// Simplified download without streaming"
        );

        // Remove the stream usage loop and replace with direct write
        if let Some(while_pos) = code.find("while let Some(chunk) = stream.next().await {") {
            if let Some(while_end) = code[while_pos..].find("\n    }") {
                let end_pos = while_pos + while_end + 6; // Include the closing brace and newline

                // Extract the download section and replace it
                let replacement = r#"
    // Direct download (simplified from streaming)
    let bytes_data = response.bytes().await?;
    file.write_all(&bytes_data).await?;
"#;
                code.replace_range(while_pos..end_pos, replacement);
            }
        }
    }

    // Fix 2: Add as_key() method for Verdict enum (was from BranchKey derive)
    if code.contains("pub enum Verdict") && code.contains(".as_key()") {
        // Find the Verdict enum definition and add the impl after it
        if let Some(verdict_pos) = code.find("pub enum Verdict {") {
            if let Some(enum_end) = code[verdict_pos..].find("\n}") {
                let insert_pos = verdict_pos + enum_end + 3; // After "}\n"
                let impl_block = r#"
impl Verdict {
    pub fn as_key(&self) -> &'static str {
        match self {
            Verdict::Approved => "Approved",
            Verdict::Rejected => "Rejected",
        }
    }
}
"#;
                code.insert_str(insert_pos, impl_block);
            }
        }
    }

    // Fix 3: Add stub TryFrom impl for ForkResults if needed
    if code.contains("pub struct ForkResults") && code.contains("results.try_into()?") {
        // Add a stub TryFrom impl that returns an error (fork/join not supported in standalone)
        if let Some(fork_pos) = code.find("pub struct ForkResults {") {
            if let Some(struct_end) = code[fork_pos..].find("\n}") {
                let insert_pos = fork_pos + struct_end + 3;
                let impl_block = r#"
impl TryFrom<NamedBranchResults> for ForkResults {
    type Error = BoxError;

    fn try_from(_results: NamedBranchResults) -> Result<Self, Self::Error> {
        // Stub implementation for standalone execution
        // Fork/join workflows require the full sayiir runtime
        Err("Fork/join not supported in standalone mode. Use sayiir runtime for parallel execution.".into())
    }
}
"#;
                code.insert_str(insert_pos, impl_block);
            }
        }
    }

    code
}

fn generate_rust_readme(spec: &OpenFlowSpec) -> String {
    format!(
        r#"# {}

Standalone workflow generated by sayiir-openflow.

## Build

```bash
cargo build --release
```

## Run

```bash
# Run with default input
cargo run

# Run with JSON input
cargo run -- '{{"key": "value"}}'
```

## Tasks

{}
"#,
        spec.summary,
        spec.value
            .modules
            .iter()
            .map(|m| format!("- {}", m.id))
            .collect::<Vec<_>>()
            .join("\n")
    )
}


fn generate_python_pyproject(
    summary: &str,
    deps: &HashMap<String, serde_json::Value>,
) -> String {
    let mut pyproject = format!(
        r#"[project]
name = "workflow"
version = "0.1.0"
description = "{}"

dependencies = [
]
"#,
        summary
    );

    if !deps.is_empty() {
        let deps_str = deps
            .iter()
            .filter_map(|(pkg, ver)| {
                ver.as_str().map(|v| format!("    \"{}=={}\"", pkg, v))
            })
            .collect::<Vec<_>>()
            .join(",\n");

        pyproject = pyproject.replace("dependencies = [\n]", &format!("dependencies = [\n{}\n]", deps_str));
    }

    pyproject
}

fn generate_python_main(spec: &OpenFlowSpec) -> Result<String> {
    use std::collections::HashSet;

    let mut imports = HashSet::new();
    let mut constants = HashSet::new();
    let mut classes = HashSet::new();
    let mut functions = Vec::new();

    // Parse each task's code to extract imports, constants, classes, and functions
    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            code: Some(task_code),
            ..
        } = &module.value
        {
            let mut in_class = false;
            let mut in_function = false;
            let mut class_lines = Vec::new();
            let mut function_lines = Vec::new();

            for line in task_code.lines() {
                let trimmed = line.trim();

                if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                    imports.insert(line.to_string());
                } else if trimmed.starts_with("class ") {
                    in_class = true;
                    class_lines = vec![line.to_string()];
                } else if trimmed.starts_with("def ") {
                    in_function = true;
                    function_lines = vec![line.to_string()];
                } else if in_class {
                    class_lines.push(line.to_string());
                    if !line.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                        classes.insert(class_lines.join("\n"));
                        in_class = false;
                    }
                } else if in_function {
                    function_lines.push(line.to_string());
                    if !line.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                        functions.push(function_lines.join("\n"));
                        in_function = false;
                    }
                } else if trimmed.contains('=') && !trimmed.starts_with('@') && !in_class && !in_function {
                    constants.insert(line.to_string());
                } else if in_function {
                    function_lines.push(line.to_string());
                }
            }

            // Handle last class/function
            if in_class {
                classes.insert(class_lines.join("\n"));
            }
            if in_function {
                functions.push(function_lines.join("\n"));
            }
        }
    }

    // Build final code with deduplicated imports
    let mut code = String::from(
        r#"#!/usr/bin/env python3
import sys
import json

"#,
    );

    // Add imports (deduplicated, sorted)
    let mut import_vec: Vec<_> = imports.into_iter().collect();
    import_vec.sort();
    for import in import_vec {
        code.push_str(&import);
        code.push('\n');
    }
    code.push('\n');

    // Add constants (deduplicated, sorted)
    let mut const_vec: Vec<_> = constants.into_iter().collect();
    const_vec.sort();
    if !const_vec.is_empty() {
        for constant in const_vec {
            code.push_str(&constant);
            code.push('\n');
        }
        code.push('\n');
    }

    // Add classes (deduplicated)
    let mut class_vec: Vec<_> = classes.into_iter().collect();
    class_vec.sort();
    for class in class_vec {
        code.push_str(&class);
        code.push_str("\n\n");
    }

    // Add functions
    for func in functions {
        code.push_str(&func);
        code.push_str("\n\n");
    }

    // Generate main
    code.push_str(
        r#"if __name__ == "__main__":
    input_data = json.loads(sys.argv[1]) if len(sys.argv) > 1 else {}

"#,
    );

    // Chain task calls
    let mut is_first = true;
    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            entry_point: Some(entry),
            ..
        } = &module.value
        {
            if is_first {
                code.push_str(&format!("    result = {}(input_data)\n", entry));
                is_first = false;
            } else {
                code.push_str(&format!("    result = {}(result)\n", entry));
            }
        }
    }

    code.push_str(
        r#"
    print(json.dumps(result, indent=2))
"#,
    );

    Ok(code)
}

fn generate_python_readme(spec: &OpenFlowSpec) -> String {
    format!(
        r#"# {}

Standalone workflow generated by sayiir-openflow.

## Install

```bash
pip install -e .
```

## Run

```bash
# Run with default input
python main.py

# Run with JSON input
python main.py '{{"key": "value"}}'
```

## Tasks

{}
"#,
        spec.summary,
        spec.value
            .modules
            .iter()
            .map(|m| format!("- {}", m.id))
            .collect::<Vec<_>>()
            .join("\n")
    )
}
fn generate_node_package_json(
    summary: &str,
    deps: &serde_json::Map<String, serde_json::Value>,
) -> String {
    let pkg = serde_json::json!({
        "name": "workflow",
        "version": "0.1.0",
        "description": summary,
        "main": "index.js",
        "dependencies": deps
    });

    serde_json::to_string_pretty(&pkg).unwrap()
}

fn generate_node_main(spec: &OpenFlowSpec) -> Result<String> {
    use std::collections::HashSet;

    let mut imports = HashSet::new();
    let mut constants = HashSet::new();
    let mut functions = Vec::new();

    // Parse each task's code to extract imports, constants, and functions
    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            code: Some(task_code),
            ..
        } = &module.value
        {
            // Split by lines and categorize
            for line in task_code.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("import ") {
                    imports.insert(line.to_string());
                } else if trimmed.starts_with("const ") && !trimmed.contains("function") {
                    constants.insert(line.to_string());
                } else if trimmed.starts_with("function ") || (!trimmed.is_empty() && !trimmed.starts_with("import") && !trimmed.starts_with("const")) {
                    functions.push(line.to_string());
                }
            }
        }
    }

    // Build final code with deduplicated imports
    let mut code = String::new();

    // Add imports (deduplicated)
    let mut import_vec: Vec<_> = imports.into_iter().collect();
    import_vec.sort();
    for import in import_vec {
        code.push_str(&import);
        code.push('\n');
    }
    code.push('\n');

    // Add constants (deduplicated)
    let mut const_vec: Vec<_> = constants.into_iter().collect();
    const_vec.sort();
    for constant in const_vec {
        code.push_str(&constant);
        code.push('\n');
    }
    code.push('\n');

    // Add functions
    code.push_str(&functions.join("\n"));
    code.push_str("\n\n");

    // Generate main
    code.push_str(
        r#"const input = process.argv[2] ? JSON.parse(process.argv[2]) : {};

(async () => {
"#,
    );

    // Chain task calls
    let mut is_first = true;
    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            entry_point: Some(entry),
            ..
        } = &module.value
        {
            if is_first {
                code.push_str(&format!("    let result = await {}(input);\n", entry));
                is_first = false;
            } else {
                code.push_str(&format!("    result = await {}(result);\n", entry));
            }
        }
    }

    code.push_str(
        r#"
    console.log(JSON.stringify(result, null, 2));
})();
"#,
    );

    Ok(code)
}

fn generate_node_readme(spec: &OpenFlowSpec) -> String {
    format!(
        r#"# {}

Standalone workflow generated by sayiir-openflow.

## Install

```bash
npm install
```

## Run

```bash
# Run with default input
node index.js

# Run with JSON input
node index.js '{{"key": "value"}}'
```

## Tasks

{}
"#,
        spec.summary,
        spec.value
            .modules
            .iter()
            .map(|m| format!("- {}", m.id))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn generate_multi_readme(spec: &OpenFlowSpec, languages: &HashSet<String>) -> String {
    let subdirs = languages
        .iter()
        .map(|lang| format!("- `{}_tasks/` - {} tasks", lang, lang))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"# {}

Multi-language workflow generated by sayiir-openflow.

## Structure

{}

## Build

Build each language subdirectory separately:

```bash
cd rust_tasks && cargo build
cd python_tasks && pip install -e .
cd node_tasks && npm install
```
"#,
        spec.summary, subdirs
    )
}

/// Extract function name from tree-sitter node
fn extract_fn_name(func_node: &tree_sitter::Node, source: &str) -> Option<String> {
    for child in func_node.children(&mut func_node.walk()) {
        if child.kind() == "identifier" {
            if let Ok(name) = child.utf8_text(source.as_bytes()) {
                return Some(name.to_string());
            }
        }
    }
    None
}
