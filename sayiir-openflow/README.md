# sayiir-openflow

Import, export, and run Rust/Python/Node.js workflows using portable OpenFlow JSON and Mermaid markdown.

## Installation

**Option 1: Install from source (recommended)**
```bash
cd sayiir-openflow
cargo install --path . --bin cargo-sayiir-openflow
```

**Option 2: Manual build and copy**
```bash
cargo build --release --bin cargo-sayiir-openflow
cp target/release/cargo-sayiir-openflow ~/.cargo/bin/
```

**Option 3: Install from crates.io** (when published)
```bash
cargo install sayiir-openflow
```

After installation, verify with:
```bash
cargo sayiir-openflow --help
```

## CLI Usage

### Export Command

Export existing projects to portable OpenFlow JSON and Mermaid:

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

### Import Command

Generate standalone executable projects from OpenFlow JSON or Mermaid:

```bash
# Import workflow and generate runnable project
cargo-sayiir-openflow sayiir-openflow import workflow.json \
  --output-dir ./my-workflow

# Import from Mermaid markdown
cargo-sayiir-openflow sayiir-openflow import workflow.md \
  --output-dir ./my-workflow

# Overwrite existing directory
cargo-sayiir-openflow sayiir-openflow import workflow.json \
  --output-dir ./my-workflow \
  --overwrite
```

**Output structure:**

**Mono-language workflow** (all tasks same language):
```
my-workflow/
├── Cargo.toml (or package.json, pyproject.toml)
├── src/
│   ├── main.rs (entry point)
│   └── tasks/ (extracted task code)
└── README.md
```

**Multi-language workflow** (mixed Rust/Python/Node.js):
```
my-workflow/
├── rust_tasks/
│   ├── Cargo.toml
│   └── src/
├── python_tasks/
│   ├── pyproject.toml
│   └── tasks/
├── node_tasks/
│   ├── package.json
│   └── tasks/
├── workflow.json
└── README.md
```

### Run Command

Execute workflows directly without generating a project:

```bash
# Run workflow with JSON input
cargo-sayiir-openflow sayiir-openflow run workflow.json \
  --input '{"url": "https://example.com"}'

# Run with custom timeout (default: 30s)
cargo-sayiir-openflow sayiir-openflow run workflow.json \
  --input '{"data": "test"}' \
  --timeout 60

# Run without input (empty JSON object)
cargo-sayiir-openflow sayiir-openflow run workflow.json
```

**Features:**
- Automatically compiles embedded code on first run
- Caches compiled modules in `~/.sayiir/cache/`
- Chains task outputs → inputs sequentially
- Returns final result as JSON

## Cargo Plugin Usage

After installing to `~/.cargo/bin/`, use as a cargo subcommand:

```bash
# Export workflow
cd examples/video-pipeline-rs
cargo sayiir-openflow export

# Import and run workflow
cargo sayiir-openflow import workflow.json --output-dir ./imported
cd imported && cargo run

# Run workflow directly
cargo sayiir-openflow run workflow.json --input '{"url": "https://..."}'
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
