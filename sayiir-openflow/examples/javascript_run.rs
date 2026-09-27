//! Run pure JavaScript workflow with rquickjs interpreter
//!
//! Run: cargo run --example javascript_run --features javascript

#[cfg(feature = "javascript")]
fn main() -> sayiir_openflow::Result<()> {
    use sayiir_openflow::*;
    use std::collections::HashMap;

    println!("Running JavaScript workflow with rquickjs interpreter...\n");

    // Create a pure JavaScript workflow (no external dependencies)
    let tasks = vec![
        TaskMetadata {
            id: "double".to_string(),
            language: Language::Node,
            source_code: "function double(x) { return x * 2; }".to_string(),
            entry_point: "double".to_string(),
            dependencies: HashMap::new(),
        },
        TaskMetadata {
            id: "add_ten".to_string(),
            language: Language::Node,
            source_code: "function addTen(x) { return x + 10; }".to_string(),
            entry_point: "addTen".to_string(),
            dependencies: HashMap::new(),
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
            dependencies: HashMap::new(),
        },
    ];

    let spec = build_openflow_spec("rquickjs Example".to_string(), tasks);

    // Validate (must be pure JavaScript with no dependencies)
    println!("→ Validating workflow...");
    javascript::validate_pure_javascript(&spec)?;
    println!("✓ Validation passed\n");

    // Run workflow: 5 → double → 10 → add_ten → 20 → format → { value: 20, ... }
    println!("→ Running workflow with input: 5");
    let input = serde_json::json!(5);
    let result = javascript::run_workflow(&spec, input)?;

    println!("\n✓ Workflow completed:");
    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

#[cfg(not(feature = "javascript"))]
fn main() {
    eprintln!("Error: This example requires the 'javascript' feature");
    eprintln!("Run with: cargo run --example javascript_run --features javascript");
    std::process::exit(1);
}
