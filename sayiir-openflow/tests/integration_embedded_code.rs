use sayiir_openflow::*;
use serde_json::json;

#[tokio::test]
async fn test_end_to_end_rust_workflow() {
    // 1. Create workflow with embedded code
    let spec = OpenFlowSpec {
        summary: "Test workflow with embedded Rust code".to_string(),
        value: OpenFlowValue {
            modules: vec![OpenFlowModule {
                id: "add_numbers".to_string(),
                value: OpenFlowModuleValue::Script {
                    path: "add_numbers".to_string(),
                    language: Some("rust".to_string()),
                    entry_point: Some("run".to_string()),
                    code: Some(
                        r#"
use serde_json::{json, Value};

fn run(input: Value) -> Result<Value, String> {
    let a = input["a"].as_i64().ok_or("missing a")?;
    let b = input["b"].as_i64().ok_or("missing b")?;
    Ok(json!({"result": a + b}))
}
"#
                        .to_string(),
                    ),
                },
            }],
        },
    };

    // 2. Export to JSON
    let json_str = export_openflow_json(&spec).unwrap();
    assert!(json_str.contains("\"language\": \"rust\""));

    // 3. Import from JSON
    let imported = import_openflow_json(&json_str).unwrap();
    assert_eq!(imported.value.modules.len(), 1);

    // 4. Check runtimes
    check_runtimes(&imported).unwrap();

    // 5. Compile module
    let module = &imported.value.modules[0];
    let cached = compile_module(module, "integration_test").await.unwrap();
    assert!(cached.executable.exists());

    // 6. Execute task
    let input = json!({"a": 10, "b": 32});
    let output = execute_task(&cached, "rust", input).await.unwrap();
    assert_eq!(output["result"], 42);

    // Cleanup
    std::fs::remove_dir_all(cached.cache_path).ok();
}

#[tokio::test]
async fn test_end_to_end_mixed_languages() {
    // Test workflow with Rust, Node.js, and Python tasks
    let spec = OpenFlowSpec {
        summary: "Multi-language workflow".to_string(),
        value: OpenFlowValue {
            modules: vec![
                OpenFlowModule {
                    id: "rust_task".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "rust_task".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(
                            r#"
use serde_json::{json, Value};
fn run(input: Value) -> Result<Value, String> {
    Ok(json!({"from_rust": true, "value": 1}))
}
"#
                            .to_string(),
                        ),
                    },
                },
                OpenFlowModule {
                    id: "node_task".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "node_task".to_string(),
                        language: Some("node".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(
                            r#"
async function run(input) {
    return { from_node: true, value: 2 };
}
"#
                            .to_string(),
                        ),
                    },
                },
                OpenFlowModule {
                    id: "python_task".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "python_task".to_string(),
                        language: Some("python".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(
                            r#"
def run(input_data):
    return {"from_python": True, "value": 3}
"#
                            .to_string(),
                        ),
                    },
                },
            ],
        },
    };

    // Export and re-import
    let json_str = export_openflow_json(&spec).unwrap();
    let imported = import_openflow_json(&json_str).unwrap();

    // Compile and execute all modules
    for module in &imported.value.modules {
        let lang = if let OpenFlowModuleValue::Script {
            language: Some(l), ..
        } = &module.value
        {
            l.clone()
        } else {
            continue;
        };

        let cached = compile_module(module, "integration_test").await.unwrap();
        let output = execute_task(&cached, &lang, json!({})).await.unwrap();

        match lang.as_str() {
            "rust" => assert_eq!(output["from_rust"], true),
            "node" => assert_eq!(output["from_node"], true),
            "python" => assert_eq!(output["from_python"], true),
            _ => panic!("unexpected language"),
        }

        std::fs::remove_dir_all(cached.cache_path).ok();
    }
}
