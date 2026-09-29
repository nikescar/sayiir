use sayiir_openflow::*;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_import_mono_rust_workflow() {
    let spec = OpenFlowSpec {
        summary: "Test workflow".to_string(),
        value: OpenFlowValue {
            modules: vec![
                OpenFlowModule {
                    id: "add_task".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "add_task".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("add".to_string()),
                        code: Some("fn add(input: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> { Ok(serde_json::json!({\"result\": 5})) }".to_string()),
                        dependencies: None,
                    },
                },
            ],
        },
    };

    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().to_path_buf();

    // Import workflow - should generate standalone project
    import_workflow(&spec, &output_dir).await.unwrap();

    // Verify Cargo.toml exists
    let cargo_toml = output_dir.join("Cargo.toml");
    assert!(cargo_toml.exists(), "Cargo.toml should exist");

    // Verify src/main.rs exists
    let main_rs = output_dir.join("src/main.rs");
    assert!(main_rs.exists(), "src/main.rs should exist");

    // Verify README.md exists
    let readme = output_dir.join("README.md");
    assert!(readme.exists(), "README.md should exist");

    // Verify project compiles
    let output = std::process::Command::new("cargo")
        .arg("build")
        .current_dir(&output_dir)
        .output()
        .unwrap();

    assert!(output.status.success(), "Project should compile successfully");
}

#[tokio::test]
async fn test_import_mono_rust_with_deps() {
    let mut deps = serde_json::Map::new();
    deps.insert("regex".to_string(), serde_json::json!("1.10"));

    let spec = OpenFlowSpec {
        summary: "Workflow with deps".to_string(),
        value: OpenFlowValue {
            modules: vec![
                OpenFlowModule {
                    id: "task1".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "task1".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some("fn run(input: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> { Ok(input) }".to_string()),
                        dependencies: Some(deps),
                    },
                },
            ],
        },
    };

    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().to_path_buf();

    import_workflow(&spec, &output_dir).await.unwrap();

    // Verify dependencies in Cargo.toml
    let cargo_toml = std::fs::read_to_string(output_dir.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("regex = \"1.10\""), "Should include regex dependency");
}
