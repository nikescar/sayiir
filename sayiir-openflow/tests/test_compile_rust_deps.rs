use sayiir_openflow::*;
use serde_json::json;

#[tokio::test]
async fn test_compile_rust_with_dependencies() {
    let mut deps = serde_json::Map::new();
    deps.insert("serde_json".to_string(), json!("1.0"));

    let module = OpenFlowModule {
        id: "deps_task".to_string(),
        value: OpenFlowModuleValue::Script {
            path: "deps_task".to_string(),
            language: Some("rust".to_string()),
            entry_point: Some("run".to_string()),
            code: Some(
                r#"
use serde_json::{json, Value};

fn run(input: Value) -> Result<Value, String> {
    Ok(json!({"deps": "work"}))
}
"#
                .to_string(),
            ),
            dependencies: Some(deps),
        },
    };

    let cached = compile_module(&module, "test_workflow").await.unwrap();

    // Verify Cargo.toml contains dependency
    let cargo_toml_path = cached.cache_path.join("Cargo.toml");
    let cargo_toml = std::fs::read_to_string(cargo_toml_path).unwrap();
    assert!(cargo_toml.contains("serde_json = \"1.0\""));

    // Verify binary compiled successfully
    assert!(cached.executable.exists());

    // Cleanup
    std::fs::remove_dir_all(cached.cache_path).ok();
}

#[tokio::test]
async fn test_execute_rust_with_dependencies() {
    let mut deps = serde_json::Map::new();
    deps.insert("chrono".to_string(), json!("0.4"));

    let module = OpenFlowModule {
        id: "chrono_task".to_string(),
        value: OpenFlowModuleValue::Script {
            path: "chrono_task".to_string(),
            language: Some("rust".to_string()),
            entry_point: Some("run".to_string()),
            code: Some(
                r#"
use serde_json::{json, Value};
use chrono::Utc;

fn run(_input: Value) -> Result<Value, String> {
    let now = Utc::now();
    Ok(json!({"timestamp": now.to_rfc3339()}))
}
"#
                .to_string(),
            ),
            dependencies: Some(deps),
        },
    };

    let cached = compile_module(&module, "test_workflow").await.unwrap();

    let input = json!({});
    let output = execute_task(&cached, "rust", input).await.unwrap();

    assert!(output["timestamp"].is_string());

    // Cleanup
    std::fs::remove_dir_all(cached.cache_path).ok();
}
