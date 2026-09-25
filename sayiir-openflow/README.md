# sayiir-openflow

Export Rust/Python/Node.js workflows to portable OpenFlow JSON and Mermaid markdown.

## Build

```bash
cargo build --release --bin cargo-sayiir-openflow
cp target/release/cargo-sayiir-openflow ~/.cargo/bin/
```

## CLI Usage

```bash
# Export current project to OpenFlow JSON and Mermaid
cargo-sayiir-openflow sayiir-openflow export

# Export to JSON only
cargo-sayiir-openflow sayiir-openflow export --format json

# Export to Mermaid only
cargo-sayiir-openflow sayiir-openflow export --format mermaid

# Custom output paths
cargo-sayiir-openflow sayiir-openflow export \
  --output workflow.json \
  --mermaid-output workflow.md

# Dry run (preview without writing files)
cargo-sayiir-openflow sayiir-openflow export --dry-run
```

## Cargo Plugin Usage

After installing to `~/.cargo/bin/`, use as a cargo subcommand:

```bash
# Navigate to your workflow project
cd examples/video-pipeline-rs

# Export with default settings
cargo sayiir-openflow export

# Export with options
cargo sayiir-openflow export --format both --dry-run
```

## Supported Languages

- **Rust**: Scans `src/**/*.rs`, extracts `#[task]` functions
- **Python**: Scans `**/*.py` (excludes venv), extracts `@task` decorators
- **Node.js**: Scans `src/**/*.{ts,js}`, extracts `task(...)` calls

Auto-detects language from project structure (Cargo.toml, *.py, package.json).

## Output Format

**OpenFlow JSON** (Windmill spec):
- Workflow definition with embedded source code
- Task metadata (language, entry points, dependencies)
- Portable across systems with appropriate runtimes

**Mermaid Markdown**:
- Flowchart diagram of workflow structure
- Human-readable documentation format

## License

MIT
