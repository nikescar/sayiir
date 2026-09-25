use sayiir_openflow::*;

fn main() -> Result<()> {
    println!("=== Creating workflow with embedded code ===\n");

    // Create workflow with embedded Rust code
    let spec = OpenFlowSpec {
        summary: "Embedded code export example".to_string(),
        value: OpenFlowValue {
            modules: vec![
                OpenFlowModule {
                    id: "calculate_tax".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "calculate_tax".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(
                            r#"use serde_json::{json, Value};

fn run(input: Value) -> Result<Value, String> {
    let price = input["price"].as_f64().ok_or("missing price")?;
    let rate = input["tax_rate"].as_f64().unwrap_or(0.08);
    Ok(json!({"total": price * (1.0 + rate)}))
}"#
                            .to_string(),
                        ),
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "format_output".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "format_output".to_string(),
                        language: Some("python".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(
                            r#"def run(input_data):
    total = input_data['total']
    return {"formatted": f"Total: ${total:.2f}"}"#
                                .to_string(),
                        ),
                        dependencies: None,
                    },
                },
            ],
        },
    };

    // Export to JSON
    println!("Exporting to JSON...");
    let json_output = export_openflow_json(&spec)?;
    std::fs::write("embedded_workflow.json", &json_output)?;
    println!("✓ Saved to embedded_workflow.json\n");
    println!("JSON preview:");
    println!("{}\n", json_output);

    // Export to Mermaid
    println!("Exporting to Mermaid...");
    let mermaid_output = export_mermaid(&spec)?;
    std::fs::write("embedded_workflow.mmd", &mermaid_output)?;
    println!("✓ Saved to embedded_workflow.mmd\n");
    println!("Mermaid preview:");
    println!("{}\n", mermaid_output);

    println!("=== Files created ===");
    println!("  - embedded_workflow.json (OpenFlow JSON with embedded code)");
    println!("  - embedded_workflow.mmd (Mermaid markdown with embedded code)");

    Ok(())
}
