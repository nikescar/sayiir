use sayiir_openflow::*;
use serde_json::json;

#[test]
fn test_preview_import_json() {
    let mut deps = serde_json::Map::new();
    deps.insert("chrono".to_string(), json!("0.4"));

    let spec = OpenFlowSpec {
        summary: "Test Workflow".to_string(),
        value: OpenFlowValue {
            modules: vec![
                OpenFlowModule {
                    id: "rust_task".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "rust_task".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some("fn run(input: Value) -> Result<Value, String> {\n    Ok(input)\n}".to_string()),
                        dependencies: Some(deps.clone()),
                    },
                },
                OpenFlowModule {
                    id: "simple_task".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "simple_task".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                },
            ],
        },
    };

    let json = serde_json::to_string(&spec).unwrap();
    let preview = preview_import_json(&json).unwrap();

    assert_eq!(preview.summary, "Test Workflow");
    assert_eq!(preview.total_modules, 2);
    assert_eq!(preview.modules.len(), 2);

    // Check rust_task preview
    assert_eq!(preview.modules[0].id, "rust_task");
    assert_eq!(preview.modules[0].language, Some("rust".to_string()));
    assert_eq!(preview.modules[0].entry_point, Some("run".to_string()));
    assert_eq!(preview.modules[0].code_lines, Some(3));
    assert_eq!(preview.modules[0].dependencies_count, 1);

    // Check simple_task preview
    assert_eq!(preview.modules[1].id, "simple_task");
    assert_eq!(preview.modules[1].language, None);
    assert_eq!(preview.modules[1].entry_point, None);
    assert_eq!(preview.modules[1].code_lines, None);
    assert_eq!(preview.modules[1].dependencies_count, 0);
}

#[test]
fn test_preview_import_mermaid() {
    let mermaid = r#"flowchart TD
    task1[Rust Task]
    task2[Python Task]
    task1 --> task2

%%% task1 (rust)
%%% Entry: run
%%% Dependencies: {"serde_json":"1.0"}
```rust
fn run(input: Value) -> Result<Value, String> {
    Ok(input)
}
```

%%% task2 (python)
%%% Entry: main
%%% Dependencies: {}
```python
def main(input):
    return input
```
"#;

    let preview = preview_import_mermaid(mermaid).unwrap();

    assert_eq!(preview.total_modules, 2);

    // Check task1 preview
    assert_eq!(preview.modules[0].id, "task1");
    assert_eq!(preview.modules[0].language, Some("rust".to_string()));
    assert_eq!(preview.modules[0].entry_point, Some("run".to_string()));
    assert_eq!(preview.modules[0].code_lines, Some(3));
    assert_eq!(preview.modules[0].dependencies_count, 1);

    // Check task2 preview
    assert_eq!(preview.modules[1].id, "task2");
    assert_eq!(preview.modules[1].language, Some("python".to_string()));
    assert_eq!(preview.modules[1].entry_point, Some("main".to_string()));
    assert_eq!(preview.modules[1].code_lines, Some(2));
    assert_eq!(preview.modules[1].dependencies_count, 0);
}

#[test]
fn test_preview_display() {
    let mut deps = serde_json::Map::new();
    deps.insert("chrono".to_string(), json!("0.4"));

    let spec = OpenFlowSpec {
        summary: "Test Workflow".to_string(),
        value: OpenFlowValue {
            modules: vec![OpenFlowModule {
                id: "rust_task".to_string(),
                value: OpenFlowModuleValue::Script {
                    path: "rust_task".to_string(),
                    language: Some("rust".to_string()),
                    entry_point: Some("run".to_string()),
                    code: Some("fn run() {\n}\n".to_string()),
                    dependencies: Some(deps),
                },
            }],
        },
    };

    let json = serde_json::to_string(&spec).unwrap();
    let preview = preview_import_json(&json).unwrap();

    let display = format!("{}", preview);
    assert!(display.contains("Workflow: Test Workflow"));
    assert!(display.contains("Modules: 1"));
    assert!(display.contains("rust_task (rust)"));
    assert!(display.contains("2 lines"));
    assert!(display.contains("1 deps"));
}
