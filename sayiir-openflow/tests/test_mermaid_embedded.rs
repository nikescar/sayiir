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

#[test]
fn test_import_mermaid_with_embedded_code() {
    let mermaid = r#"flowchart TD
    rust_task[rust_task]
    node_task[node_task]
    rust_task --> node_task

%%% rust_task (rust)
%%% Entry: run
%%% Dependencies: {"chrono":"0.4"}
```rust
fn run(input: Value) -> Result<Value, String> {
    Ok(input)
}
```

%%% node_task (node)
%%% Entry: main
%%% Dependencies: {"axios":"^1.6.0"}
```javascript
async function main(input) {
    return input;
}
```
"#;

    let spec = import_mermaid(mermaid).unwrap();

    assert_eq!(spec.value.modules.len(), 2);

    // Verify first task (rust)
    let rust_module = &spec.value.modules[0];
    assert_eq!(rust_module.id, "rust_task");
    if let OpenFlowModuleValue::Script {
        language,
        entry_point,
        code,
        dependencies,
        ..
    } = &rust_module.value
    {
        assert_eq!(language.as_ref().unwrap(), "rust");
        assert_eq!(entry_point.as_ref().unwrap(), "run");
        assert!(code.as_ref().unwrap().contains("fn run"));

        let deps = dependencies.as_ref().unwrap();
        assert_eq!(deps.get("chrono").unwrap().as_str().unwrap(), "0.4");
    } else {
        panic!("Expected Script variant");
    }

    // Verify second task (node)
    let node_module = &spec.value.modules[1];
    assert_eq!(node_module.id, "node_task");
    if let OpenFlowModuleValue::Script {
        language,
        entry_point,
        code,
        dependencies,
        ..
    } = &node_module.value
    {
        assert_eq!(language.as_ref().unwrap(), "node");
        assert_eq!(entry_point.as_ref().unwrap(), "main");
        assert!(code.as_ref().unwrap().contains("async function main"));

        let deps = dependencies.as_ref().unwrap();
        assert_eq!(deps.get("axios").unwrap().as_str().unwrap(), "^1.6.0");
    } else {
        panic!("Expected Script variant");
    }
}

#[test]
fn test_import_mermaid_backward_compat() {
    let mermaid = r#"flowchart TD
    task1[task1]
    task2[task2]
    task1 --> task2
"#;

    let spec = import_mermaid(mermaid).unwrap();

    assert_eq!(spec.value.modules.len(), 2);

    // Both tasks should have all optional fields as None
    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            language,
            entry_point,
            code,
            dependencies,
            ..
        } = &module.value
        {
            assert!(language.is_none());
            assert!(entry_point.is_none());
            assert!(code.is_none());
            assert!(dependencies.is_none());
        }
    }
}
