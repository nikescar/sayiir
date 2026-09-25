use sayiir_openflow::*;

#[tokio::test]
async fn test_compile_python_module() {
    let module = OpenFlowModule {
        id: "test_python_task".to_string(),
        value: OpenFlowModuleValue::Script {
            path: "test_python_task".to_string(),
            language: Some("python".to_string()),
            entry_point: Some("run".to_string()),
            code: Some(r#"
def run(input_data):
    return {"result": "ok", "input": input_data}
"#.to_string()),
        },
    };

    let cached = compile_module(&module, "test_workflow").await.unwrap();

    // Verify task.py exists
    assert!(cached.executable.exists());
    assert!(cached.executable.to_str().unwrap().ends_with("task.py"));

    // Cleanup
    std::fs::remove_dir_all(cached.cache_path).ok();
}
