//! Export Rust workflow to OpenFlow JSON
//!
//! Run: cargo run --example export_rust

use sayiir_openflow::*;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("Exporting Rust workflow to OpenFlow JSON...\n");

    // Detect language
    let project_dir = PathBuf::from("../examples/video-pipeline-rs");
    let language = detect_language(&project_dir)?;
    println!("✓ Detected {:?} project", language);

    // Scan project files
    let project_scan = scan::scan_project(&project_dir, language)?;
    println!("✓ Scanned {} files", project_scan.task_files.len());

    // Build task registry
    let mut task_registry = extract::TaskRegistry::new();
    for file_path in &project_scan.task_files {
        let source = std::fs::read_to_string(file_path)?;
        let tasks = extract::rust::extract_all_rust_tasks(&source)?;
        for task in tasks {
            task_registry.insert(task);
        }
    }
    println!("✓ Found {} tasks", task_registry.len());

    // Parse workflow
    let workflow_source = std::fs::read_to_string(&project_scan.workflow_file)?;
    let wf = parse::rust::parse_rust_workflow(&workflow_source)?;
    println!("✓ Workflow: {}", wf.name);

    // Parse dependencies
    let deps = parse_cargo_deps(&project_dir).unwrap_or_default();

    // Build OpenFlow spec
    let mut matched_tasks = Vec::new();
    for task_name in &wf.task_names {
        if let Some(task_src) = task_registry.get_by_name(task_name) {
            let clean_code = extract_clean::extract_rust_function(
                &task_src.source_code,
                &task_src.entry_point,
                Some(&project_dir),
            )
            .unwrap_or_else(|| task_src.source_code.clone());

            matched_tasks.push(TaskMetadata {
                id: task_src.id.clone(),
                language: Language::Rust,
                source_code: clean_code,
                entry_point: task_src.entry_point.clone(),
                dependencies: deps.clone(),
            });
        }
    }

    let spec = build_openflow_spec(wf.name, matched_tasks);

    // Export to JSON
    let json = serde_json::to_string_pretty(&spec)?;
    std::fs::write("workflow_rust.json", &json)?;

    println!("\n✓ Exported to workflow_rust.json ({} bytes)", json.len());
    Ok(())
}
