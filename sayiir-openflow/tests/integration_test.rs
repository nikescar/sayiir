use sayiir_openflow::*;
use std::path::PathBuf;

#[test]
fn test_video_pipeline_rust_extraction() {
    let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../examples/video-pipeline-rs");

    // Scan
    let scan = scan::scan_project(&project_dir, ProjectLanguage::Rust).unwrap();
    assert!(scan.workflow_file.ends_with("main.rs"));
    assert!(scan.task_files.len() >= 3);

    // Extract tasks
    let mut registry = extract::TaskRegistry::new();
    for file in &scan.task_files {
        let source = std::fs::read_to_string(file).unwrap();
        let tasks = extract::rust::extract_all_rust_tasks(&source).unwrap();
        for task in tasks {
            registry.insert(task);
        }
    }

    assert!(registry.len() >= 10);
    assert!(registry.get("download_video").is_some());
    assert!(registry.get("transcode_720p").is_some());

    // Parse workflow
    let wf_source = std::fs::read_to_string(&scan.workflow_file).unwrap();
    let wf = parse::rust::parse_rust_workflow(&wf_source).unwrap();

    assert_eq!(wf.name, "video-pipeline");
    assert!(wf.task_names.contains(&"download_video".to_string()));
}

#[test]
fn test_approval_workflow_python_extraction() {
    let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../examples/approval-workflow-py");

    if !project_dir.exists() {
        println!("Skipping: approval-workflow-py not found");
        return;
    }

    // Scan
    let scan = scan::scan_project(&project_dir, ProjectLanguage::Python).unwrap();
    assert!(scan.workflow_file.ends_with("main.py"));

    // Extract tasks
    let mut registry = extract::TaskRegistry::new();
    for file in &scan.task_files {
        let source = std::fs::read_to_string(file).unwrap();
        let tasks = extract::python::extract_all_python_tasks(&source).unwrap();
        for task in tasks {
            registry.insert(task);
        }
    }

    assert!(registry.len() >= 2);
    assert!(registry.get("submit_expense").is_some());

    // Parse workflow
    let wf_source = std::fs::read_to_string(&scan.workflow_file).unwrap();
    let wf = parse::python::parse_python_workflow(&wf_source).unwrap();

    assert_eq!(wf.name, "expense-approval");
}

#[test]
fn test_rag_agent_node_extraction() {
    let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../examples/rag-agent-cf");

    if !project_dir.exists() {
        println!("Skipping: rag-agent-cf not found");
        return;
    }

    // Scan
    let scan = scan::scan_project(&project_dir, ProjectLanguage::Node).unwrap();
    assert!(scan.task_files.len() >= 3);

    // Extract tasks
    let mut registry = extract::TaskRegistry::new();
    for file in &scan.task_files {
        let source = std::fs::read_to_string(file).unwrap();
        let tasks = extract::node::extract_all_node_tasks(&source).unwrap();
        for task in tasks {
            registry.insert(task);
        }
    }

    assert!(registry.len() >= 2);
}
