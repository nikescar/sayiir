use sayiir_openflow::*;
use serde_json::json;

#[tokio::test]
async fn test_compile_python_with_dependencies() {
    let mut deps = serde_json::Map::new();
    deps.insert("requests".to_string(), json!("2.31.0"));

    let module = OpenFlowModule {
        id: "python_deps_task".to_string(),
        value: OpenFlowModuleValue::Script {
            path: "python_deps_task".to_string(),
            language: Some("python".to_string()),
            entry_point: Some("run".to_string()),
            code: Some(
                r#"
import requests

def run(input_data):
    return {"has_requests": True, "version": requests.__version__}
"#
                .to_string(),
            ),
            dependencies: Some(deps),
        },
    };

    let cached = compile_module(&module, "test_workflow").await.unwrap();

    // Verify requirements.txt exists and contains dependency
    let requirements_path = cached.cache_path.join("requirements.txt");
    assert!(requirements_path.exists());

    let requirements = std::fs::read_to_string(requirements_path).unwrap();
    assert!(requirements.contains("requests==2.31.0"));

    // Verify venv created
    let venv_path = cached.cache_path.join("venv");
    assert!(venv_path.exists());

    // Cleanup
    std::fs::remove_dir_all(cached.cache_path).ok();
}

#[tokio::test]
async fn test_execute_python_with_dependencies() {
    let mut deps = serde_json::Map::new();
    deps.insert("requests".to_string(), json!("2.31.0"));

    let module = OpenFlowModule {
        id: "requests_task".to_string(),
        value: OpenFlowModuleValue::Script {
            path: "requests_task".to_string(),
            language: Some("python".to_string()),
            entry_point: Some("run".to_string()),
            code: Some(
                r#"
import requests

def run(input_data):
    return {"has_requests": True, "version": requests.__version__}
"#
                .to_string(),
            ),
            dependencies: Some(deps),
        },
    };

    let cached = compile_module(&module, "test_workflow").await.unwrap();

    let input = json!({});
    let output = execute_task(&cached, "python", input).await.unwrap();

    assert_eq!(output["has_requests"], true);
    assert!(output["version"].is_string());

    // Cleanup
    std::fs::remove_dir_all(cached.cache_path).ok();
}
