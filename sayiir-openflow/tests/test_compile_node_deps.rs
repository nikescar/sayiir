use sayiir_openflow::*;
use serde_json::json;

#[tokio::test]
async fn test_compile_node_with_dependencies() {
    let mut deps = serde_json::Map::new();
    deps.insert("axios".to_string(), json!("^1.6.0"));

    let module = OpenFlowModule {
        id: "node_deps_task".to_string(),
        value: OpenFlowModuleValue::Script {
            path: "node_deps_task".to_string(),
            language: Some("node".to_string()),
            entry_point: Some("run".to_string()),
            code: Some(
                r#"
const axios = require('axios');

async function run(input) {
    return { hasAxios: typeof axios !== 'undefined' };
}
"#
                .to_string(),
            ),
            dependencies: Some(deps),
        },
    };

    let cached = compile_module(&module, "test_workflow").await.unwrap();

    // Verify package.json exists and contains dependency
    let package_json_path = cached.cache_path.join("package.json");
    assert!(package_json_path.exists());

    let package_json = std::fs::read_to_string(package_json_path).unwrap();
    assert!(package_json.contains("\"axios\""));
    assert!(package_json.contains("^1.6.0"));

    // Verify node_modules installed
    let node_modules = cached.cache_path.join("node_modules");
    assert!(node_modules.exists());

    // Cleanup
    std::fs::remove_dir_all(cached.cache_path).ok();
}

#[tokio::test]
async fn test_execute_node_with_dependencies() {
    let mut deps = serde_json::Map::new();
    deps.insert("axios".to_string(), json!("^1.6.0"));

    let module = OpenFlowModule {
        id: "axios_task".to_string(),
        value: OpenFlowModuleValue::Script {
            path: "axios_task".to_string(),
            language: Some("node".to_string()),
            entry_point: Some("run".to_string()),
            code: Some(
                r#"
const axios = require('axios');

async function run(input) {
    return { hasAxios: typeof axios !== 'undefined', axiosVersion: axios.VERSION };
}
"#
                .to_string(),
            ),
            dependencies: Some(deps),
        },
    };

    let cached = compile_module(&module, "test_workflow").await.unwrap();

    let input = json!({});
    let output = execute_task(&cached, "node", input).await.unwrap();

    assert_eq!(output["hasAxios"], true);
    assert!(output["axiosVersion"].is_string());

    // Cleanup
    std::fs::remove_dir_all(cached.cache_path).ok();
}
