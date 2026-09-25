use sayiir_openflow::*;

#[tokio::test]
async fn test_compile_node_module() {
    let module = OpenFlowModule {
        id: "test_node_task".to_string(),
        value: OpenFlowModuleValue::Script {
            path: "test_node_task".to_string(),
            language: Some("node".to_string()),
            entry_point: Some("run".to_string()),
            code: Some(r#"
async function run(input) {
    return { result: "ok", input: input };
}
"#.to_string()),
        },
    };

    let cached = compile_module(&module, "test_workflow").await.unwrap();

    // Verify task.js exists
    assert!(cached.executable.exists());
    assert!(cached.executable.to_str().unwrap().ends_with("task.js"));

    // Cleanup
    std::fs::remove_dir_all(cached.cache_path).ok();
}

#[tokio::test]
async fn test_execute_node_task() {
    let module = OpenFlowModule {
        id: "echo_task".to_string(),
        value: OpenFlowModuleValue::Script {
            path: "echo_task".to_string(),
            language: Some("node".to_string()),
            entry_point: Some("run".to_string()),
            code: Some(r#"
async function run(input) {
    return { result: "ok", input: input };
}
"#.to_string()),
        },
    };

    let cached = compile_module(&module, "test_workflow").await.unwrap();

    let input = serde_json::json!({"test": "data"});
    let output = execute_task(&cached, "node", input.clone()).await.unwrap();

    assert_eq!(output["result"], "ok");
    assert_eq!(output["input"], input);

    // Cleanup
    std::fs::remove_dir_all(cached.cache_path).ok();
}
