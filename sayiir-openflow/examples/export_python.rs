//! Export Python workflow to OpenFlow JSON
//!
//! Run: cargo run --example export_python

use sayiir_openflow::*;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("Exporting Python workflow to OpenFlow JSON...\n");

    // For this example, we'll create a minimal Python workflow spec manually
    // In practice, you would scan actual Python files

    let tasks = vec![
        TaskMetadata {
            id: "fetch_data".to_string(),
            language: Language::Python,
            source_code: r#"
def fetch_data(url):
    """Fetch data from URL"""
    import requests
    response = requests.get(url)
    return response.json()
"#
            .to_string(),
            entry_point: "fetch_data".to_string(),
            dependencies: serde_json::Map::from_iter(vec![(
                "requests".to_string(),
                serde_json::json!("2.31.0"),
            )]),
        },
        TaskMetadata {
            id: "process_data".to_string(),
            language: Language::Python,
            source_code: r#"
def process_data(data):
    """Process data"""
    return {
        "count": len(data),
        "items": data[:5]
    }
"#
            .to_string(),
            entry_point: "process_data".to_string(),
            dependencies: serde_json::Map::new(),
        },
    ];

    let spec = build_openflow_spec("Python Data Pipeline".to_string(), tasks);

    // Export to JSON
    let json = serde_json::to_string_pretty(&spec)?;
    std::fs::write("workflow_python.json", &json)?;

    println!("✓ Exported to workflow_python.json ({} bytes)", json.len());
    println!("\nWorkflow: {}", spec.summary);
    println!("Tasks: {}", spec.value.modules.len());

    Ok(())
}
