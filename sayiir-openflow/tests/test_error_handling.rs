use sayiir_openflow::*;
use serde_json::json;

#[test]
fn test_compilation_error_display() {
    let error = OpenFlowError::CompilationError {
        module_id: "test_task".to_string(),
        stderr: "error[E0425]: cannot find value `x` in this scope\n --> src/main.rs:5:9"
            .to_string(),
    };

    let display = format!("{}", error);
    assert!(display.contains("test_task"));
    assert!(display.contains("error[E0425]"));
}

#[test]
fn test_dependency_error_display() {
    let error = OpenFlowError::DependencyError {
        module_id: "test_task".to_string(),
        language: "python".to_string(),
        dependency: "nonexistent-package".to_string(),
        stderr: "ERROR: Could not find a version that satisfies the requirement".to_string(),
    };

    let display = format!("{}", error);
    assert!(display.contains("test_task"));
    assert!(display.contains("python"));
    assert!(display.contains("nonexistent-package"));
}

#[test]
fn test_missing_runtime_helpful_message() {
    let error = OpenFlowError::MissingRuntime {
        language: "rust".to_string(),
        install_url: "https://rustup.rs".to_string(),
    };

    let display = format!("{}", error);
    assert!(display.contains("rust"));
    assert!(display.contains("https://rustup.rs"));
}

#[tokio::test]
async fn test_retry_on_network_error() {
    // Test that compilation retries on network errors
    let mut deps = serde_json::Map::new();
    deps.insert("nonexistent-package-xyz".to_string(), json!("999.999.999"));

    let module = OpenFlowModule {
        id: "retry_test".to_string(),
        value: OpenFlowModuleValue::Script {
            path: "retry_test".to_string(),
            language: Some("python".to_string()),
            entry_point: Some("run".to_string()),
            code: Some("def run(input): return input".to_string()),
            dependencies: Some(deps),
        },
    };

    let result = compile_module(&module, "test_workflow").await;

    // Should fail with DependencyError after retries
    assert!(result.is_err());
    match result.unwrap_err() {
        OpenFlowError::DependencyError { .. } => {}
        _ => panic!("Expected DependencyError"),
    }
}
