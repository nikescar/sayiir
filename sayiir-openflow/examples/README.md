# CLI Examples

Demonstrates `cargo-sayiir-openflow` CLI and cargo plugin usage.

## Export

Export existing Sayiir projects to OpenFlow JSON and Mermaid:

```bash
# Export video-pipeline-rs example
cd ../../examples/video-pipeline-rs
cargo sayiir-openflow export

# Output: openflow.json and openflow.md in current directory
```

## Import

Generate runnable project from OpenFlow JSON:

```bash
# Import workflow
cargo sayiir-openflow import ../test_cli.json --output-dir ./imported-workflow

# Run imported project
cd imported-workflow && cargo run
```

## Run

Execute workflow directly without generating project:

```bash
# Run workflow with input
cargo sayiir-openflow run ../test_cli.json --input '{"url": "https://example.com"}'

# Run without input
cargo sayiir-openflow run ../test_cli.json
```

## Test Workflow

`../test_cli.json` contains a test workflow for import/run demonstrations.
