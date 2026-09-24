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

## License

MIT
