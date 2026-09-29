use crate::compile::CachedModule;
use crate::{OpenFlowError, Result};
use serde_json::Value;
use std::time::Duration;

/// Default task timeout
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Execute a compiled task
pub async fn execute_task(cached: &CachedModule, language: &str, input: Value) -> Result<Value> {
    execute_task_with_timeout(
        cached,
        language,
        input,
        Duration::from_secs(DEFAULT_TIMEOUT_SECS),
    )
    .await
}

/// Execute a task with custom timeout
pub async fn execute_task_with_timeout(
    cached: &CachedModule,
    language: &str,
    input: Value,
    timeout: Duration,
) -> Result<Value> {
    let input_json = serde_json::to_string(&input)?;

    let (command, args): (String, Vec<String>) = match language {
        "rust" => (
            cached.executable.display().to_string(),
            vec![input_json.clone()],
        ),
        "node" => (
            "node".to_string(),
            vec![
                cached.executable.display().to_string(),
                input_json.clone(),
            ],
        ),
        "python" => {
            // Use venv python if it exists
            let venv_python = cached.cache_path.join("venv/bin/python3");
            let python_cmd = if venv_python.exists() {
                venv_python.display().to_string()
            } else {
                "python3".to_string()
            };
            (
                python_cmd,
                vec![
                    cached.executable.display().to_string(),
                    input_json.clone(),
                ],
            )
        }
        _ => {
            return Err(OpenFlowError::Unsupported(format!(
                "language: {}",
                language
            )));
        }
    };

    // Spawn process
    let output = tokio::time::timeout(
        timeout,
        tokio::process::Command::new(&command).args(&args).output(),
    )
    .await
    .map_err(|_| OpenFlowError::Timeout {
        seconds: timeout.as_secs(),
    })?
    .map_err(OpenFlowError::IoError)?;

    // Check exit status
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(OpenFlowError::ExecutionError(stderr.to_string()));
    }

    // Parse stdout as JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: Value = serde_json::from_str(&stdout).map_err(|e| {
        OpenFlowError::ExecutionError(format!(
            "failed to parse task output as JSON: {}. Output: {}",
            e, stdout
        ))
    })?;

    Ok(result)
}
