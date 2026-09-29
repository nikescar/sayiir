//! Run OpenFlow workflows directly with caching.

use crate::{compile_module, execute_task_with_timeout, OpenFlowError, OpenFlowModuleValue, OpenFlowSpec, Result};
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

/// Run workflow with default timeout
pub async fn run_workflow(spec: &OpenFlowSpec, input: Value) -> Result<Value> {
    run_workflow_with_timeout(spec, input, Duration::from_secs(30)).await
}

/// Run workflow with custom timeout per task
pub async fn run_workflow_with_timeout(
    spec: &OpenFlowSpec,
    input: Value,
    timeout: Duration,
) -> Result<Value> {
    let workflow_id = compute_workflow_id(spec);
    let mut current_input = input;

    for module in &spec.value.modules {
        // Get language for this module
        let language = match &module.value {
            OpenFlowModuleValue::Script {
                language: Some(lang),
                ..
            } => lang.clone(),
            _ => {
                return Err(OpenFlowError::InvalidWorkflow(format!(
                    "module '{}': missing language",
                    module.id
                )))
            }
        };

        // Compile module (uses cache if available)
        let cached = compile_module(module, &workflow_id).await?;

        // Execute task
        let result = execute_task_with_timeout(&cached, &language, current_input, timeout).await?;

        // Chain output to next input
        current_input = result;
    }

    Ok(current_input)
}

fn compute_workflow_id(spec: &OpenFlowSpec) -> String {
    let mut hasher = DefaultHasher::new();
    spec.summary.hash(&mut hasher);
    // Hash module structure for cache invalidation on workflow changes
    for module in &spec.value.modules {
        module.id.hash(&mut hasher);
        let OpenFlowModuleValue::Script {
            language,
            code,
            entry_point,
            ..
        } = &module.value;

        language.hash(&mut hasher);
        code.hash(&mut hasher);
        entry_point.hash(&mut hasher);
    }
    format!("{:x}", hasher.finish())
}
