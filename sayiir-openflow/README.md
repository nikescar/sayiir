# sayiir-openflow

OpenFlow JSON and Mermaid markdown import/export for Sayiir workflow engine.

Supports importing and exporting workflows in OpenFlow JSON format (Windmill spec) and Mermaid markdown flowcharts.

## Features

- Import OpenFlow JSON to Sayiir workflows
- Export Sayiir workflows to OpenFlow JSON
- Import Mermaid markdown flowcharts
- Export workflows to Mermaid markdown

## Usage

```rust
use sayiir_openflow::{import_openflow_json, OpenFlowSpec};

let json = r#"{"summary": "Example workflow", "value": {"modules": []}}"#;
// let workflow = import_openflow_json(json)?;
```

## Embedded Code Support

sayiir-openflow supports embedding full source code (Rust, Node.js, Python) directly in OpenFlow JSON and Mermaid exports. This enables portable workflows that run on any system with the appropriate language runtimes.

### Example

```rust
use sayiir_openflow::*;
use serde_json::json;

// Create workflow with embedded code
let spec = OpenFlowSpec {
    summary: "Example workflow".to_string(),
    value: OpenFlowValue {
        modules: vec![OpenFlowModule {
            id: "my_task".to_string(),
            value: OpenFlowModuleValue::Script {
                path: "my_task".to_string(),
                language: Some("rust".to_string()),
                entry_point: Some("run".to_string()),
                code: Some("fn run(input: Value) -> Result<Value, String> { Ok(input) }".to_string()),
                dependencies: None,
            },
        }],
    },
};

// Export to JSON
let json = export_openflow_json(&spec)?;

// Import and validate
let imported = import_openflow_json(&json)?;
check_runtimes(&imported)?;

// Compile and execute
let cached = compile_module(&imported.value.modules[0], "workflow_id").await?;
let output = execute_task(&cached, "rust", json!({"key": "value"})).await?;
```

### Supported Languages

- **Rust** - Compiled to native binary via `cargo build --release`
- **Node.js** - Executed via `node task.js`
- **Python** - Executed via `python3 task.py`

### Requirements

- Rust tasks require `cargo` (install from https://rustup.rs)
- Node.js tasks require `node` (install from https://nodejs.org)
- Python tasks require `python3` (install from https://www.python.org)

### Task Signature

All tasks must follow this signature:

**Rust:**
```rust
fn run(input: serde_json::Value) -> Result<serde_json::Value, String>
```

**Node.js:**
```javascript
async function run(input) { return {...}; }
```

**Python:**
```python
def run(input_data): return {...}
```

### External Dependencies

Tasks can declare external dependencies using the `dependencies` field:

**Rust dependencies** (Cargo.toml format):
```rust
let mut deps = serde_json::Map::new();
deps.insert("chrono".to_string(), json!("0.4"));

OpenFlowModuleValue::Script {
    // ... other fields
    dependencies: Some(deps),
}
```

**Node.js dependencies** (package.json format):
```rust
let mut deps = serde_json::Map::new();
deps.insert("axios".to_string(), json!("^1.6.0"));

OpenFlowModuleValue::Script {
    // ... other fields
    dependencies: Some(deps),
}
```

**Python dependencies** (requirements.txt format):
```rust
let mut deps = serde_json::Map::new();
deps.insert("requests".to_string(), json!("2.31.0"));

OpenFlowModuleValue::Script {
    // ... other fields
    dependencies: Some(deps),
}
```

Dependencies are automatically installed during compilation:
- **Rust**: Generates `Cargo.toml` and runs `cargo build --release`
- **Node.js**: Generates `package.json` and runs `npm install`
- **Python**: Generates `requirements.txt`, creates `venv`, and runs `pip install`

See `examples/dependencies_example.rs` for a complete example.

### Cache Location

Compiled artifacts are cached in `~/.sayiir/cache/<workflow_id>_<module_id>_<language>/`.

See `examples/embedded_code_example.rs` for a complete example.

## License

MIT
