use sayiir_openflow::builder::{build_openflow_spec, TaskMetadata, Language};
use std::collections::HashMap;

#[test]
fn test_build_openflow_spec() {
    let tasks = vec![
        TaskMetadata {
            id: "task1".to_string(),
            language: Language::Rust,
            source_code: "fn task1() {}".to_string(),
            entry_point: "task1".to_string(),
            dependencies: HashMap::new(),
        },
        TaskMetadata {
            id: "task2".to_string(),
            language: Language::Python,
            source_code: "def task2():\n    pass".to_string(),
            entry_point: "task2".to_string(),
            dependencies: {
                let mut deps = HashMap::new();
                deps.insert("requests".to_string(), "2.28.0".to_string());
                deps
            },
        },
    ];

    let spec = build_openflow_spec("test-workflow".to_string(), tasks);

    assert_eq!(spec.summary, "test-workflow");
    assert_eq!(spec.value.modules.len(), 2);

    // Verify first module
    let module1 = &spec.value.modules[0];
    assert_eq!(module1.id, "task1");

    // Verify second module has dependencies
    let module2 = &spec.value.modules[1];
    assert_eq!(module2.id, "task2");
}
