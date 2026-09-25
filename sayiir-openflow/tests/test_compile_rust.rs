use sayiir_openflow::*;

#[tokio::test]
async fn test_compile_rust_module() {
    let module = OpenFlowModule {
        id: "test_task".to_string(),
        value: OpenFlowModuleValue::Script {
            path: "test_task".to_string(),
            language: Some("rust".to_string()),
            entry_point: Some("run".to_string()),
            code: Some(
                r#"
use serde_json::Value;

fn run(input: Value) -> Result<Value, String> {
    Ok(serde_json::json!({"result": "ok"}))
}
"#
                .to_string(),
            ),
            dependencies: None,
        },
    };

    let cached = compile_module(&module, "test_workflow").await.unwrap();

    // Verify binary exists
    assert!(cached.executable.exists());
    assert!(cached.executable.is_file());

    // Cleanup
    std::fs::remove_dir_all(cached.cache_path).ok();
}

#[tokio::test]
async fn test_compile_rust_syntax_error() {
    let module = OpenFlowModule {
        id: "broken_task".to_string(),
        value: OpenFlowModuleValue::Script {
            path: "broken_task".to_string(),
            language: Some("rust".to_string()),
            entry_point: Some("run".to_string()),
            code: Some("fn run( { invalid syntax }".to_string()),
            dependencies: None,
        },
    };

    let result = compile_module(&module, "test_workflow").await;
    assert!(result.is_err());

    if let Err(OpenFlowError::CompilationError { stderr, .. }) = result {
        assert!(stderr.contains("error"));
    } else {
        panic!("Expected CompilationError");
    }
}

#[tokio::test]
async fn test_execute_rust_task() {
    let module = OpenFlowModule {
        id: "echo_task".to_string(),
        value: OpenFlowModuleValue::Script {
            path: "echo_task".to_string(),
            language: Some("rust".to_string()),
            entry_point: Some("run".to_string()),
            code: Some(
                r#"
use serde_json::Value;

fn run(input: Value) -> Result<Value, String> {
    Ok(serde_json::json!({"result": "ok", "input": input}))
}
"#
                .to_string(),
            ),
            dependencies: None,
        },
    };

    let cached = compile_module(&module, "test_workflow").await.unwrap();

    let input = serde_json::json!({"test": "data"});
    let output = execute_task(&cached, "rust", input.clone()).await.unwrap();

    assert_eq!(output["result"], "ok");
    assert_eq!(output["input"], input);

    // Cleanup
    std::fs::remove_dir_all(cached.cache_path).ok();
}
