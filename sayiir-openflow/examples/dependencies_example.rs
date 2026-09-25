use sayiir_openflow::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Example: Workflow with external dependencies
    // Shows how to define dependencies for Rust, Node.js, and Python tasks

    let mut rust_deps = serde_json::Map::new();
    rust_deps.insert("chrono".to_string(), json!("0.4"));

    let mut node_deps = serde_json::Map::new();
    node_deps.insert("axios".to_string(), json!("^1.6.0"));

    let mut python_deps = serde_json::Map::new();
    python_deps.insert("requests".to_string(), json!("2.31.0"));

    let spec = OpenFlowSpec {
        summary: "Example workflow with dependencies".to_string(),
        value: OpenFlowValue {
            modules: vec![
                // Rust task with chrono dependency
                OpenFlowModule {
                    id: "rust_task".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "rust_task".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(
                            r#"
use serde_json::{json, Value};
use chrono::Utc;

fn run(_input: Value) -> Result<Value, String> {
    let now = Utc::now();
    Ok(json!({
        "timestamp": now.to_rfc3339(),
        "message": "Task executed with chrono"
    }))
}
"#
                            .to_string(),
                        ),
                        dependencies: Some(rust_deps),
                    },
                },
                // Node.js task with axios dependency
                OpenFlowModule {
                    id: "node_task".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "node_task".to_string(),
                        language: Some("node".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(
                            r#"
const axios = require('axios');

async function run(input) {
    // Example: Make HTTP request with axios
    // const response = await axios.get('https://api.example.com/data');

    return {
        message: 'Task executed with axios',
        axiosVersion: axios.VERSION,
        prevTimestamp: input.timestamp
    };
}
"#
                            .to_string(),
                        ),
                        dependencies: Some(node_deps),
                    },
                },
                // Python task with requests dependency
                OpenFlowModule {
                    id: "python_task".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "python_task".to_string(),
                        language: Some("python".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(
                            r#"
import requests

def run(input_data):
    # Example: Make HTTP request with requests
    # response = requests.get('https://api.example.com/data')

    return {
        "message": "Task executed with requests",
        "requests_version": requests.__version__,
        "prev_message": input_data.get("message")
    }
"#
                            .to_string(),
                        ),
                        dependencies: Some(python_deps),
                    },
                },
            ],
        },
    };

    println!("=== Exporting workflow with dependencies ===");
    let json_str = export_openflow_json(&spec)?;
    println!("{}", json_str);

    println!("\n=== Compiling modules ===");
    for module in &spec.value.modules {
        let lang = if let OpenFlowModuleValue::Script { language, .. } = &module.value {
            language.as_ref().unwrap()
        } else {
            continue;
        };

        println!("Compiling {} task...", lang);
        let cached = compile_module(module, "example_workflow").await?;
        println!("  Compiled to: {:?}", cached.executable);

        // Verify dependency files exist
        match lang.as_str() {
            "rust" => {
                let cargo_toml = cached.cache_path.join("Cargo.toml");
                if cargo_toml.exists() {
                    println!("  ✓ Cargo.toml created");
                }
            }
            "node" => {
                let package_json = cached.cache_path.join("package.json");
                let node_modules = cached.cache_path.join("node_modules");
                if package_json.exists() {
                    println!("  ✓ package.json created");
                }
                if node_modules.exists() {
                    println!("  ✓ node_modules installed");
                }
            }
            "python" => {
                let requirements = cached.cache_path.join("requirements.txt");
                let venv = cached.cache_path.join("venv");
                if requirements.exists() {
                    println!("  ✓ requirements.txt created");
                }
                if venv.exists() {
                    println!("  ✓ venv created");
                }
            }
            _ => {}
        }

        println!("\n=== Executing {} task ===", lang);
        let input = if lang == "rust" {
            json!({})
        } else {
            // Chain output from previous task
            json!({"timestamp": "2024-01-01T00:00:00Z", "message": "test"})
        };

        let output = execute_task(&cached, lang, input).await?;
        println!("Result: {}", serde_json::to_string_pretty(&output)?);

        // Cleanup
        std::fs::remove_dir_all(cached.cache_path).ok();
    }

    Ok(())
}
