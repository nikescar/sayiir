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
                    code: Some(
                        "fn run(input: Value) -> Result<Value, String> { Ok(input) }".to_string(),
                    ),
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

#[test]
fn test_mermaid_round_trip() {
    let mut rust_deps = serde_json::Map::new();
    rust_deps.insert("chrono".to_string(), json!("0.4"));

    let mut python_deps = serde_json::Map::new();
    python_deps.insert("requests".to_string(), json!("2.31.0"));

    let original_spec = OpenFlowSpec {
        summary: "Round-trip test".to_string(),
        value: OpenFlowValue {
            modules: vec![
                OpenFlowModule {
                    id: "rust_task".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "rust_task".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(
                            "fn run(input: Value) -> Result<Value, String> {\n    Ok(input)\n}"
                                .to_string(),
                        ),
                        dependencies: Some(rust_deps),
                    },
                },
                OpenFlowModule {
                    id: "python_task".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "python_task".to_string(),
                        language: Some("python".to_string()),
                        entry_point: Some("main".to_string()),
                        code: Some("def main(input):\n    return input".to_string()),
                        dependencies: Some(python_deps),
                    },
                },
            ],
        },
    };

    // Export to Mermaid
    let mermaid = export_mermaid(&original_spec).unwrap();

    // Import back
    let imported_spec = import_mermaid(&mermaid).unwrap();

    // Verify modules match
    assert_eq!(
        imported_spec.value.modules.len(),
        original_spec.value.modules.len()
    );

    for (original, imported) in original_spec
        .value
        .modules
        .iter()
        .zip(&imported_spec.value.modules)
    {
        if let (
            OpenFlowModuleValue::Script {
                language: orig_lang,
                entry_point: orig_entry,
                code: orig_code,
                dependencies: orig_deps,
                ..
            },
            OpenFlowModuleValue::Script {
                language: imp_lang,
                entry_point: imp_entry,
                code: imp_code,
                dependencies: imp_deps,
                ..
            },
        ) = (&original.value, &imported.value)
        {
            assert_eq!(orig_lang, imp_lang);
            assert_eq!(orig_entry, imp_entry);
            assert_eq!(orig_code, imp_code);

            // Dependencies should match
            match (orig_deps, imp_deps) {
                (Some(o), Some(i)) => {
                    assert_eq!(o.len(), i.len());
                    for (key, val) in o {
                        assert_eq!(i.get(key), Some(val));
                    }
                }
                (None, None) => {}
                _ => panic!("Dependency mismatch"),
            }
        }
    }

    // Re-export and verify identical
    let re_exported = export_mermaid(&imported_spec).unwrap();

    // Should produce same Mermaid markdown
    assert_eq!(mermaid, re_exported);
}

#[test]
fn test_mermaid_multi_language_workflow() {
    let mut node_deps = serde_json::Map::new();
    node_deps.insert("axios".to_string(), json!("^1.6.0"));

    let spec = OpenFlowSpec {
        summary: "Multi-language workflow".to_string(),
        value: OpenFlowValue {
            modules: vec![
                OpenFlowModule {
                    id: "fetch_data".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "fetch_data".to_string(),
                        language: Some("node".to_string()),
                        entry_point: Some("fetch".to_string()),
                        code: Some(
                            "async function fetch(input) { return {data: [1,2,3]}; }".to_string(),
                        ),
                        dependencies: Some(node_deps),
                    },
                },
                OpenFlowModule {
                    id: "process_data".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "process_data".to_string(),
                        language: Some("python".to_string()),
                        entry_point: Some("process".to_string()),
                        code: Some(
                            "def process(input):\n    return {\"processed\": True}".to_string(),
                        ),
                        dependencies: None,
                    },
                },
            ],
        },
    };

    let mermaid = export_mermaid(&spec).unwrap();

    // Verify structure
    assert!(mermaid.contains("fetch_data[fetch_data]"));
    assert!(mermaid.contains("process_data[process_data]"));
    assert!(mermaid.contains("fetch_data --> process_data"));
    assert!(mermaid.contains("```node"));
    assert!(mermaid.contains("```python"));

    let imported = import_mermaid(&mermaid).unwrap();
    assert_eq!(imported.value.modules.len(), 2);
}
