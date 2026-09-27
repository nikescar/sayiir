# sayiir-openflow

Import, export, and run Rust/Python/Node.js workflows using portable OpenFlow JSON and Mermaid markdown.

## Installation

**Option 1: Install from source (recommended)**
```bash
cd sayiir-openflow
cargo install --path . --bin cargo-sayiir-openflow
```

**Option 2: With Brimstone JavaScript execution (optional)**
```bash
cd sayiir-openflow
cargo install --path . --bin cargo-sayiir-openflow --features brimstone
```

**Option 3: Manual build and copy**
```bash
cargo build --release --bin cargo-sayiir-openflow
cp target/release/cargo-sayiir-openflow ~/.cargo/bin/
```

**Option 4: Install from crates.io** (when published)
```bash
cargo install sayiir-openflow
# Or with Brimstone
cargo install sayiir-openflow --features brimstone
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

### Run Command (Brimstone Feature Only)

**⚠️ Requires `--features brimstone` during installation**

Execute pure JavaScript workflows directly using the Brimstone interpreter:

```bash
# Run pure JavaScript workflow with JSON input
cargo-sayiir-openflow sayiir-openflow run workflow.json \
  --input '{"value": 42}'

# Run without input (empty JSON object)
cargo-sayiir-openflow sayiir-openflow run workflow.json
```

**Requirements:**
- ✅ All tasks must be JavaScript (`language: "node"`)
- ✅ Zero external dependencies (no `npm` packages)
- ❌ Rust/Python tasks not supported
- ❌ JavaScript with dependencies not supported

**Features:**
- Lightweight Brimstone interpreter (no Node.js required)
- ~97% ECMAScript conformance (test262)
- Chains task outputs → inputs sequentially
- Returns final result as JSON

**Why Brimstone?**
- Fast execution for dependency-free JavaScript
- Compact binary footprint
- Standard-conforming ECMAScript implementation
- No external runtime dependencies

## Cargo Plugin Usage

After installing to `~/.cargo/bin/`, use as a cargo subcommand:

```bash
# Export workflow
cd examples/video-pipeline-rs
cargo sayiir-openflow export

# Import and run workflow
cargo sayiir-openflow import workflow.json --output-dir ./imported
cd imported && cargo run

# Run pure JavaScript workflow (requires brimstone feature)
cargo sayiir-openflow run workflow.json --input '{"value": 42}'
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

## Tree-sitter Extraction Limitations

The export command uses Tree-sitter to extract clean, portable code from tasks. Below are language-specific limitations:

### Rust

**Supported:**
- Extracts functions with or without `#[task]` attributes
- Includes `use` declarations, type definitions, constants, helper functions, and `impl` blocks
- Cross-module type resolution for `use crate::module::*` glob imports

**Excluded:**
- Sayiir runtime imports (`sayiir_runtime`, `sayiir_core`, `sayiir::`, `sayiir_persistence`)
- Test functions (`#[test]`, `#[cfg(test)]`)
- Sayiir-specific derive macros and impls

**Cross-module types:**
- Only resolves types from `crate::*` glob imports
- Requires `project_dir` to access source files
- Does not extract types from external crate dependencies

### Python

**Supported:**
- Extracts functions with or without `@task` decorators (decorator is stripped)
- Includes imports, class definitions, and simple constants

**Excluded:**
- Sayiir imports (`from sayiir`, `import sayiir`)
- Workflow definitions (`Flow(...)`, `flow(...)`)
- Function call assignments (e.g., `result = calculate()`)
- Runtime execution code (`run_workflow`, `runDurableWorkflow`)

**Constants:**
- Only includes simple assignments (primitives, `Path("...")`)
- Skips object instantiation, function calls, and complex expressions

### Node.js / TypeScript

**Supported:**
- Extracts task functions (converts arrow functions to regular functions)
- Includes imports and simple constants (UPPER_CASE or primitives)

**Excluded:**
- Sayiir imports (`from "sayiir"`, `from 'sayiir'`)
- TypeScript type definitions (`interface`, `type` aliases) — compile-time only, not needed in runtime JavaScript
- Type-only imports (`import type`)
- Task wrapper calls (`task(...)` stripped from output)
- Workflow definitions (`flow(...)`, `branch(...)`)
- Object instantiation (`new ...`)
- Object literals and function call assignments
- Runtime execution code

**Output format:**
- Produces runtime JavaScript, not TypeScript
- Type annotations are stripped (e.g., `: Order` removed)
- Arrow functions converted to named functions

## Programmatic API Examples

The `examples/` directory contains programmatic usage examples:

### Export Examples
```bash
# Export Rust workflow
cargo run --example export_rust

# Export Python workflow
cargo run --example export_python

# Export JavaScript workflow
cargo run --example export_javascript
```

### Import Example
```bash
# Import OpenFlow JSON and generate standalone project
cargo run --example import_workflow
```

### Brimstone Execution Example
```bash
# Run pure JavaScript workflow with Brimstone (requires feature)
cargo run --example brimstone_run --features brimstone
```

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `brimstone` | Enables JavaScript execution via Brimstone interpreter | ❌ Disabled |

**Enable Brimstone:**
```bash
cargo install --path . --features brimstone
# Or in Cargo.toml
sayiir-openflow = { version = "1.0", features = ["brimstone"] }
```

## License

MIT
