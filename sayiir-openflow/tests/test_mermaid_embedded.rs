use sayiir_openflow::*;
use serde_json::json;

#[test]
fn test_export_mermaid_with_embedded_code() {
    let mut deps = serde_json::Map::new();
    deps.insert("chrono".to_string(), json!("0.4"));

    let spec = OpenFlowSpec {
        summary: "Test workflow".to_string(),
        value: OpenFlowValue {
            modules: vec![OpenFlowModule {
                id: "rust_task".to_string(),
                value: OpenFlowModuleValue::Script {
                    path: "rust_task".to_string(),
                    language: Some("rust".to_string()),
                    entry_point: Some("run".to_string()),
                    code: Some("fn run(input: Value) -> Result<Value, String> { Ok(input) }".to_string()),
                    dependencies: Some(deps),
                },
            }],
        },
    };

    let mermaid = export_mermaid(&spec).unwrap();

    // Verify flowchart structure
    assert!(mermaid.contains("flowchart TD"));
    assert!(mermaid.contains("rust_task[rust_task]"));

    // Verify metadata comments
    assert!(mermaid.contains("%%% rust_task (rust)"));
    assert!(mermaid.contains("%%% Entry: run"));
    assert!(mermaid.contains("%%% Dependencies: {\"chrono\":\"0.4\"}"));

    // Verify code block
    assert!(mermaid.contains("```rust"));
    assert!(mermaid.contains("fn run(input: Value)"));
    assert!(mermaid.contains("```"));
}

#[test]
fn test_export_mermaid_without_code_backward_compat() {
    let spec = OpenFlowSpec {
        summary: "Simple workflow".to_string(),
        value: OpenFlowValue {
            modules: vec![OpenFlowModule {
                id: "task1".to_string(),
                value: OpenFlowModuleValue::Script {
                    path: "task1".to_string(),
                    language: None,
                    entry_point: None,
                    code: None,
                    dependencies: None,
                },
            }],
        },
    };

    let mermaid = export_mermaid(&spec).unwrap();

    // Should only contain flowchart, no code blocks
    assert!(mermaid.contains("flowchart TD"));
    assert!(mermaid.contains("task1[task1]"));
    assert!(!mermaid.contains("```"));
}
