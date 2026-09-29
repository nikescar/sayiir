use sayiir_openflow::*;
use serde_json::json;

/// Integration test: Multi-language workflow with dependencies
#[tokio::test]
async fn test_multi_language_workflow_with_dependencies() {
    let mut rust_deps = serde_json::Map::new();
    rust_deps.insert("chrono".to_string(), json!("0.4"));

    let mut node_deps = serde_json::Map::new();
    node_deps.insert("axios".to_string(), json!("^1.6.0"));

    let mut python_deps = serde_json::Map::new();
    python_deps.insert("requests".to_string(), json!("2.31.0"));

    let spec = OpenFlowSpec {
        summary: "Multi-language workflow with dependencies".to_string(),
        value: OpenFlowValue {
            modules: vec![
                OpenFlowModule {
                    id: "rust_timestamp".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "rust_timestamp".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(
                            r#"
use serde_json::{json, Value};
use chrono::Utc;

fn run(_input: Value) -> Result<Value, String> {
    let now = Utc::now();
    Ok(json!({"timestamp": now.to_rfc3339(), "language": "rust"}))
}
"#
                            .to_string(),
                        ),
                        dependencies: Some(rust_deps),
                    },
                },
                OpenFlowModule {
                    id: "node_check".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "node_check".to_string(),
                        language: Some("node".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(
                            r#"
const axios = require('axios');

async function run(input) {
    return {
        hasAxios: typeof axios !== 'undefined',
        axiosVersion: axios.VERSION,
        language: 'node',
        prevTimestamp: input.timestamp
    };
}
"#
                            .to_string(),
                        ),
                        dependencies: Some(node_deps),
                    },
                },
                OpenFlowModule {
                    id: "python_check".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "python_check".to_string(),
                        language: Some("python".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(
                            r#"
import requests

def run(input_data):
    return {
        "has_requests": True,
        "version": requests.__version__,
        "language": "python",
        "prev_axios_version": input_data.get("axiosVersion")
    }
"#
                            .to_string(),
                        ),
                        dependencies: Some(python_deps),
                    },
                },
            ],
        },
    };

    // Export and re-import to verify serialization
    let json_str = export_openflow_json(&spec).unwrap();
    let imported = import_openflow_json(&json_str).unwrap();

    // Compile all modules
    let rust_cached = compile_module(&imported.value.modules[0], "integration_test")
        .await
        .unwrap();
    let node_cached = compile_module(&imported.value.modules[1], "integration_test")
        .await
        .unwrap();
    let python_cached = compile_module(&imported.value.modules[2], "integration_test")
        .await
        .unwrap();

    // Execute in sequence (simulating workflow chaining)
    let rust_output = execute_task(&rust_cached, "rust", json!({})).await.unwrap();
    assert_eq!(rust_output["language"], "rust");
    assert!(rust_output["timestamp"].is_string());

    let node_output = execute_task(&node_cached, "node", rust_output)
        .await
        .unwrap();
    assert_eq!(node_output["language"], "node");
    assert_eq!(node_output["hasAxios"], true);
    assert!(node_output["axiosVersion"].is_string());
    assert!(node_output["prevTimestamp"].is_string());

    let python_output = execute_task(&python_cached, "python", node_output)
        .await
        .unwrap();
    assert_eq!(python_output["language"], "python");
    assert_eq!(python_output["has_requests"], true);
    assert!(python_output["version"].is_string());
    assert!(python_output["prev_axios_version"].is_string());

    // Cleanup
    std::fs::remove_dir_all(rust_cached.cache_path).ok();
    std::fs::remove_dir_all(node_cached.cache_path).ok();
    std::fs::remove_dir_all(python_cached.cache_path).ok();
}

/// Integration test: Verify dependencies field is optional
#[tokio::test]
async fn test_workflow_without_dependencies() {
    let spec = OpenFlowSpec {
        summary: "Workflow without dependencies".to_string(),
        value: OpenFlowValue {
            modules: vec![OpenFlowModule {
                id: "simple_task".to_string(),
                value: OpenFlowModuleValue::Script {
                    path: "simple_task".to_string(),
                    language: Some("python".to_string()),
                    entry_point: Some("run".to_string()),
                    code: Some(
                        r#"
def run(input_data):
    return {"result": "ok"}
"#
                        .to_string(),
                    ),
                    dependencies: None,
                },
            }],
        },
    };

    let json_str = export_openflow_json(&spec).unwrap();
    let imported = import_openflow_json(&json_str).unwrap();

    let cached = compile_module(&imported.value.modules[0], "integration_test")
        .await
        .unwrap();

    let output = execute_task(&cached, "python", json!({})).await.unwrap();
    assert_eq!(output["result"], "ok");

    std::fs::remove_dir_all(cached.cache_path).ok();
}
