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

- **Mermaid Embedded Code Support (Phase 3)** - Human-readable markdown format with embedded code
  - Extended export_mermaid() to append code blocks with metadata comments
  - Extended import_mermaid() to parse code blocks and extract metadata
  - Metadata format: `%%% task_id (language)`, `%%% Entry: function_name`, `%%% Dependencies: {...}`
  - Code blocks use standard markdown fence syntax (```rust, ```python, ```javascript)
  - Round-trip tests verify export → import → export produces identical output
  - Backward compatible (simple Mermaid flowcharts without code still work)
  - Documentation and examples for Mermaid workflow format
  - 48 total tests passing

- **Error Handling and Polish (Phase 4)** - Production-ready error handling and UX improvements
  - Enhanced error types with structured DependencyError (module_id, language, dependency, stderr)
  - Automatic retry with exponential backoff (3 attempts, 1s/2s/4s) for npm/pip network errors
  - Cache cleanup utility: `cleanup_stale_cache()` removes caches older than threshold (default 7 days)
  - Import preview: `preview_import_json()` and `preview_import_mermaid()` show summary before importing
  - Preview displays: module count, code sizes, languages, entry points, dependencies count
  - Improved error display with detailed context for compilation and dependency failures
  - Comprehensive error handling tests (compilation, dependency, missing runtime, retry behavior)
  - Cache cleanup tests with platform-independent filetime crate
  - Import preview tests with Display trait implementation
  - 57 total tests passing

### Dependencies
- Added `which` for runtime detection
- Added `dirs` for cache directory path
- Added `tokio` features: process, time
- Added `filetime` (dev) for cache cleanup tests
