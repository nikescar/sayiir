# sayiir-openflow Import and Run Design Specification

> **Status**: Design specification for implementing import and direct execution of OpenFlow workflows
> 
> **Goal**: Enable `cargo-sayiir-openflow import` and `cargo-sayiir-openflow run` commands to import and execute OpenFlow JSON/Mermaid workflows without original source code

## Overview

Add two commands to cargo-sayiir-openflow CLI:

1. **`import`** - Convert OpenFlow JSON or Mermaid markdown to runnable Rust/Python/Node.js projects
2. **`run`** - Execute OpenFlow JSON or Mermaid workflows directly (compile embedded code, run tasks)

## Use Cases

### Use Case 1: Import and Modify
```bash
# Receive workflow.json from colleague
cargo sayiir-openflow import workflow.json --output-dir ./my-workflow
cd my-workflow
# Edit tasks in src/
cargo run  # Run modified workflow
```

### Use Case 2: Direct Execution
```bash
# Download workflow from Windmill
curl https://windmill/api/flows/video-pipeline > workflow.json

# Run directly without importing
cargo sayiir-openflow run workflow.json
```

### Use Case 3: Mermaid to Code
```bash
# Extract code from documentation
cargo sayiir-openflow import workflow.md --output-dir ./workflow-impl
```

## Command Specifications

### Import Command

**Signature:**
```bash
cargo-sayiir-openflow import <INPUT> [OPTIONS]

Arguments:
  <INPUT>  OpenFlow JSON or Mermaid markdown file

Options:
  -o, --output-dir <DIR>     Output directory (default: ./<workflow-name>)
  -l, --language <LANG>      Target language: rust, python, node (auto-detect if omitted)
  -f, --force                Overwrite existing directory
  --no-deps                  Skip dependency installation
```

**Behavior:**

1. Parse input file (auto-detect JSON vs Mermaid)
2. Extract workflow metadata, tasks, dependencies
3. Generate project structure:
   ```
   <workflow-name>/
   ├── Cargo.toml / package.json / pyproject.toml
   ├── src/
   │   ├── main.rs / index.ts / main.py
   │   └── tasks.rs / tasks.ts / tasks.py
   └── README.md
   ```
4. Write task source code to files
5. Install dependencies (if not `--no-deps`)
6. Print instructions for running

**Examples:**

```bash
# Import Rust workflow
cargo sayiir-openflow import workflow.json
# Output: Created ./video-pipeline/

# Import to custom directory
cargo sayiir-openflow import workflow.json -o ~/projects/my-workflow

# Force overwrite
cargo sayiir-openflow import workflow.json -o ./existing-dir --force

# Import Mermaid markdown
cargo sayiir-openflow import workflow.md
```

**Output Messages:**
```
✓ Parsed workflow: video-pipeline
✓ Detected language: Rust
✓ Extracted 10 tasks
✓ Created ./video-pipeline/
✓ Generated Cargo.toml
✓ Generated src/main.rs
✓ Generated src/tasks.rs
✓ Installed dependencies
✓ Ready to run: cd video-pipeline && cargo run
```

### Run Command

**Signature:**
```bash
cargo-sayiir-openflow run <INPUT> [OPTIONS]

Arguments:
  <INPUT>  OpenFlow JSON or Mermaid markdown file

Options:
  --input <JSON>             Workflow input as JSON string
  --input-file <FILE>        Workflow input from file
  --cache-dir <DIR>          Cache directory (default: ~/.sayiir/cache)
  --clean-cache              Remove cached builds before running
  --show-output              Show task stdout/stderr
```

**Behavior:**

1. Parse input file
2. Compile embedded code to cache directory:
   ```
   ~/.sayiir/cache/<workflow-id>/
   ├── <task-id-1>/
   │   ├── Cargo.toml / package.json / requirements.txt
   │   ├── src/ or main.py or index.js
   │   └── target/ or node_modules/ or venv/
   └── <task-id-2>/
       └── ...
   ```
3. Execute tasks in workflow order
4. Pass output of task N as input to task N+1
5. Print final result

**Examples:**

```bash
# Run workflow with no input
cargo sayiir-openflow run workflow.json

# Run with input
cargo sayiir-openflow run workflow.json --input '{"url": "https://example.com/video.mp4"}'

# Run with input from file
cargo sayiir-openflow run workflow.json --input-file input.json

# Show task output
cargo sayiir-openflow run workflow.json --show-output

# Clean cache and rebuild
cargo sayiir-openflow run workflow.json --clean-cache
```

**Output Messages:**
```
✓ Parsed workflow: video-pipeline
✓ Found 10 tasks
✓ Compiling download_video (Rust)... 2.3s
✓ Compiling validate_upload (Rust)... cached
✓ Executing download_video... 1.2s
✓ Executing validate_upload... 0.3s
✓ Executing transcode_720p... 45.6s
✓ Executing transcode_1080p... 89.2s
✓ Executing transcode_4k... 203.4s
✓ Executing generate_thumbnails... 5.1s
✓ Executing moderate_content... 2.3s
✓ Executing merge_results... 0.1s
✓ Executing update_database... 0.4s
✓ Executing notify_user... 0.2s

Result:
{
  "video_id": "abc123",
  "status": "completed",
  "formats": ["720p", "1080p", "4k"],
  "thumbnail_url": "https://cdn.example.com/thumbnails/abc123.jpg"
}
```

## Architecture

### Import Implementation

**Module**: `src/import.rs`

```rust
pub struct ImportConfig {
    pub input_path: PathBuf,
    pub output_dir: PathBuf,
    pub language: Option<Language>,
    pub force: bool,
    pub install_deps: bool,
}

pub fn import_workflow(config: ImportConfig) -> ExportResult<()> {
    // 1. Parse input file
    let spec = if config.input_path.extension() == Some("json") {
        parse_openflow_json(&config.input_path)?
    } else {
        parse_mermaid(&config.input_path)?
    };
    
    // 2. Detect language
    let language = config.language.unwrap_or_else(|| detect_language_from_spec(&spec));
    
    // 3. Generate project structure
    create_project_scaffold(&config.output_dir, language)?;
    
    // 4. Write task files
    write_tasks(&spec, &config.output_dir, language)?;
    
    // 5. Write main workflow file
    write_workflow_main(&spec, &config.output_dir, language)?;
    
    // 6. Install dependencies
    if config.install_deps {
        install_dependencies(&config.output_dir, language)?;
    }
    
    Ok(())
}
```

**Project Templates:**

Rust:
```rust
// Cargo.toml
[package]
name = "{{workflow_name}}"
version = "0.1.0"
edition = "2021"

[dependencies]
sayiir = "0.1"
serde_json = "1.0"
{{user_dependencies}}

// src/main.rs
use sayiir::*;
mod tasks;
use tasks::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workflow = flow("{{workflow_name}}")
        {{#each tasks}}
        .then({{this}})
        {{/each}}
        .build();
    
    let result = run_workflow(workflow, serde_json::json!({})).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

// src/tasks.rs
use sayiir::task;
use serde_json::Value;

{{#each task_definitions}}
#[task(id = "{{id}}")]
{{source_code}}
{{/each}}
```

Python:
```python
# pyproject.toml
[project]
name = "{{workflow_name}}"
version = "0.1.0"
dependencies = [
    "sayiir>=0.1.0",
    {{user_dependencies}}
]

# main.py
from sayiir import Flow, run_workflow
from tasks import *

workflow = (
    Flow("{{workflow_name}}")
    {{#each tasks}}
    .then({{this}})
    {{/each}}
    .build()
)

if __name__ == "__main__":
    result = run_workflow(workflow, {})
    print(result)

# tasks.py
from sayiir import task

{{#each task_definitions}}
{{source_code}}

{{/each}}
```

Node.js:
```typescript
// package.json
{
  "name": "{{workflow_name}}",
  "version": "0.1.0",
  "type": "module",
  "dependencies": {
    "sayiir": "^0.1.0",
    {{user_dependencies}}
  }
}

// index.ts
import { flow, runWorkflow } from "sayiir";
import * as tasks from "./tasks.js";

const workflow = flow("{{workflow_name}}")
  {{#each tasks}}
  .then(tasks.{{this}})
  {{/each}}
  .build();

const result = await runWorkflow(workflow, {});
console.log(JSON.stringify(result, null, 2));

// tasks.ts
import { task } from "sayiir";

{{#each task_definitions}}
{{source_code}}

{{/each}}
```

### Run Implementation

**Module**: `src/run.rs`

```rust
pub struct RunConfig {
    pub input_path: PathBuf,
    pub workflow_input: serde_json::Value,
    pub cache_dir: PathBuf,
    pub clean_cache: bool,
    pub show_output: bool,
}

pub async fn run_workflow(config: RunConfig) -> ExportResult<serde_json::Value> {
    // 1. Parse workflow
    let spec = parse_workflow(&config.input_path)?;
    
    // 2. Clean cache if requested
    if config.clean_cache {
        clean_workflow_cache(&config.cache_dir, &spec.summary)?;
    }
    
    // 3. Compile all tasks
    let compiled_tasks = compile_tasks(&spec, &config.cache_dir).await?;
    
    // 4. Execute tasks in order
    let mut result = config.workflow_input;
    for module in &spec.value.modules {
        let executable = &compiled_tasks[&module.id];
        result = execute_task(executable, result, config.show_output).await?;
        println!("✓ Executed {}", module.id);
    }
    
    Ok(result)
}

async fn compile_tasks(
    spec: &OpenFlowSpec,
    cache_dir: &Path,
) -> ExportResult<HashMap<String, PathBuf>> {
    let mut compiled = HashMap::new();
    
    for module in &spec.value.modules {
        let task_cache_dir = cache_dir
            .join(&spec.summary)
            .join(&module.id);
        
        // Check if already compiled
        if let Some(executable) = find_cached_executable(&task_cache_dir, &module.language)? {
            println!("✓ Compiling {} (cached)", module.id);
            compiled.insert(module.id.clone(), executable);
            continue;
        }
        
        // Compile task
        let start = std::time::Instant::now();
        let executable = compile_task(module, &task_cache_dir).await?;
        println!("✓ Compiling {} ({:.1}s)", module.id, start.elapsed().as_secs_f64());
        
        compiled.insert(module.id.clone(), executable);
    }
    
    Ok(compiled)
}

async fn compile_task(
    module: &OpenFlowModule,
    cache_dir: &Path,
) -> ExportResult<PathBuf> {
    std::fs::create_dir_all(cache_dir)?;
    
    match module.language.as_deref() {
        Some("rust") => compile_rust_task(module, cache_dir).await,
        Some("python") => compile_python_task(module, cache_dir).await,
        Some("node") | Some("javascript") | Some("typescript") => {
            compile_node_task(module, cache_dir).await
        }
        _ => Err(ExportError::UnsupportedLanguage(
            module.language.clone().unwrap_or_default()
        )),
    }
}

async fn compile_rust_task(
    module: &OpenFlowModule,
    cache_dir: &Path,
) -> ExportResult<PathBuf> {
    // 1. Write Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "task"
path = "src/main.rs"

[dependencies]
serde_json = "1.0"
{}"#,
        module.id,
        format_rust_dependencies(&module.dependencies)
    );
    std::fs::write(cache_dir.join("Cargo.toml"), cargo_toml)?;
    
    // 2. Write src/main.rs
    let main_rs = format!(
        r#"use serde_json::Value;

{}

fn main() -> Result<(), Box<dyn std::error::Error>> {{
    let input_json = std::env::args().nth(1).expect("Missing input JSON");
    let input: Value = serde_json::from_str(&input_json)?;
    let output = {}(input)?;
    println!("{{}}", serde_json::to_string(&output)?);
    Ok(())
}}"#,
        module.code.as_ref().unwrap_or(&String::new()),
        module.entry_point.as_ref().unwrap_or(&"run".to_string())
    );
    std::fs::create_dir_all(cache_dir.join("src"))?;
    std::fs::write(cache_dir.join("src/main.rs"), main_rs)?;
    
    // 3. Compile
    let output = tokio::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(cache_dir)
        .output()
        .await?;
    
    if !output.status.success() {
        return Err(ExportError::CompilationError {
            module_id: module.id.clone(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    
    Ok(cache_dir.join("target/release/task"))
}

async fn compile_python_task(
    module: &OpenFlowModule,
    cache_dir: &Path,
) -> ExportResult<PathBuf> {
    // 1. Write task.py
    let task_py = format!(
        r#"import sys
import json

{}

if __name__ == "__main__":
    input_json = sys.argv[1]
    input_data = json.loads(input_json)
    output = {}(input_data)
    print(json.dumps(output))
"#,
        module.code.as_ref().unwrap_or(&String::new()),
        module.entry_point.as_ref().unwrap_or(&"run".to_string())
    );
    std::fs::write(cache_dir.join("task.py"), task_py)?;
    
    // 2. Create venv and install dependencies
    if let Some(deps) = &module.dependencies {
        if !deps.is_empty() {
            // Create venv
            let output = tokio::process::Command::new("python3")
                .args(["-m", "venv", "venv"])
                .current_dir(cache_dir)
                .output()
                .await?;
            
            if !output.status.success() {
                return Err(ExportError::CompilationError {
                    module_id: module.id.clone(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                });
            }
            
            // Write requirements.txt
            let requirements: Vec<String> = deps
                .iter()
                .map(|(k, v)| format!("{}=={}", k, v.as_str().unwrap_or("")))
                .collect();
            std::fs::write(cache_dir.join("requirements.txt"), requirements.join("\n"))?;
            
            // Install dependencies
            let pip_path = cache_dir.join("venv/bin/pip");
            let output = tokio::process::Command::new(&pip_path)
                .args(["install", "-r", "requirements.txt"])
                .current_dir(cache_dir)
                .output()
                .await?;
            
            if !output.status.success() {
                return Err(ExportError::DependencyError {
                    module_id: module.id.clone(),
                    language: "python".to_string(),
                    dependency: "requirements.txt".to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                });
            }
        }
    }
    
    Ok(cache_dir.join("task.py"))
}

async fn compile_node_task(
    module: &OpenFlowModule,
    cache_dir: &Path,
) -> ExportResult<PathBuf> {
    // 1. Write task.js
    let task_js = format!(
        r#"{}

const inputJson = process.argv[2];
const input = JSON.parse(inputJson);
const output = await {}(input);
console.log(JSON.stringify(output));
"#,
        module.code.as_ref().unwrap_or(&String::new()),
        module.entry_point.as_ref().unwrap_or(&"run".to_string())
    );
    std::fs::write(cache_dir.join("task.js"), task_js)?;
    
    // 2. Install dependencies
    if let Some(deps) = &module.dependencies {
        if !deps.is_empty() {
            // Write package.json
            let package_json = serde_json::json!({
                "type": "module",
                "dependencies": deps
            });
            std::fs::write(
                cache_dir.join("package.json"),
                serde_json::to_string_pretty(&package_json)?
            )?;
            
            // npm install
            let output = tokio::process::Command::new("npm")
                .args(["install"])
                .current_dir(cache_dir)
                .output()
                .await?;
            
            if !output.status.success() {
                return Err(ExportError::DependencyError {
                    module_id: module.id.clone(),
                    language: "node".to_string(),
                    dependency: "package.json".to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                });
            }
        }
    }
    
    Ok(cache_dir.join("task.js"))
}

async fn execute_task(
    executable: &Path,
    input: serde_json::Value,
    show_output: bool,
) -> ExportResult<serde_json::Value> {
    let input_json = serde_json::to_string(&input)?;
    
    let mut cmd = match executable.extension().and_then(|s| s.to_str()) {
        Some("py") => {
            let mut c = tokio::process::Command::new("python3");
            c.arg(executable);
            c
        }
        Some("js") => {
            let mut c = tokio::process::Command::new("node");
            c.arg(executable);
            c
        }
        _ => {
            // Rust binary
            tokio::process::Command::new(executable)
        }
    };
    
    cmd.arg(&input_json);
    
    if !show_output {
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
    }
    
    let output = cmd.output().await?;
    
    if !output.status.success() {
        return Err(ExportError::ExecutionError {
            task: executable.display().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&output_str)?;
    
    Ok(result)
}
```

## Error Handling

**New Error Variants:**

```rust
pub enum ExportError {
    // ... existing variants
    
    /// Compilation failed
    CompilationError {
        module_id: String,
        stderr: String,
    },
    
    /// Dependency installation failed
    DependencyError {
        module_id: String,
        language: String,
        dependency: String,
        stderr: String,
    },
    
    /// Task execution failed
    ExecutionError {
        task: String,
        stderr: String,
    },
    
    /// Unsupported language
    UnsupportedLanguage(String),
    
    /// Output directory exists
    OutputDirExists(PathBuf),
}
```

## CLI Integration

**Update `src/bin/cargo-sayiir-openflow.rs`:**

```rust
#[derive(Parser)]
enum Command {
    /// Export workflow to OpenFlow JSON and Mermaid markdown
    Export {
        // ... existing export args
    },
    
    /// Import OpenFlow JSON or Mermaid to runnable project
    Import {
        /// Input file (OpenFlow JSON or Mermaid markdown)
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        /// Output directory
        #[arg(short, long, default_value = ".")]
        output_dir: PathBuf,
        
        /// Target language (auto-detect if not specified)
        #[arg(short, long, value_enum)]
        language: Option<Language>,
        
        /// Force overwrite existing directory
        #[arg(short, long)]
        force: bool,
        
        /// Skip dependency installation
        #[arg(long)]
        no_deps: bool,
    },
    
    /// Run OpenFlow JSON or Mermaid workflow directly
    Run {
        /// Input file (OpenFlow JSON or Mermaid markdown)
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        /// Workflow input as JSON string
        #[arg(long)]
        input_json: Option<String>,
        
        /// Workflow input from file
        #[arg(long)]
        input_file: Option<PathBuf>,
        
        /// Cache directory
        #[arg(long, default_value = "~/.sayiir/cache")]
        cache_dir: PathBuf,
        
        /// Clean cache before running
        #[arg(long)]
        clean_cache: bool,
        
        /// Show task stdout/stderr
        #[arg(long)]
        show_output: bool,
    },
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_import_rust_workflow() {
        let json = include_str!("../tests/fixtures/workflow.json");
        let spec = parse_openflow_json(json).unwrap();
        let temp_dir = TempDir::new().unwrap();
        
        import_workflow(ImportConfig {
            input_path: PathBuf::from("workflow.json"),
            output_dir: temp_dir.path().to_path_buf(),
            language: Some(Language::Rust),
            force: false,
            install_deps: false,
        }).unwrap();
        
        assert!(temp_dir.path().join("Cargo.toml").exists());
        assert!(temp_dir.path().join("src/main.rs").exists());
        assert!(temp_dir.path().join("src/tasks.rs").exists());
    }
    
    #[tokio::test]
    async fn test_compile_rust_task() {
        let module = OpenFlowModule {
            id: "hello".to_string(),
            language: Some("rust".to_string()),
            code: Some(r#"
                fn run(input: serde_json::Value) -> Result<serde_json::Value, String> {
                    Ok(serde_json::json!({"message": "Hello"}))
                }
            "#.to_string()),
            entry_point: Some("run".to_string()),
            dependencies: None,
        };
        
        let temp_dir = TempDir::new().unwrap();
        let executable = compile_rust_task(&module, temp_dir.path()).await.unwrap();
        
        assert!(executable.exists());
        assert!(executable.is_file());
    }
    
    #[tokio::test]
    async fn test_execute_task() {
        // ... test execution
    }
}
```

### Integration Tests

```bash
# Test import command
cargo test test_import_command

# Test run command
cargo test test_run_command

# Test round-trip (export → import → run)
cargo test test_export_import_run_roundtrip
```

## Implementation Tasks

### Phase 1: Import Command (Week 1)

- [ ] Task 1.1: Add `import` subcommand to CLI
- [ ] Task 1.2: Implement `parse_openflow_json()` and `parse_mermaid()`
- [ ] Task 1.3: Implement project scaffold generation (Rust)
- [ ] Task 1.4: Implement task file generation (Rust)
- [ ] Task 1.5: Implement dependency installation (Rust)
- [ ] Task 1.6: Add tests for Rust import
- [ ] Task 1.7: Implement Python import support
- [ ] Task 1.8: Implement Node.js import support
- [ ] Task 1.9: Add integration tests

### Phase 2: Run Command (Week 2)

- [ ] Task 2.1: Add `run` subcommand to CLI
- [ ] Task 2.2: Implement task compilation (Rust)
- [ ] Task 2.3: Implement task execution (Rust)
- [ ] Task 2.4: Implement caching logic
- [ ] Task 2.5: Add tests for Rust run
- [ ] Task 2.6: Implement Python compilation/execution
- [ ] Task 2.7: Implement Node.js compilation/execution
- [ ] Task 2.8: Add error handling and retry logic
- [ ] Task 2.9: Add integration tests

### Phase 3: Documentation and Polish (Week 3)

- [ ] Task 3.1: Update README with import/run examples
- [ ] Task 3.2: Add example workflows for testing
- [ ] Task 3.3: Add performance benchmarks
- [ ] Task 3.4: Optimize caching strategy
- [ ] Task 3.5: Add progress indicators for long compilations
- [ ] Task 3.6: Add `--dry-run` support for run command
- [ ] Task 3.7: Add telemetry/logging
- [ ] Task 3.8: Update CHANGELOG
- [ ] Task 3.9: Release v0.2.0

## Success Criteria

✅ **Import works:**
```bash
cargo sayiir-openflow import workflow.json
cd video-pipeline && cargo run
# Output: workflow executes successfully
```

✅ **Run works:**
```bash
cargo sayiir-openflow run workflow.json --input '{"url": "..."}'
# Output: workflow result JSON
```

✅ **Round-trip works:**
```bash
# Original project
cd examples/video-pipeline-rs
cargo sayiir-openflow export

# Import and run
cargo sayiir-openflow import workflow.json -o /tmp/imported
cd /tmp/imported && cargo run

# Direct run
cargo sayiir-openflow run workflow.json --input-file input.json
```

✅ **All languages supported:**
- Rust: compile to binary, execute
- Python: venv + pip install, execute with python3
- Node.js: npm install, execute with node

✅ **Caching works:**
- First run compiles (slow)
- Subsequent runs use cache (fast)
- `--clean-cache` forces recompile

## Dependencies

**New Cargo dependencies:**

```toml
[dependencies]
tokio = { version = "1.0", features = ["process", "rt-multi-thread", "macros"] }
tempfile = "3.8"  # For testing

[dev-dependencies]
assert_cmd = "2.0"  # CLI testing
predicates = "3.0"  # Assertions
```

## Security Considerations

⚠️ **Code Execution Risk:**
- Running arbitrary code from JSON/Mermaid files
- Mitigation: compile in isolated cache directory, warn users

⚠️ **Dependency Installation:**
- Running `cargo build`, `npm install`, `pip install`
- Mitigation: use official package managers, verify checksums

⚠️ **File System Access:**
- Writing to cache directory
- Mitigation: use `~/.sayiir/cache` by default, allow custom via `--cache-dir`

## Performance Considerations

**Compilation Time:**
- Rust: 2-30s per task (depending on dependencies)
- Python: <1s (no compilation, just venv setup)
- Node.js: 1-5s (npm install time)

**Caching Strategy:**
- Cache compiled binaries by task ID + code hash
- Invalidate on code change
- Keep last 30 days of cache

**Parallel Compilation:**
- Compile independent tasks in parallel
- Use `tokio::task::spawn()` for parallelism

## Future Enhancements

### v0.3.0: Advanced Features
- Parallel task execution (fork/join)
- Task retry policies
- Timeouts
- Signal handling
- Workflow pause/resume

### v0.4.0: Remote Execution
- Execute on remote workers
- Distributed task execution
- Cloud function deployment

### v0.5.0: Visual Editor
- Web UI for editing Mermaid diagrams
- Live preview
- Collaborative editing

## References

- OpenFlow Spec: https://github.com/windmill-labs/windmill
- Sayiir Runtime: https://github.com/sayiir/sayiir
- Cargo Subcommands: https://doc.rust-lang.org/cargo/reference/external-tools.html

## Appendix: Example Workflows

### Example 1: Simple Rust Workflow

**workflow.json:**
```json
{
  "summary": "hello-world",
  "value": {
    "modules": [
      {
        "id": "greet",
        "value": {
          "type": "script",
          "path": "greet",
          "language": "rust",
          "entry_point": "run",
          "code": "fn run(input: serde_json::Value) -> Result<serde_json::Value, String> {\n    let name = input[\"name\"].as_str().unwrap_or(\"World\");\n    Ok(serde_json::json!({\"message\": format!(\"Hello, {}!\", name)}))\n}",
          "dependencies": {}
        }
      }
    ]
  }
}
```

**Run:**
```bash
cargo sayiir-openflow run workflow.json --input '{"name": "Alice"}'
# Output: {"message": "Hello, Alice!"}
```

### Example 2: Multi-Task Python Workflow

**workflow.json:**
```json
{
  "summary": "data-pipeline",
  "value": {
    "modules": [
      {
        "id": "fetch",
        "value": {
          "type": "script",
          "language": "python",
          "entry_point": "run",
          "code": "import requests\n\ndef run(input_data):\n    response = requests.get(input_data['url'])\n    return {'text': response.text}",
          "dependencies": {"requests": "2.31.0"}
        }
      },
      {
        "id": "process",
        "value": {
          "type": "script",
          "language": "python",
          "entry_point": "run",
          "code": "def run(input_data):\n    text = input_data['text']\n    return {'word_count': len(text.split())}",
          "dependencies": {}
        }
      }
    ]
  }
}
```

**Run:**
```bash
cargo sayiir-openflow run workflow.json --input '{"url": "https://example.com"}'
# Output: {"word_count": 1234}
```

## Questions for Review

1. Should `import` generate a single-file or multi-file project structure?
2. Should `run` support parallel task execution (fork/join)?
3. Should we cache at binary level or source level?
4. Should we support custom task runtimes (WASM, containers)?
5. Should we validate dependencies against known vulnerabilities?

---

**End of Specification**
