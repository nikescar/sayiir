use sayiir_openflow::{scan::scan_project, ProjectLanguage};
use std::path::PathBuf;

#[test]
fn test_scan_rust_project() {
    let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../examples/video-pipeline-rs");

    let scan = scan_project(&project_dir, ProjectLanguage::Rust).unwrap();

    // Should find workflow in main.rs
    assert!(scan.workflow_file.ends_with("main.rs"));

    // Should find multiple .rs files
    assert!(scan.task_files.len() >= 2);
    assert!(scan.task_files.iter().any(|p| p.ends_with("tasks.rs")));
}
