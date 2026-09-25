# Changelog

## [Unreleased]

### Added
- **Embedded Code Support (Phase 1)** - Export/import workflows with embedded Rust, Node.js, and Python code
  - Extended OpenFlow JSON schema with `language`, `entry_point`, and `code` fields
  - Runtime detection for cargo, node, python3
  - Compilation pipeline for Rust (cargo build), Node.js (task.js wrapper), Python (task.py wrapper)
  - Execution layer with JSON input/output via stdout
  - Timeout support (default 30s)
  - Cache directory at `~/.sayiir/cache/<workflow_id>_<module_id>_<language>/`
  - Self-contained code only (no external dependencies in Phase 1)
  - 80%+ test coverage
  - Full end-to-end integration tests

- **External Dependencies Support (Phase 2)** - Add external dependency management for embedded code
  - Extended schema with optional `dependencies` field (JSON object: `{"package": "version"}`)
  - Rust: Generates `Cargo.toml` with dependencies, runs `cargo build --release`
  - Node.js: Generates `package.json`, runs `npm install`
  - Python: Generates `requirements.txt`, creates venv, runs `pip install`
  - Backward compatible (dependencies field is optional)
  - Multi-language integration tests (Rust→Node→Python chaining)
  - Documentation with examples for all three languages
  - 42 total tests passing

### Dependencies
- Added `which` for runtime detection
- Added `dirs` for cache directory path
- Added `tokio` features: process, time
