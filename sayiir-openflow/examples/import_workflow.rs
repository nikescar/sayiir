//! Import OpenFlow JSON and generate standalone project
//!
//! Run: cargo run --example import_workflow

use sayiir_openflow::*;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Importing OpenFlow workflow...\n");

    // Read workflow JSON (use the JavaScript workflow from export example)
    let json = std::fs::read_to_string("workflow_javascript.json")
        .expect("Run export_javascript example first to create workflow_javascript.json");

    // Parse OpenFlow JSON
    let spec = import_openflow_json(&json)?;
    println!("✓ Loaded workflow: {}", spec.summary);
    println!("  Modules: {}", spec.value.modules.len());

    // Import workflow (generate standalone project)
    let output_dir = PathBuf::from("./imported_workflow");
    import_workflow(&spec, &output_dir).await?;

    println!("\n✓ Generated standalone project in: {}", output_dir.display());
    println!("\nNext steps:");
    println!("  cd imported_workflow");
    println!("  npm install  # (if Node.js workflow)");
    println!("  npm start    # Run the workflow");

    Ok(())
}
