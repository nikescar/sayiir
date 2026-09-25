//! Import OpenFlow workflows and generate standalone executable projects.

use crate::{OpenFlowError, OpenFlowModule, OpenFlowModuleValue, OpenFlowSpec, Result};
use std::collections::{HashMap, HashSet};
use std::path::Path;

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
    let mut code = String::from(
        r#"use serde_json::Value;

"#,
    );

    // Embed all task functions (already cleaned by Tree-sitter during export)
    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            code: Some(task_code),
            ..
        } = &module.value
        {
            code.push_str(task_code);
            code.push_str("\n\n");
        }
    }

    // Generate async main function with tokio runtime
    code.push_str(
        r#"#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let input = if args.len() > 1 {
        serde_json::from_str(&args[1])?
    } else {
        serde_json::json!({})
    };

"#,
    );

    // Chain task calls with .await for async functions
    let mut is_first = true;
    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            entry_point: Some(entry),
            ..
        } = &module.value
        {
            if is_first {
                code.push_str(&format!(
                    "    let result = {}(input).await.expect(\"Task '{}' failed\");\n",
                    entry, module.id
                ));
                is_first = false;
            } else {
                code.push_str(&format!(
                    "    let result = {}(result).await.expect(\"Task '{}' failed\");\n",
                    entry, module.id
                ));
            }
        }
    }

    code.push_str(
        r#"
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
"#,
    );

    Ok(code)
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
    let mut code = String::from(
        r#"#!/usr/bin/env python3
import sys
import json

"#,
    );

    // Embed all task functions (already cleaned by Tree-sitter during export)
    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            code: Some(task_code),
            ..
        } = &module.value
        {
            code.push_str(task_code);
            code.push_str("\n\n");
        }
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
    let mut code = String::new();

    // Embed all task functions (already cleaned by Tree-sitter during export)
    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            code: Some(task_code),
            ..
        } = &module.value
        {
            code.push_str(task_code);
            code.push_str("\n\n");
        }
    }

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
