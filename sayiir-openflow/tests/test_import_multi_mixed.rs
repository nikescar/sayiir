use sayiir_openflow::*;
use tempfile::TempDir;

#[tokio::test]
async fn test_import_multi_language_workflow() {
    let spec = OpenFlowSpec {
        summary: "Multi-language workflow".to_string(),
        value: OpenFlowValue {
            modules: vec![
                OpenFlowModule {
                    id: "rust_task".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "rust_task".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("process".to_string()),
                        code: Some("fn process(input: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> { Ok(input) }".to_string()),
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "python_task".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "python_task".to_string(),
                        language: Some("python".to_string()),
                        entry_point: Some("transform".to_string()),
                        code: Some("def transform(data): return data".to_string()),
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "node_task".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "node_task".to_string(),
                        language: Some("node".to_string()),
                        entry_point: Some("format".to_string()),
                        code: Some("async function format(input) { return input; }".to_string()),
                        dependencies: None,
                    },
                },
            ],
        },
    };

    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().to_path_buf();

    // Import workflow - should generate subdirectories
    import_workflow(&spec, &output_dir).await.unwrap();

    // Verify subdirectories exist
    assert!(output_dir.join("rust_tasks").exists(), "rust_tasks/ should exist");
    assert!(output_dir.join("python_tasks").exists(), "python_tasks/ should exist");
    assert!(output_dir.join("node_tasks").exists(), "node_tasks/ should exist");

    // Verify each subdir has its own project files
    assert!(output_dir.join("rust_tasks/Cargo.toml").exists());
    assert!(output_dir.join("rust_tasks/src/main.rs").exists());

    assert!(output_dir.join("python_tasks/pyproject.toml").exists());
    assert!(output_dir.join("python_tasks/main.py").exists());

    assert!(output_dir.join("node_tasks/package.json").exists());
    assert!(output_dir.join("node_tasks/index.js").exists());

    // Verify main README exists
    let readme = output_dir.join("README.md");
    assert!(readme.exists(), "Main README.md should exist");
    let readme_content = std::fs::read_to_string(&readme).unwrap();
    assert!(readme_content.contains("rust_tasks"));
    assert!(readme_content.contains("python_tasks"));
    assert!(readme_content.contains("node_tasks"));
}

#[tokio::test]
async fn test_import_multi_rust_python() {
    let spec = OpenFlowSpec {
        summary: "Rust + Python workflow".to_string(),
        value: OpenFlowValue {
            modules: vec![
                OpenFlowModule {
                    id: "task1".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "task1".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some("fn run(input: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> { Ok(input) }".to_string()),
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "task2".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "task2".to_string(),
                        language: Some("python".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some("def run(data): return data".to_string()),
                        dependencies: None,
                    },
                },
            ],
        },
    };

    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().to_path_buf();

    import_workflow(&spec, &output_dir).await.unwrap();

    // Only rust_tasks and python_tasks should exist (no node_tasks)
    assert!(output_dir.join("rust_tasks").exists());
    assert!(output_dir.join("python_tasks").exists());
    assert!(!output_dir.join("node_tasks").exists());
}
