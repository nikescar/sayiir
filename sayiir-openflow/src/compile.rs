use crate::{OpenFlowError, OpenFlowModule, OpenFlowModuleValue, Result};
use std::path::PathBuf;
use std::process::Command;

/// Cached compiled module
#[derive(Debug, Clone)]
pub struct CachedModule {
    /// Path to cache directory
    pub cache_path: PathBuf,
    /// Path to executable (binary for Rust, script for Node.js/Python)
    pub executable: PathBuf,
}

/// Compile a module with embedded code
pub async fn compile_module(module: &OpenFlowModule, workflow_id: &str) -> Result<CachedModule> {
    let OpenFlowModuleValue::Script { language: Some(lang), code: Some(code), entry_point: Some(entry), .. } = &module.value else {
        return Err(OpenFlowError::InvalidWorkflow(
            format!("module '{}': missing language, code, or entry_point", module.id)
        ));
    };

    match lang.as_str() {
        "rust" => compile_rust(module, workflow_id, code, entry).await,
        "node" => compile_node(module, workflow_id, code, entry).await,
        "python" => compile_python(module, workflow_id, code, entry).await,
        _ => Err(OpenFlowError::Unsupported(format!("language: {}", lang))),
    }
}

async fn compile_rust(
    module: &OpenFlowModule,
    workflow_id: &str,
    code: &str,
    entry_point: &str,
) -> Result<CachedModule> {
    // Create cache directory
    let cache_dir = get_cache_dir(workflow_id, &module.id, "rust")?;
    std::fs::create_dir_all(&cache_dir)?;

    // Write Cargo.toml
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde_json = "1.0"
"#, module.id);

    std::fs::write(cache_dir.join("Cargo.toml"), cargo_toml)?;

    // Create src directory
    let src_dir = cache_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    // Write main.rs with wrapper
    let main_rs = format!(r#"{code}

fn main() {{
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {{
        eprintln!("Usage: {{}} <json_input>", args[0]);
        std::process::exit(1);
    }}

    let input = serde_json::from_str(&args[1]).expect("invalid JSON input");

    match {entry_point}(input) {{
        Ok(result) => {{
            println!("{{}}", serde_json::to_string(&result).expect("failed to serialize result"));
        }}
        Err(e) => {{
            eprintln!("Error: {{}}", e);
            std::process::exit(1);
        }}
    }}
}}
"#);

    std::fs::write(src_dir.join("main.rs"), main_rs)?;

    // Run cargo build --release
    let output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(&cache_dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(OpenFlowError::CompilationError {
            module_id: module.id.clone(),
            stderr: stderr.to_string(),
        });
    }

    // Get executable path
    let executable = cache_dir.join("target/release").join(&module.id);

    if !executable.exists() {
        return Err(OpenFlowError::CompilationError {
            module_id: module.id.clone(),
            stderr: "compiled binary not found".to_string(),
        });
    }

    Ok(CachedModule {
        cache_path: cache_dir,
        executable,
    })
}

async fn compile_node(
    _module: &OpenFlowModule,
    _workflow_id: &str,
    _code: &str,
    _entry_point: &str,
) -> Result<CachedModule> {
    // Placeholder for Task 5
    Err(OpenFlowError::Unsupported("node compilation not implemented yet".into()))
}

async fn compile_python(
    _module: &OpenFlowModule,
    _workflow_id: &str,
    _code: &str,
    _entry_point: &str,
) -> Result<CachedModule> {
    // Placeholder for Task 6
    Err(OpenFlowError::Unsupported("python compilation not implemented yet".into()))
}

fn get_cache_dir(workflow_id: &str, module_id: &str, language: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| {
        OpenFlowError::InvalidWorkflow("cannot determine home directory".into())
    })?;

    Ok(home
        .join(".sayiir")
        .join("cache")
        .join(format!("{}_{}_{}", workflow_id, module_id, language)))
}
