//! Run pure JavaScript workflow with Brimstone interpreter
//!
//! Run: cargo run --example brimstone_run --features brimstone

#[cfg(feature = "brimstone")]
fn main() -> sayiir_openflow::Result<()> {
    use sayiir_openflow::*;

    println!("Running JavaScript workflow with Brimstone interpreter...\n");

    // Create a pure JavaScript workflow (no external dependencies)
    let tasks = vec![
        TaskMetadata {
            id: "double".to_string(),
            language: Language::Node,
            source_code: "function double(x) { return x * 2; }".to_string(),
            entry_point: "double".to_string(),
            dependencies: serde_json::Map::new(),
        },
        TaskMetadata {
            id: "add_ten".to_string(),
            language: Language::Node,
            source_code: "function addTen(x) { return x + 10; }".to_string(),
            entry_point: "addTen".to_string(),
            dependencies: serde_json::Map::new(),
        },
        TaskMetadata {
            id: "format".to_string(),
            language: Language::Node,
            source_code: r#"
function format(x) {
    return {
        value: x,
        message: "Final result: " + x
    };
}
"#
            .to_string(),
            entry_point: "format".to_string(),
            dependencies: serde_json::Map::new(),
        },
    ];

    let spec = build_openflow_spec("Brimstone Example".to_string(), tasks);

    // Validate (must be pure JavaScript with no dependencies)
    println!("→ Validating workflow...");
    brimstone::validate_pure_javascript(&spec)?;
    println!("✓ Validation passed\n");

    // Run workflow: 5 → double → 10 → add_ten → 20 → format → { value: 20, ... }
    println!("→ Running workflow with input: 5");
    let input = serde_json::json!(5);
    let result = brimstone::run_workflow(&spec, input)?;

    println!("\n✓ Workflow completed:");
    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

#[cfg(not(feature = "brimstone"))]
fn main() {
    eprintln!("Error: This example requires the 'brimstone' feature");
    eprintln!("Run with: cargo run --example brimstone_run --features brimstone");
    std::process::exit(1);
}
