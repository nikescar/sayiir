use sayiir_openflow::*;
use serde_json::json;

#[test]
fn test_parse_module_with_dependencies() {
    let json = r#"{
        "summary": "Test",
        "value": {
            "modules": [{
                "id": "task1",
                "value": {
                    "type": "script",
                    "path": "task1",
                    "language": "rust",
                    "entry_point": "run",
                    "code": "fn run(input: Value) -> Result<Value, String> { Ok(input) }",
                    "dependencies": {
                        "serde_json": "1.0",
                        "reqwest": "0.11"
                    }
                }
            }]
        }
    }"#;

    let spec = import_openflow_json(json).unwrap();

    if let OpenFlowModuleValue::Script { dependencies, .. } = &spec.value.modules[0].value {
        let deps = dependencies.as_ref().unwrap();
        assert_eq!(deps.get("serde_json").unwrap(), "1.0");
        assert_eq!(deps.get("reqwest").unwrap(), "0.11");
    } else {
        panic!("expected Script variant");
    }
}

#[test]
fn test_export_module_with_dependencies() {
    let spec = OpenFlowSpec {
        summary: "Test".to_string(),
        value: OpenFlowValue {
            modules: vec![OpenFlowModule {
                id: "task1".to_string(),
                value: OpenFlowModuleValue::Script {
                    path: "task1".to_string(),
                    language: Some("rust".to_string()),
                    entry_point: Some("run".to_string()),
                    code: Some("fn run() {}".to_string()),
                    dependencies: Some({
                        let mut map = serde_json::Map::new();
                        map.insert("serde_json".to_string(), json!("1.0"));
                        map
                    }),
                },
            }],
        },
    };

    let json_str = export_openflow_json(&spec).unwrap();
    assert!(json_str.contains("\"dependencies\""));
    assert!(json_str.contains("\"serde_json\""));
}

#[test]
fn test_dependencies_field_optional() {
    let json = r#"{
        "summary": "Test",
        "value": {
            "modules": [{
                "id": "task1",
                "value": {
                    "type": "script",
                    "path": "task1",
                    "language": "rust",
                    "entry_point": "run",
                    "code": "fn run() {}"
                }
            }]
        }
    }"#;

    let spec = import_openflow_json(json).unwrap();

    if let OpenFlowModuleValue::Script { dependencies, .. } = &spec.value.modules[0].value {
        assert!(dependencies.is_none());
    }
}
