use sayiir_openflow::*;
use serde_json::json;

#[tokio::test]
async fn test_run_workflow_simple() {
    let spec = OpenFlowSpec {
        summary: "Add workflow".to_string(),
        value: OpenFlowValue {
            modules: vec![OpenFlowModule {
                id: "add_task".to_string(),
                value: OpenFlowModuleValue::Script {
                    path: "add_task".to_string(),
                    language: Some("rust".to_string()),
                    entry_point: Some("add".to_string()),
                    code: Some(
                        r#"fn add(input: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let a = input["a"].as_i64().unwrap_or(0);
    let b = input["b"].as_i64().unwrap_or(0);
    Ok(serde_json::json!({"result": a + b}))
}"#
                        .to_string(),
                    ),
                    dependencies: None,
                },
            }],
        },
    };

    let input = json!({"a": 5, "b": 3});
    let result = run_workflow(&spec, input).await.unwrap();

    assert_eq!(result["result"], 8);
}

#[tokio::test]
async fn test_run_workflow_chained() {
    let spec = OpenFlowSpec {
        summary: "Chained workflow".to_string(),
        value: OpenFlowValue {
            modules: vec![
                OpenFlowModule {
                    id: "double".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "double".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("double".to_string()),
                        code: Some(
                            r#"fn double(input: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let n = input["n"].as_i64().unwrap_or(0);
    Ok(serde_json::json!({"n": n * 2}))
}"#
                            .to_string(),
                        ),
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "add_ten".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "add_ten".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("add_ten".to_string()),
                        code: Some(
                            r#"fn add_ten(input: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let n = input["n"].as_i64().unwrap_or(0);
    Ok(serde_json::json!({"n": n + 10}))
}"#
                            .to_string(),
                        ),
                        dependencies: None,
                    },
                },
            ],
        },
    };

    let input = json!({"n": 5});
    let result = run_workflow(&spec, input).await.unwrap();

    // 5 * 2 = 10, then 10 + 10 = 20
    assert_eq!(result["n"], 20);
}

#[tokio::test]
async fn test_run_workflow_cache_reuse() {
    let spec = OpenFlowSpec {
        summary: "Cache test".to_string(),
        value: OpenFlowValue {
            modules: vec![OpenFlowModule {
                id: "identity".to_string(),
                value: OpenFlowModuleValue::Script {
                    path: "identity".to_string(),
                    language: Some("rust".to_string()),
                    entry_point: Some("identity".to_string()),
                    code: Some(
                        r#"fn identity(input: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    Ok(input)
}"#
                        .to_string(),
                    ),
                    dependencies: None,
                },
            }],
        },
    };

    // First run - should compile
    let input1 = json!({"data": "first"});
    let result1 = run_workflow(&spec, input1.clone()).await.unwrap();
    assert_eq!(result1["data"], "first");

    // Second run - should use cache
    let input2 = json!({"data": "second"});
    let result2 = run_workflow(&spec, input2.clone()).await.unwrap();
    assert_eq!(result2["data"], "second");

    // Cache directory should exist
    let cache_dir = get_workflow_cache_dir(&spec).unwrap();
    assert!(cache_dir.exists(), "Cache directory should exist after first run");
}

fn get_workflow_cache_dir(spec: &OpenFlowSpec) -> std::io::Result<std::path::PathBuf> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    spec.summary.hash(&mut hasher);
    for module in &spec.value.modules {
        module.id.hash(&mut hasher);
    }
    let workflow_id = format!("{:x}", hasher.finish());

    let home = dirs::home_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "home dir not found")
    })?;
    Ok(home.join(".sayiir").join("cache"))
}
