//! Export JavaScript workflow to OpenFlow JSON
//!
//! Run: cargo run --example export_javascript

use sayiir_openflow::*;

fn main() -> Result<()> {
    println!("Exporting JavaScript workflow to OpenFlow JSON...\n");

    // Create pure JavaScript workflow (no external dependencies)
    // This workflow is compatible with Brimstone execution
    let tasks = vec![
        TaskMetadata {
            id: "calculate".to_string(),
            language: Language::Node,
            source_code: r#"
function calculate(input) {
    // Pure JavaScript calculation
    const result = input.value * 2 + 10;
    return { result: result };
}
"#
            .to_string(),
            entry_point: "calculate".to_string(),
            dependencies: serde_json::Map::new(), // No dependencies
        },
        TaskMetadata {
            id: "format".to_string(),
            language: Language::Node,
            source_code: r#"
function format(data) {
    // Format the result
    return {
        formatted: `Result: ${data.result}`,
        timestamp: new Date().toISOString()
    };
}
"#
            .to_string(),
            entry_point: "format".to_string(),
            dependencies: serde_json::Map::new(), // No dependencies
        },
    ];

    let spec = build_openflow_spec("JavaScript Calculator".to_string(), tasks);

    // Export to JSON
    let json = serde_json::to_string_pretty(&spec)?;
    std::fs::write("workflow_javascript.json", &json)?;

    println!("✓ Exported to workflow_javascript.json ({} bytes)", json.len());
    println!("\nWorkflow: {}", spec.summary);
    println!("Tasks: {}", spec.value.modules.len());
    println!("\n✓ This workflow is compatible with Brimstone (pure JavaScript, no dependencies)");

    Ok(())
}
