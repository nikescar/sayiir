use sayiir_openflow::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a workflow with embedded Rust code
    let spec = OpenFlowSpec {
        summary: "Calculate product price with tax".to_string(),
        value: OpenFlowValue {
            modules: vec![OpenFlowModule {
                id: "calculate_price".to_string(),
                value: OpenFlowModuleValue::Script {
                    path: "calculate_price".to_string(),
                    language: Some("rust".to_string()),
                    entry_point: Some("run".to_string()),
                    code: Some(
                        r#"
use serde_json::{json, Value};

fn run(input: Value) -> Result<Value, String> {
    let price = input["price"].as_f64().ok_or("missing price")?;
    let tax_rate = input["tax_rate"].as_f64().ok_or("missing tax_rate")?;

    let total = price * (1.0 + tax_rate);

    Ok(json!({
        "price": price,
        "tax": price * tax_rate,
        "total": total
    }))
}
"#
                        .to_string(),
                    ),
                },
            }],
        },
    };

    println!("=== Exporting workflow to JSON ===");
    let json_str = export_openflow_json(&spec)?;
    println!("{}", json_str);

    println!("\n=== Importing workflow ===");
    let imported = import_openflow_json(&json_str)?;

    println!("=== Checking runtimes ===");
    check_runtimes(&imported)?;
    println!("All required runtimes are installed");

    println!("\n=== Compiling module ===");
    let module = &imported.value.modules[0];
    let cached = compile_module(module, "example_workflow").await?;
    println!("Compiled to: {:?}", cached.executable);

    println!("\n=== Executing task ===");
    let input = json!({"price": 100.0, "tax_rate": 0.08});
    let output = execute_task(&cached, "rust", input).await?;
    println!("Result: {}", serde_json::to_string_pretty(&output)?);

    // Cleanup
    std::fs::remove_dir_all(cached.cache_path).ok();

    Ok(())
}
