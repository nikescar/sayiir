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
