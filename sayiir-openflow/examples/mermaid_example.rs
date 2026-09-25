use sayiir_openflow::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Mermaid Embedded Code Example ===\n");

    // Create workflow with embedded code
    let mut rust_deps = serde_json::Map::new();
    rust_deps.insert("chrono".to_string(), json!("0.4"));

    let spec = OpenFlowSpec {
        summary: "Example Mermaid workflow".to_string(),
        value: OpenFlowValue {
            modules: vec![
                OpenFlowModule {
                    id: "timestamp_task".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "timestamp_task".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(
                            r#"use serde_json::{json, Value};
use chrono::Utc;

fn run(_input: Value) -> Result<Value, String> {
    let now = Utc::now();
    Ok(json!({"timestamp": now.to_rfc3339()}))
}"#
                            .to_string(),
                        ),
                        dependencies: Some(rust_deps),
                    },
                },
                OpenFlowModule {
                    id: "format_task".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "format_task".to_string(),
                        language: Some("python".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(
                            r#"def run(input_data):
    return {
        "message": f"Timestamp: {input_data['timestamp']}",
        "formatted": True
    }"#
                            .to_string(),
                        ),
                        dependencies: None,
                    },
                },
            ],
        },
    };

    // Export to Mermaid markdown
    println!("Exporting to Mermaid...\n");
    let mermaid = export_mermaid(&spec)?;
    println!("{}\n", mermaid);

    // Save to file
    std::fs::write("workflow.mmd", &mermaid)?;
    println!("Saved to workflow.mmd\n");

    // Import back from Mermaid
    println!("Importing from Mermaid...\n");
    let imported = import_mermaid(&mermaid)?;
    println!("Imported {} modules\n", imported.value.modules.len());

    // Verify round-trip
    let re_exported = export_mermaid(&imported)?;
    if mermaid == re_exported {
        println!("✓ Round-trip successful: export → import → export identical\n");
    } else {
        println!("✗ Round-trip failed: outputs differ\n");
    }

    // Compile and execute (if runtimes available)
    println!("=== Compiling and Executing ===\n");

    for module in &imported.value.modules {
        if let OpenFlowModuleValue::Script {
            language: Some(lang),
            ..
        } = &module.value
        {
            println!("Compiling {} ({})...", module.id, lang);

            match compile_module(module, "mermaid_example").await {
                Ok(cached) => {
                    println!("  Compiled to: {:?}", cached.executable);

                    let input = json!({});
                    match execute_task(&cached, lang, input).await {
                        Ok(output) => {
                            println!("  Output: {}", serde_json::to_string_pretty(&output)?);
                        }
                        Err(e) => {
                            println!("  Execution error: {}", e);
                        }
                    }

                    // Cleanup
                    std::fs::remove_dir_all(cached.cache_path).ok();
                }
                Err(e) => {
                    println!("  Compilation error: {}", e);
                }
            }
        }
    }

    Ok(())
}
