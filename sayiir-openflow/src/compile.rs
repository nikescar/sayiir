use crate::{OpenFlowError, OpenFlowModule, OpenFlowModuleValue, Result};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

/// Cached compiled module
#[derive(Debug, Clone)]
pub struct CachedModule {
    /// Path to cache directory
    pub cache_path: PathBuf,
    /// Path to executable (binary for Rust, script for Node.js/Python)
    pub executable: PathBuf,
}

/// Retry an async operation with exponential backoff
async fn retry_with_backoff<F, Fut, T, E>(mut f: F, max_attempts: u32) -> std::result::Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, E>>,
{
    let mut attempt = 0;
    loop {
        attempt += 1;
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt >= max_attempts => return Err(e),
            Err(_) => {
                let backoff = Duration::from_secs(2u64.pow(attempt - 1));
                tokio::time::sleep(backoff).await;
            }
        }
    }
}

/// Compile a module with embedded code
pub async fn compile_module(module: &OpenFlowModule, workflow_id: &str) -> Result<CachedModule> {
    let OpenFlowModuleValue::Script {
        language: Some(lang),
        code: Some(code),
        entry_point: Some(entry),
        ..
    } = &module.value
    else {
        return Err(OpenFlowError::InvalidWorkflow(format!(
            "module '{}': missing language, code, or entry_point",
            module.id
        )));
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

    // Extract dependencies
    let dependencies = if let OpenFlowModuleValue::Script { dependencies, .. } = &module.value {
        dependencies.clone().unwrap_or_default()
    } else {
        serde_json::Map::new()
    };

    // Build dependencies section for Cargo.toml
    let mut deps_section = String::from("[dependencies]\nserde_json = \"1.0\"\n");
    for (pkg, version) in dependencies {
        // Skip serde_json since it's already included
        if pkg == "serde_json" {
            continue;
        }
        if let Some(ver_str) = version.as_str() {
            deps_section.push_str(&format!("{} = \"{}\"\n", pkg, ver_str));
        }
    }

    // Write Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

{}
"#,
        module.id, deps_section
    );

    std::fs::write(cache_dir.join("Cargo.toml"), cargo_toml)?;

    // Create src directory
    let src_dir = cache_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    // Write main.rs with wrapper
    let main_rs = format!(
        r#"{code}

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
"#
    );

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
    module: &OpenFlowModule,
    workflow_id: &str,
    code: &str,
    entry_point: &str,
) -> Result<CachedModule> {
    // Create cache directory
    let cache_dir = get_cache_dir(workflow_id, &module.id, "node")?;
    std::fs::create_dir_all(&cache_dir)?;

    // Extract dependencies
    let dependencies = if let OpenFlowModuleValue::Script { dependencies, .. } = &module.value {
        dependencies.clone().unwrap_or_default()
    } else {
        serde_json::Map::new()
    };

    // Write package.json if dependencies exist
    if !dependencies.is_empty() {
        let package_json = serde_json::json!({
            "name": module.id,
            "version": "1.0.0",
            "dependencies": dependencies
        });

        std::fs::write(
            cache_dir.join("package.json"),
            serde_json::to_string_pretty(&package_json)?,
        )?;

        // Run npm install with retry
        let module_id = module.id.clone();
        let cache_dir_clone = cache_dir.clone();

        let result = retry_with_backoff(
            || async {
                let output = tokio::process::Command::new("npm")
                    .arg("install")
                    .arg("--silent")
                    .current_dir(&cache_dir_clone)
                    .output()
                    .await?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    // Extract first failing package from error if possible
                    let dependency = dependencies
                        .keys()
                        .next()
                        .map(|s| s.clone())
                        .unwrap_or_else(|| "unknown".to_string());

                    return Err(OpenFlowError::DependencyError {
                        module_id: module_id.clone(),
                        language: "node".to_string(),
                        dependency,
                        stderr: stderr.to_string(),
                    });
                }
                Ok(())
            },
            3,
        )
        .await;

        result?;
    }

    // Write task.js with wrapper
    let task_js = format!(
        r#"{code}

// Wrapper
const input = JSON.parse(process.argv[2]);

Promise.resolve({entry_point}(input))
    .then(result => {{
        console.log(JSON.stringify(result));
        process.exit(0);
    }})
    .catch(err => {{
        console.error(err.message);
        process.exit(1);
    }});
"#
    );

    let task_path = cache_dir.join("task.js");
    std::fs::write(&task_path, task_js)?;

    Ok(CachedModule {
        cache_path: cache_dir,
        executable: task_path,
    })
}

async fn compile_python(
    module: &OpenFlowModule,
    workflow_id: &str,
    code: &str,
    entry_point: &str,
) -> Result<CachedModule> {
    // Create cache directory
    let cache_dir = get_cache_dir(workflow_id, &module.id, "python")?;
    std::fs::create_dir_all(&cache_dir)?;

    // Extract dependencies
    let dependencies = if let OpenFlowModuleValue::Script { dependencies, .. } = &module.value {
        dependencies.clone().unwrap_or_default()
    } else {
        serde_json::Map::new()
    };

    // Write requirements.txt if dependencies exist
    if !dependencies.is_empty() {
        let mut requirements = String::new();
        for (pkg, version) in &dependencies {
            if let Some(ver_str) = version.as_str() {
                requirements.push_str(&format!("{}=={}\n", pkg, ver_str));
            }
        }

        std::fs::write(cache_dir.join("requirements.txt"), requirements)?;

        // Create virtual environment
        let venv_output = tokio::process::Command::new("python3")
            .arg("-m")
            .arg("venv")
            .arg("venv")
            .current_dir(&cache_dir)
            .output()
            .await?;

        if !venv_output.status.success() {
            let stderr = String::from_utf8_lossy(&venv_output.stderr);
            return Err(OpenFlowError::CompilationError {
                module_id: module.id.clone(),
                stderr: format!("venv creation failed: {}", stderr),
            });
        }

        // Install dependencies using venv pip with retry
        let pip_path = cache_dir.join("venv/bin/pip");
        let module_id = module.id.clone();
        let cache_dir_clone = cache_dir.clone();

        let result = retry_with_backoff(
            || async {
                let pip_output = tokio::process::Command::new(&pip_path)
                    .arg("install")
                    .arg("-q")
                    .arg("-r")
                    .arg("requirements.txt")
                    .current_dir(&cache_dir_clone)
                    .output()
                    .await?;

                if !pip_output.status.success() {
                    let stderr = String::from_utf8_lossy(&pip_output.stderr);
                    // Extract first failing package from error if possible
                    let dependency = dependencies
                        .keys()
                        .next()
                        .map(|s| s.clone())
                        .unwrap_or_else(|| "unknown".to_string());

                    return Err(OpenFlowError::DependencyError {
                        module_id: module_id.clone(),
                        language: "python".to_string(),
                        dependency,
                        stderr: stderr.to_string(),
                    });
                }
                Ok(())
            },
            3,
        )
        .await;

        result?;
    }

    // Write task.py with wrapper
    let task_py = format!(
        r#"#!/usr/bin/env python3
import json
import sys

{code}

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python task.py <json_input>", file=sys.stderr)
        sys.exit(1)

    input_data = json.loads(sys.argv[1])

    try:
        result = {entry_point}(input_data)
        print(json.dumps(result))
        sys.exit(0)
    except Exception as e:
        print(str(e), file=sys.stderr)
        sys.exit(1)
"#
    );

    let task_path = cache_dir.join("task.py");
    std::fs::write(&task_path, task_py)?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&task_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&task_path, perms)?;
    }

    Ok(CachedModule {
        cache_path: cache_dir,
        executable: task_path,
    })
}

fn get_cache_dir(workflow_id: &str, module_id: &str, language: &str) -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| OpenFlowError::InvalidWorkflow("cannot determine home directory".into()))?;

    Ok(home
        .join(".sayiir")
        .join("cache")
        .join(format!("{}_{}_{}", workflow_id, module_id, language)))
}

/// Cleanup cache directories older than the specified number of days
///
/// # Arguments
/// * `cache_root` - Optional custom cache root directory. If None, uses default ~/.sayiir/cache
/// * `days_threshold` - Remove caches older than this many days
pub fn cleanup_stale_cache(cache_root: Option<PathBuf>, days_threshold: u64) -> Result<()> {
    use std::time::{Duration, SystemTime};

    let cache_dir = if let Some(root) = cache_root {
        root
    } else {
        let home = dirs::home_dir().ok_or_else(|| {
            OpenFlowError::InvalidWorkflow("cannot determine home directory".into())
        })?;
        home.join(".sayiir").join("cache")
    };

    if !cache_dir.exists() {
        return Ok(());
    }

    let threshold = SystemTime::now() - Duration::from_secs(days_threshold * 24 * 3600);

    for entry in std::fs::read_dir(&cache_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let metadata = std::fs::metadata(&path)?;
            if let Ok(modified) = metadata.modified() {
                if modified < threshold {
                    // Remove stale cache directory
                    std::fs::remove_dir_all(&path)?;
                }
            }
        }
    }

    Ok(())
}
