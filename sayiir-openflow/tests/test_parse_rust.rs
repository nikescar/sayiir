use sayiir_openflow::parse::rust::{parse_rust_workflow, RustWorkflow};

#[test]
fn test_parse_simple_workflow() {
    let source = r#"
        use sayiir_runtime::prelude::*;

        fn main() {
            let workflow = workflow! {
                name: "test-workflow",
                codec: JsonCodec,
                steps: [task1, task2, task3]
            }.unwrap();
        }
    "#;

    let result = parse_rust_workflow(source).unwrap();
    assert_eq!(result.name, "test-workflow");
    assert_eq!(result.task_names, vec!["task1", "task2", "task3"]);
}

#[test]
fn test_parse_workflow_with_parallel_tasks() {
    let source = r#"
        workflow! {
            name: "parallel-test",
            codec: JsonCodec,
            steps: [
                download,
                (transcode_720p || transcode_1080p),
                upload
            ]
        }
    "#;

    let result = parse_rust_workflow(source).unwrap();
    assert_eq!(result.name, "parallel-test");
    assert_eq!(result.task_names, vec!["download", "transcode_720p", "transcode_1080p", "upload"]);
}

#[test]
fn test_no_workflow_found() {
    let source = r#"
        fn main() {
            println!("Hello");
        }
    "#;

    let result = parse_rust_workflow(source);
    assert!(result.is_err());
}
