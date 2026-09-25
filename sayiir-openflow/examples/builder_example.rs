use sayiir_openflow::builder::WorkflowBuilder;
use sayiir_openflow::{export_mermaid, export_openflow_json};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Using the builder API - much simpler!
    let spec = WorkflowBuilder::new("Video processing pipeline")
        .rust_task(
            "download_video",
            "run",
            r#"use serde_json::{json, Value};

fn run(input: Value) -> Result<Value, String> {
    let upload_id = input["upload_id"].as_str().ok_or("missing upload_id")?;
    Ok(json!({
        "upload_id": upload_id,
        "local_path": format!("/tmp/{}/source.mp4", upload_id),
        "size_bytes": 52428800,
        "width": 1920,
        "height": 1080
    }))
}"#,
        )
        .rust_task(
            "validate_upload",
            "run",
            r#"use serde_json::Value;

fn run(input: Value) -> Result<Value, String> {
    let size = input["size_bytes"].as_u64().ok_or("missing size")?;
    if size > 10_737_418_240 {
        return Err("file too large".to_string());
    }
    Ok(input)
}"#,
        )
        .python_task(
            "moderate_content",
            "run",
            r#"import random

def run(input_data):
    is_approved = random.random() < 0.9
    return {
        "verdict": "approved" if is_approved else "rejected",
        "confidence": round(0.85 + random.random() * 0.14, 2)
    }"#,
        )
        .node_task(
            "upload_to_cdn",
            "run",
            r#"function run(input) {
    const uploadId = input.upload_id;
    return {
        upload_id: uploadId,
        cdn_url: `https://cdn.example.com/${uploadId}/video.mp4`
    };
}"#,
        )
        .build();

    // Export to JSON
    let json = export_openflow_json(&spec)?;
    std::fs::write("builder_workflow.json", &json)?;
    println!("✓ Exported to builder_workflow.json");

    // Export to Mermaid
    let mermaid = export_mermaid(&spec)?;
    std::fs::write("builder_workflow.mmd", &mermaid)?;
    println!("✓ Exported to builder_workflow.mmd");

    println!("✓ Exported {} tasks using builder API", spec.value.modules.len());

    Ok(())
}
