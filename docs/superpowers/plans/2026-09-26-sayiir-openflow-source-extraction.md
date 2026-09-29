# sayiir-openflow Source Code Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable `cargo-sayiir-openflow export` to extract workflows and tasks from multi-file Rust, Python, and Node.js projects into portable OpenFlow JSON/Mermaid formats.

**Architecture:** Multi-file scanner → Task registry → Language-specific parsers (Rust/Python/Node) → Task matcher → OpenFlow builder. Regex-based parsing for Python/Node to avoid heavy dependencies.

**Tech Stack:** Rust (syn, quote), regex, glob/walkdir for file scanning

**Spec:** `docs/superpowers/specs/2026-09-26-sayiir-openflow-source-extraction.md`

## Global Constraints

- Rust edition 2021
- No external Python/Node.js AST tools (use regex parsing)
- Preserve existing `extract::rust::extract_rust_task` for backward compatibility
- Graceful degradation: missing tasks → warn, don't fail
- All new code in `/home/wj/work/sayiir/sayiir-openflow/`

---

## Task 1: Project Setup

**Files:**
- Modify: `/home/wj/work/sayiir/sayiir-openflow/Cargo.toml`
- Delete: `/home/wj/work/sayiir/sayiir-openflow/src/bin/sayiir-openflow.rs`

**Interfaces:**
- Consumes: None
- Produces: `regex` and `walkdir` dependencies available

- [ ] **Step 1: Add dependencies to Cargo.toml**

```toml
# Add after existing dependencies
regex = "1.10"
walkdir = "2.4"
```

- [ ] **Step 2: Delete old CLI binary**

```bash
rm /home/wj/work/sayiir/sayiir-openflow/src/bin/sayiir-openflow.rs
```

- [ ] **Step 3: Verify build still works**

```bash
cd /home/wj/work/sayiir/sayiir-openflow
cargo build
```

Expected: Builds successfully (sayiir-openflow binary removed, cargo-sayiir-openflow remains)

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml src/bin/
git commit -m "chore: add regex/walkdir deps, remove sayiir-openflow binary"
```

---

## Task 2: Multi-File Scanner

**Files:**
- Create: `/home/wj/work/sayiir/sayiir-openflow/src/scan.rs`
- Modify: `/home/wj/work/sayiir/sayiir-openflow/src/lib.rs` (add `pub mod scan;`)

**Interfaces:**
- Consumes: `ProjectLanguage` from `detect.rs`
- Produces: 
  ```rust
  pub struct ProjectScan {
      pub workflow_file: PathBuf,
      pub task_files: Vec<PathBuf>,
  }
  pub fn scan_project(project_dir: &Path, language: ProjectLanguage) -> Result<ProjectScan>
  ```

- [ ] **Step 1: Write test for Rust project scanning**

Create `/home/wj/work/sayiir/sayiir-openflow/tests/test_scan.rs`:

```rust
use sayiir_openflow::{scan::scan_project, ProjectLanguage};
use std::path::PathBuf;

#[test]
fn test_scan_rust_project() {
    let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../examples/video-pipeline-rs");
    
    let scan = scan_project(&project_dir, ProjectLanguage::Rust).unwrap();
    
    // Should find workflow in main.rs
    assert!(scan.workflow_file.ends_with("main.rs"));
    
    // Should find multiple .rs files
    assert!(scan.task_files.len() >= 2);
    assert!(scan.task_files.iter().any(|p| p.ends_with("tasks.rs")));
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test test_scan_rust_project
```

Expected: FAIL with "module `scan` not found"

- [ ] **Step 3: Create scan.rs module**

Create `/home/wj/work/sayiir/sayiir-openflow/src/scan.rs`:

```rust
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::{ProjectLanguage, Result, ExportError};

pub struct ProjectScan {
    pub workflow_file: PathBuf,
    pub task_files: Vec<PathBuf>,
}

pub fn scan_project(project_dir: &Path, language: ProjectLanguage) -> Result<ProjectScan> {
    match language {
        ProjectLanguage::Rust => scan_rust_project(project_dir),
        ProjectLanguage::Python => scan_python_project(project_dir),
        ProjectLanguage::Node => scan_node_project(project_dir),
    }
}

fn scan_rust_project(project_dir: &Path) -> Result<ProjectScan> {
    let src_dir = project_dir.join("src");
    
    // Collect all .rs files in src/
    let mut rs_files: Vec<PathBuf> = WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        .map(|e| e.path().to_path_buf())
        .collect();
    
    // Sort for deterministic order
    rs_files.sort();
    
    // Workflow file priority: src/main.rs, src/lib.rs, first .rs file
    let workflow_file = if src_dir.join("main.rs").exists() {
        src_dir.join("main.rs")
    } else if src_dir.join("lib.rs").exists() {
        src_dir.join("lib.rs")
    } else {
        rs_files.first()
            .ok_or_else(|| ExportError::NoWorkflowFound)?
            .clone()
    };
    
    Ok(ProjectScan {
        workflow_file,
        task_files: rs_files,
    })
}

fn scan_python_project(project_dir: &Path) -> Result<ProjectScan> {
    let mut py_files: Vec<PathBuf> = WalkDir::new(project_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            // Skip venv, __pycache__, .venv, site-packages
            let skip = path.components().any(|c| {
                matches!(c.as_os_str().to_str(), Some("venv" | "__pycache__" | ".venv" | "site-packages"))
            });
            !skip && path.extension().map_or(false, |ext| ext == "py")
        })
        .map(|e| e.path().to_path_buf())
        .collect();
    
    py_files.sort();
    
    // Workflow file priority: main.py, app.py, workflow.py, __main__.py, first .py
    let workflow_file = ["main.py", "app.py", "workflow.py", "__main__.py"]
        .iter()
        .map(|name| project_dir.join(name))
        .find(|p| p.exists())
        .or_else(|| py_files.first().cloned())
        .ok_or_else(|| ExportError::NoWorkflowFound)?;
    
    Ok(ProjectScan {
        workflow_file,
        task_files: py_files,
    })
}

fn scan_node_project(project_dir: &Path) -> Result<ProjectScan> {
    let src_dir = project_dir.join("src");
    
    // Collect .ts and .js files, prefer src/ over root
    let mut node_files: Vec<PathBuf> = if src_dir.exists() {
        WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().map_or(false, |ext| ext == "ts" || ext == "js")
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    } else {
        WalkDir::new(project_dir)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().map_or(false, |ext| ext == "ts" || ext == "js")
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    };
    
    node_files.sort();
    
    // Workflow file priority: src/index.ts, index.ts, src/main.ts, main.ts, first file
    let workflow_file = [
        src_dir.join("index.ts"),
        project_dir.join("index.ts"),
        src_dir.join("main.ts"),
        project_dir.join("main.ts"),
    ]
    .iter()
    .find(|p| p.exists())
    .cloned()
    .or_else(|| node_files.first().cloned())
    .ok_or_else(|| ExportError::NoWorkflowFound)?;
    
    Ok(ProjectScan {
        workflow_file,
        task_files: node_files,
    })
}
```

- [ ] **Step 4: Export scan module in lib.rs**

Add to `/home/wj/work/sayiir/sayiir-openflow/src/lib.rs` after existing `pub mod` declarations:

```rust
pub mod scan;
```

- [ ] **Step 5: Run test to verify it passes**

```bash
cargo test test_scan_rust_project
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/scan.rs src/lib.rs tests/test_scan.rs
git commit -m "feat(scan): add multi-file project scanner for Rust/Python/Node"
```

---

## Task 3: Task Registry

**Files:**
- Modify: `/home/wj/work/sayiir/sayiir-openflow/src/extract/mod.rs`

**Interfaces:**
- Consumes: `TaskSource` from `extract/mod.rs`
- Produces:
  ```rust
  pub struct TaskRegistry {
      tasks: HashMap<String, TaskSource>,
  }
  impl TaskRegistry {
      pub fn new() -> Self
      pub fn insert(&mut self, task: TaskSource)
      pub fn get(&self, task_id: &str) -> Option<&TaskSource>
      pub fn len(&self) -> usize
  }
  ```

- [ ] **Step 1: Write test for task registry**

Add to `/home/wj/work/sayiir/sayiir-openflow/src/extract/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_registry() {
        let mut registry = TaskRegistry::new();
        
        registry.insert(TaskSource {
            id: "task1".to_string(),
            entry_point: "run".to_string(),
            source_code: "fn run() {}".to_string(),
        });
        
        registry.insert(TaskSource {
            id: "task2".to_string(),
            entry_point: "execute".to_string(),
            source_code: "fn execute() {}".to_string(),
        });
        
        assert_eq!(registry.len(), 2);
        assert!(registry.get("task1").is_some());
        assert!(registry.get("task2").is_some());
        assert!(registry.get("task3").is_none());
        
        // Overwrite test
        registry.insert(TaskSource {
            id: "task1".to_string(),
            entry_point: "new_run".to_string(),
            source_code: "fn new_run() {}".to_string(),
        });
        
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.get("task1").unwrap().entry_point, "new_run");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test test_task_registry
```

Expected: FAIL with "TaskRegistry not found"

- [ ] **Step 3: Implement TaskRegistry**

Add to `/home/wj/work/sayiir/sayiir-openflow/src/extract/mod.rs` after `TaskSource` definition:

```rust
use std::collections::HashMap;

pub struct TaskRegistry {
    tasks: HashMap<String, TaskSource>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }
    
    pub fn insert(&mut self, task: TaskSource) {
        self.tasks.insert(task.id.clone(), task);
    }
    
    pub fn get(&self, task_id: &str) -> Option<&TaskSource> {
        self.tasks.get(task_id)
    }
    
    pub fn len(&self) -> usize {
        self.tasks.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test test_task_registry
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/extract/mod.rs
git commit -m "feat(extract): add TaskRegistry for task storage and lookup"
```

---

## Task 4: Rust Multi-Task Extraction

**Files:**
- Modify: `/home/wj/work/sayiir/sayiir-openflow/src/extract/rust.rs`

**Interfaces:**
- Consumes: `TaskSource` from `extract/mod.rs`
- Produces:
  ```rust
  pub fn extract_all_rust_tasks(source: &str) -> Result<Vec<TaskSource>>
  ```

- [ ] **Step 1: Write test for extracting all Rust tasks**

Add to `/home/wj/work/sayiir/sayiir-openflow/src/extract/rust.rs` in `#[cfg(test)] mod tests`:

```rust
#[test]
fn test_extract_all_rust_tasks() {
    let source = r#"
#[task(id = "task1")]
fn first_task() {}

#[task(id = "task2")]
async fn second_task(input: String) -> Result<String, BoxError> {
    Ok(input)
}

#[task]
fn third_task() {}

fn not_a_task() {}
"#;
    
    let tasks = extract_all_rust_tasks(source).unwrap();
    
    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0].id, "task1");
    assert_eq!(tasks[1].id, "task2");
    assert_eq!(tasks[2].id, "third_task"); // Inferred from function name
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test test_extract_all_rust_tasks
```

Expected: FAIL with "extract_all_rust_tasks not found"

- [ ] **Step 3: Implement extract_all_rust_tasks**

Add to `/home/wj/work/sayiir/sayiir-openflow/src/extract/rust.rs`:

```rust
pub fn extract_all_rust_tasks(source: &str) -> Result<Vec<TaskSource>> {
    let ast: File = syn::parse_str(source)
        .map_err(|e| ExportError::InvalidWorkflowSyntax(format!("Failed to parse Rust: {}", e)))?;
    
    let mut tasks = Vec::new();
    
    for item in &ast.items {
        if let Item::Fn(func) = item {
            if let Some(task_id) = get_task_id(&func.attrs, &func.sig.ident) {
                let entry_point = func.sig.ident.to_string();
                let source_code = func.to_token_stream().to_string();
                
                tasks.push(TaskSource {
                    id: task_id,
                    source_code,
                    entry_point,
                });
            }
        }
    }
    
    Ok(tasks)
}

fn get_task_id(attrs: &[Attribute], fn_ident: &syn::Ident) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("task") {
            // If #[task(id = "explicit_id")], use explicit_id
            if let Ok(list) = attr.meta.require_list() {
                let tokens = list.tokens.to_string();
                // Simple extraction: look for id = "value"
                if let Some(start) = tokens.find(r#"id = ""#) {
                    let after = &tokens[start + 6..];
                    if let Some(end) = after.find('"') {
                        return Some(after[..end].to_string());
                    }
                }
            }
            // If just #[task], infer from function name
            return Some(fn_ident.to_string());
        }
    }
    None
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test test_extract_all_rust_tasks
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/extract/rust.rs
git commit -m "feat(extract): add extract_all_rust_tasks for multi-file scanning"
```

---

## Task 5: Python Task Extraction

**Files:**
- Create: `/home/wj/work/sayiir/sayiir-openflow/src/extract/python.rs`
- Modify: `/home/wj/work/sayiir/sayiir-openflow/src/extract/mod.rs` (add `pub mod python;`)

**Interfaces:**
- Consumes: `TaskSource` from `extract/mod.rs`
- Produces:
  ```rust
  pub fn extract_all_python_tasks(source: &str) -> Result<Vec<TaskSource>>
  ```

- [ ] **Step 1: Write test for Python task extraction**

Create `/home/wj/work/sayiir/sayiir-openflow/tests/test_extract_python.rs`:

```rust
use sayiir_openflow::extract::python::extract_all_python_tasks;

#[test]
fn test_extract_all_python_tasks() {
    let source = r#"
@task
def submit_expense(expense: dict) -> dict:
    print(f"Expense submitted")
    return {**expense, "status": "pending"}

@task
def process_approved(approval: dict) -> str:
    return f"Approved"

def not_a_task():
    pass
"#;
    
    let tasks = extract_all_python_tasks(source).unwrap();
    
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].id, "submit_expense");
    assert_eq!(tasks[0].entry_point, "submit_expense");
    assert!(tasks[0].source_code.contains("@task"));
    assert_eq!(tasks[1].id, "process_approved");
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test test_extract_all_python_tasks
```

Expected: FAIL with "module `python` not found in `extract`"

- [ ] **Step 3: Create python.rs module**

Create `/home/wj/work/sayiir/sayiir-openflow/src/extract/python.rs`:

```rust
use crate::extract::TaskSource;
use crate::error::Result;

pub fn extract_all_python_tasks(source: &str) -> Result<Vec<TaskSource>> {
    let mut tasks = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i].trim();
        
        // Look for @task decorator
        if line == "@task" {
            i += 1;
            if i >= lines.len() {
                break;
            }
            
            // Next line should be: def function_name(...)
            let def_line = lines[i].trim();
            if let Some(fn_name) = parse_function_name(def_line) {
                // Extract function body (indent-aware)
                let (fn_source, end_idx) = extract_python_function(&lines, i);
                
                tasks.push(TaskSource {
                    id: fn_name.clone(),
                    entry_point: fn_name,
                    source_code: format!("@task\n{}", fn_source),
                });
                
                i = end_idx;
            }
        }
        
        i += 1;
    }
    
    Ok(tasks)
}

fn parse_function_name(line: &str) -> Option<String> {
    if !line.starts_with("def ") {
        return None;
    }
    
    let after_def = &line[4..];
    let paren_pos = after_def.find('(')?;
    let fn_name = after_def[..paren_pos].trim();
    Some(fn_name.to_string())
}

fn extract_python_function(lines: &[&str], start: usize) -> (String, usize) {
    let def_line = lines[start];
    let base_indent = count_leading_spaces(def_line);
    
    let mut fn_lines = vec![def_line];
    let mut i = start + 1;
    
    while i < lines.len() {
        let line = lines[i];
        
        // Empty lines are part of the function
        if line.trim().is_empty() {
            fn_lines.push(line);
            i += 1;
            continue;
        }
        
        let line_indent = count_leading_spaces(line);
        
        // If indentation <= base, function ended
        if line_indent <= base_indent {
            break;
        }
        
        fn_lines.push(line);
        i += 1;
    }
    
    (fn_lines.join("\n"), i)
}

fn count_leading_spaces(line: &str) -> usize {
    line.chars().take_while(|c| c.is_whitespace()).count()
}
```

- [ ] **Step 4: Export python module**

Add to `/home/wj/work/sayiir/sayiir-openflow/src/extract/mod.rs`:

```rust
pub mod python;
```

- [ ] **Step 5: Run test to verify it passes**

```bash
cargo test test_extract_all_python_tasks
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/extract/python.rs src/extract/mod.rs tests/test_extract_python.rs
git commit -m "feat(extract): add Python @task extraction"
```

---

## Task 6: Python Workflow Parsing

**Files:**
- Create: `/home/wj/work/sayiir/sayiir-openflow/src/parse/python.rs`
- Modify: `/home/wj/work/sayiir/sayiir-openflow/src/parse/mod.rs` (add `pub mod python;`)

**Interfaces:**
- Consumes: None
- Produces:
  ```rust
  pub struct PythonWorkflow {
      pub name: String,
      pub task_names: Vec<String>,
  }
  pub fn parse_python_workflow(source: &str) -> Result<PythonWorkflow>
  ```

- [ ] **Step 1: Write test for Python workflow parsing**

Create `/home/wj/work/sayiir/sayiir-openflow/tests/test_parse_python.rs`:

```rust
use sayiir_openflow::parse::python::parse_python_workflow;

#[test]
fn test_parse_python_workflow() {
    let source = r#"
workflow = (
    Flow("expense-approval")
    .then(submit_expense)
    .wait_for_signal("approval", timeout=timedelta(hours=48))
    .then(process_approved)
    .build()
)
"#;
    
    let wf = parse_python_workflow(source).unwrap();
    
    assert_eq!(wf.name, "expense-approval");
    assert_eq!(wf.task_names, vec!["submit_expense", "process_approved"]);
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test test_parse_python_workflow
```

Expected: FAIL with "module `python` not found in `parse`"

- [ ] **Step 3: Create python.rs parser**

Create `/home/wj/work/sayiir/sayiir-openflow/src/parse/python.rs`:

```rust
use crate::error::{ExportError, Result};
use regex::Regex;

pub struct PythonWorkflow {
    pub name: String,
    pub task_names: Vec<String>,
}

pub fn parse_python_workflow(source: &str) -> Result<PythonWorkflow> {
    let name = extract_flow_name(source)
        .ok_or_else(|| ExportError::NoWorkflowFound)?;
    
    let task_names = extract_then_calls(source);
    
    if task_names.is_empty() {
        return Err(ExportError::NoWorkflowFound);
    }
    
    Ok(PythonWorkflow { name, task_names })
}

fn extract_flow_name(source: &str) -> Option<String> {
    let pattern = r#"Flow\s*\(\s*["']([^"']+)["']\s*\)"#;
    let re = Regex::new(pattern).ok()?;
    let caps = re.captures(source)?;
    Some(caps.get(1)?.as_str().to_string())
}

fn extract_then_calls(source: &str) -> Vec<String> {
    let pattern = r#"\.then\s*\(\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)"#;
    let re = Regex::new(pattern).unwrap();
    
    re.captures_iter(source)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}
```

- [ ] **Step 4: Export python module**

Add to `/home/wj/work/sayiir/sayiir-openflow/src/parse/mod.rs`:

```rust
pub mod python;
```

- [ ] **Step 5: Run test to verify it passes**

```bash
cargo test test_parse_python_workflow
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/parse/python.rs src/parse/mod.rs tests/test_parse_python.rs
git commit -m "feat(parse): add Python Flow().then().build() parser"
```

---

## Task 7: Node.js Task Extraction

**Files:**
- Modify: `/home/wj/work/sayiir/sayiir-openflow/src/extract/node.rs`

**Interfaces:**
- Consumes: `TaskSource` from `extract/mod.rs`
- Produces:
  ```rust
  pub fn extract_all_node_tasks(source: &str) -> Result<Vec<TaskSource>>
  ```

- [ ] **Step 1: Write test for Node task extraction**

Create `/home/wj/work/sayiir/sayiir-openflow/tests/test_extract_node.rs`:

```rust
use sayiir_openflow::extract::node::extract_all_node_tasks;

#[test]
fn test_extract_all_node_tasks() {
    let source = r#"
const prepareDoc = task("ingest:prepare", async (input: IngestInput): Promise<PreparedDoc> => {
    const result = await fetch(input.url);
    return { docId: "123", chunkCount: 5 };
});

const embedAndIndex = task("ingest:embed", async (input) => {
    return { success: true };
});
"#;
    
    let tasks = extract_all_node_tasks(source).unwrap();
    
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].id, "ingest:prepare");
    assert_eq!(tasks[0].entry_point, "prepareDoc");
    assert!(tasks[0].source_code.contains("task("));
    assert_eq!(tasks[1].id, "ingest:embed");
    assert_eq!(tasks[1].entry_point, "embedAndIndex");
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test test_extract_all_node_tasks
```

Expected: FAIL with "extract_all_node_tasks not found"

- [ ] **Step 3: Implement extract_all_node_tasks**

Replace contents of `/home/wj/work/sayiir/sayiir-openflow/src/extract/node.rs`:

```rust
use crate::extract::TaskSource;
use crate::error::{ExportError, Result};
use regex::Regex;

pub fn extract_all_node_tasks(source: &str) -> Result<Vec<TaskSource>> {
    let mut tasks = Vec::new();
    
    let pattern = r#"(?:const|let|var)\s+(\w+)\s*=\s*task\s*\(\s*["']([^"']+)["']\s*,\s*async"#;
    let re = Regex::new(pattern).unwrap();
    
    for cap in re.captures_iter(source) {
        let var_name = cap.get(1).unwrap().as_str();
        let task_id = cap.get(2).unwrap().as_str();
        let task_start = cap.get(0).unwrap().start();
        
        if let Ok(task_source) = extract_task_body(source, task_start) {
            tasks.push(TaskSource {
                id: task_id.to_string(),
                entry_point: var_name.to_string(),
                source_code: task_source,
            });
        }
    }
    
    Ok(tasks)
}

fn extract_task_body(source: &str, start: usize) -> Result<String> {
    let after_start = &source[start..];
    let first_brace = after_start.find('{')
        .ok_or_else(|| ExportError::InvalidWorkflowSyntax("No opening brace".into()))?;
    
    let brace_start = start + first_brace;
    
    let mut depth = 0;
    let mut end_pos = brace_start;
    
    for (i, ch) in source[brace_start..].chars().enumerate() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end_pos = brace_start + i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    
    // Add closing );
    while end_pos < source.len() {
        let ch = source.chars().nth(end_pos).unwrap();
        if ch == ';' {
            end_pos += 1;
            break;
        } else if !ch.is_whitespace() && ch != ')' {
            break;
        }
        end_pos += 1;
    }
    
    Ok(source[start..end_pos].to_string())
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test test_extract_all_node_tasks
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/extract/node.rs tests/test_extract_node.rs
git commit -m "feat(extract): add Node.js task() extraction"
```

---

## Task 8: Node.js Workflow Parsing

**Files:**
- Create: `/home/wj/work/sayiir/sayiir-openflow/src/parse/node.rs`
- Modify: `/home/wj/work/sayiir/sayiir-openflow/src/parse/mod.rs` (add `pub mod node;`)

**Interfaces:**
- Consumes: None
- Produces:
  ```rust
  pub struct NodeWorkflow {
      pub name: String,
      pub task_names: Vec<String>,
  }
  pub fn parse_node_workflow(source: &str) -> Result<NodeWorkflow>
  ```

- [ ] **Step 1: Write test for Node workflow parsing**

Create `/home/wj/work/sayiir/sayiir-openflow/tests/test_parse_node.rs`:

```rust
use sayiir_openflow::parse::node::parse_node_workflow;

#[test]
fn test_parse_node_workflow() {
    let source = r#"
export function buildIngestWorkflow(ctx: RagContext): Workflow<IngestInput, IngestOutput> {
    return flow<IngestInput, IngestOutput>("ingest-workflow")
        .then(prepareDoc)
        .then(embedAndIndex)
        .build();
}
"#;
    
    let wf = parse_node_workflow(source).unwrap();
    
    assert_eq!(wf.name, "ingest-workflow");
    assert_eq!(wf.task_names, vec!["prepareDoc", "embedAndIndex"]);
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test test_parse_node_workflow
```

Expected: FAIL with "module `node` not found in `parse`"

- [ ] **Step 3: Create node.rs parser**

Create `/home/wj/work/sayiir/sayiir-openflow/src/parse/node.rs`:

```rust
use crate::error::{ExportError, Result};
use regex::Regex;

pub struct NodeWorkflow {
    pub name: String,
    pub task_names: Vec<String>,
}

pub fn parse_node_workflow(source: &str) -> Result<NodeWorkflow> {
    let name = extract_flow_name(source)
        .ok_or_else(|| ExportError::NoWorkflowFound)?;
    
    let task_names = extract_then_calls(source);
    
    if task_names.is_empty() {
        return Err(ExportError::NoWorkflowFound);
    }
    
    Ok(NodeWorkflow { name, task_names })
}

fn extract_flow_name(source: &str) -> Option<String> {
    let pattern = r#"flow(?:<[^>]+>)?\s*\(\s*["']([^"']+)["']\s*\)"#;
    let re = Regex::new(pattern).ok()?;
    let caps = re.captures(source)?;
    Some(caps.get(1)?.as_str().to_string())
}

fn extract_then_calls(source: &str) -> Vec<String> {
    let pattern = r#"\.then\s*\(\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)"#;
    let re = Regex::new(pattern).unwrap();
    
    re.captures_iter(source)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}
```

- [ ] **Step 4: Export node module**

Add to `/home/wj/work/sayiir/sayiir-openflow/src/parse/mod.rs`:

```rust
pub mod node;
```

- [ ] **Step 5: Run test to verify it passes**

```bash
cargo test test_parse_node_workflow
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/parse/node.rs src/parse/mod.rs tests/test_parse_node.rs
git commit -m "feat(parse): add Node.js flow().then().build() parser"
```

---

## Task 9: Update CLI with Multi-File Pipeline

**Files:**
- Modify: `/home/wj/work/sayiir/sayiir-openflow/src/bin/cargo-sayiir-openflow.rs`

**Interfaces:**
- Consumes: All modules from previous tasks
- Produces: Working `cargo-sayiir-openflow export` command

- [ ] **Step 1: Test current CLI on video-pipeline-rs**

```bash
cd /home/wj/work/sayiir/examples/video-pipeline-rs
cargo-sayiir-openflow sayiir-openflow export --dry-run
```

Expected: Warnings about missing tasks (baseline before fix)

- [ ] **Step 2: Replace export_workflow function**

Replace the `export_workflow` function in `/home/wj/work/sayiir/sayiir-openflow/src/bin/cargo-sayiir-openflow.rs` (lines 71-187) with:

```rust
fn export_workflow(
    input: Option<PathBuf>,
    format: Format,
    output: PathBuf,
    mermaid_output: PathBuf,
    dry_run: bool,
) -> error::ExportResult<()> {
    use sayiir_openflow::extract::TaskRegistry;
    
    // 1. Detect language
    let project_dir = std::env::current_dir().map_err(|e| error::ExportError::FileReadError {
        path: PathBuf::from("."),
        source: e,
    })?;
    
    let language = detect_language(&project_dir)?;
    println!("✓ Detected {:?} project", language);
    
    // 2. Scan project files
    let project_scan = sayiir_openflow::scan::scan_project(&project_dir, language)?;
    println!("✓ Scanned {} files", project_scan.task_files.len());
    
    // 3. Build task registry
    let mut task_registry = TaskRegistry::new();
    let lang_enum = match language {
        ProjectLanguage::Rust => Language::Rust,
        ProjectLanguage::Python => Language::Python,
        ProjectLanguage::Node => Language::Node,
    };
    
    for file_path in &project_scan.task_files {
        let source = std::fs::read_to_string(file_path).map_err(|e| {
            error::ExportError::FileReadError {
                path: file_path.clone(),
                source: e,
            }
        })?;
        
        let tasks = match language {
            ProjectLanguage::Rust => sayiir_openflow::extract::rust::extract_all_rust_tasks(&source)?,
            ProjectLanguage::Python => sayiir_openflow::extract::python::extract_all_python_tasks(&source)?,
            ProjectLanguage::Node => sayiir_openflow::extract::node::extract_all_node_tasks(&source)?,
        };
        
        for task in tasks {
            task_registry.insert(task);
        }
    }
    
    println!("✓ Found {} tasks", task_registry.len());
    
    // 4. Parse workflow
    let workflow_source = std::fs::read_to_string(&project_scan.workflow_file).map_err(|e| {
        error::ExportError::FileReadError {
            path: project_scan.workflow_file.clone(),
            source: e,
        }
    })?;
    
    let (workflow_name, task_names) = match language {
        ProjectLanguage::Rust => {
            let wf = parse::rust::parse_rust_workflow(&workflow_source)?;
            (wf.name, wf.task_names)
        }
        ProjectLanguage::Python => {
            let wf = sayiir_openflow::parse::python::parse_python_workflow(&workflow_source)?;
            (wf.name, wf.task_names)
        }
        ProjectLanguage::Node => {
            let wf = sayiir_openflow::parse::node::parse_node_workflow(&workflow_source)?;
            (wf.name, wf.task_names)
        }
    };
    
    println!("✓ Found workflow: {}", workflow_name);
    
    // 5. Parse dependencies
    let deps = match language {
        ProjectLanguage::Rust => parse_cargo_deps(&project_dir).unwrap_or_else(|e| {
            eprintln!("⚠ Failed to parse dependencies: {}", e);
            eprintln!("  → Exporting without dependency info");
            HashMap::new()
        }),
        ProjectLanguage::Python => parse_python_deps(&project_dir).unwrap_or_else(|e| {
            eprintln!("⚠ Failed to parse dependencies: {}", e);
            eprintln!("  → Exporting without dependency info");
            HashMap::new()
        }),
        ProjectLanguage::Node => parse_node_deps(&project_dir).unwrap_or_else(|e| {
            eprintln!("⚠ Failed to parse dependencies: {}", e);
            eprintln!("  → Exporting without dependency info");
            HashMap::new()
        }),
    };
    
    // 6. Match tasks to workflow
    let mut matched_tasks = Vec::new();
    for task_name in &task_names {
        if let Some(task_src) = task_registry.get(task_name) {
            println!("✓ Extracted {} ({} lines)", task_src.id, task_src.source_code.lines().count());
            
            matched_tasks.push(TaskMetadata {
                id: task_src.id.clone(),
                language: lang_enum,
                source_code: task_src.source_code.clone(),
                entry_point: task_src.entry_point.clone(),
                dependencies: deps.clone(),
            });
        } else {
            eprintln!("⚠ Task '{}' not found: Task '{}' not found", task_name, task_name);
            eprintln!("  → Check task definition exists");
            eprintln!("  → Exporting task name only (workflow not portable)");
        }
    }
    
    // 7. Build OpenFlowSpec
    let spec = build_openflow_spec(workflow_name, matched_tasks);
    
    // 8. Export (existing code continues...)
    if matches!(format, Format::Json | Format::Both) {
        let json = serde_json::to_string_pretty(&spec)
            .map_err(|e| error::ExportError::InvalidWorkflowSyntax(format!("Failed to serialize JSON: {}", e)))?;
        if !dry_run {
            std::fs::write(&output, &json).map_err(|e| error::ExportError::FileWriteError {
                path: output.clone(),
                source: e,
            })?;
            let size = json.len() as f64 / 1024.0;
            println!("✓ Generated {} ({:.1} KB)", output.display(), size);
        } else {
            println!("✓ Would generate {} ({:.1} KB)", output.display(), json.len() as f64 / 1024.0);
        }
    }

    if matches!(format, Format::Mermaid | Format::Both) {
        let mermaid = generate_mermaid_stub(&spec);
        if !dry_run {
            std::fs::write(&mermaid_output, &mermaid).map_err(|e| {
                error::ExportError::FileWriteError {
                    path: mermaid_output.clone(),
                    source: e,
                }
            })?;
            let size = mermaid.len() as f64 / 1024.0;
            println!("✓ Generated {} ({:.1} KB)", mermaid_output.display(), size);
        } else {
            println!("✓ Would generate {} ({:.1} KB)", mermaid_output.display(), mermaid.len() as f64 / 1024.0);
        }
    }

    Ok(())
}
```

- [ ] **Step 3: Test on video-pipeline-rs**

```bash
cd /home/wj/work/sayiir/examples/video-pipeline-rs
cargo-sayiir-openflow sayiir-openflow export --dry-run
```

Expected: "✓ Found 11 tasks" with successful extraction messages

- [ ] **Step 4: Test on approval-workflow-py**

```bash
cd /home/wj/work/sayiir/examples/approval-workflow-py
cargo-sayiir-openflow sayiir-openflow export --dry-run
```

Expected: "✓ Detected Python project" with successful extraction

- [ ] **Step 5: Test on rag-agent-cf**

```bash
cd /home/wj/work/sayiir/examples/rag-agent-cf
cargo-sayiir-openflow sayiir-openflow export --dry-run
```

Expected: "✓ Detected Node project" with successful extraction

- [ ] **Step 6: Commit**

```bash
git add src/bin/cargo-sayiir-openflow.rs
git commit -m "feat(cli): implement multi-file extraction pipeline for all languages"
```

---

## Task 10: Integration Tests

**Files:**
- Create: `/home/wj/work/sayiir/sayiir-openflow/tests/integration_test.rs`

**Interfaces:**
- Consumes: All components
- Produces: End-to-end test coverage

- [ ] **Step 1: Create integration test file**

Create `/home/wj/work/sayiir/sayiir-openflow/tests/integration_test.rs`:

```rust
use sayiir_openflow::*;
use std::path::PathBuf;

#[test]
fn test_video_pipeline_rust_extraction() {
    let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../examples/video-pipeline-rs");
    
    // Scan
    let scan = scan::scan_project(&project_dir, ProjectLanguage::Rust).unwrap();
    assert!(scan.workflow_file.ends_with("main.rs"));
    assert!(scan.task_files.len() >= 3);
    
    // Extract tasks
    let mut registry = extract::TaskRegistry::new();
    for file in &scan.task_files {
        let source = std::fs::read_to_string(file).unwrap();
        let tasks = extract::rust::extract_all_rust_tasks(&source).unwrap();
        for task in tasks {
            registry.insert(task);
        }
    }
    
    assert!(registry.len() >= 10);
    assert!(registry.get("download_video").is_some());
    assert!(registry.get("transcode_720p").is_some());
    
    // Parse workflow
    let wf_source = std::fs::read_to_string(&scan.workflow_file).unwrap();
    let wf = parse::rust::parse_rust_workflow(&wf_source).unwrap();
    
    assert_eq!(wf.name, "video-pipeline");
    assert!(wf.task_names.contains(&"download_video".to_string()));
}

#[test]
fn test_approval_workflow_python_extraction() {
    let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../examples/approval-workflow-py");
    
    if !project_dir.exists() {
        println!("Skipping: approval-workflow-py not found");
        return;
    }
    
    // Scan
    let scan = scan::scan_project(&project_dir, ProjectLanguage::Python).unwrap();
    assert!(scan.workflow_file.ends_with("main.py"));
    
    // Extract tasks
    let mut registry = extract::TaskRegistry::new();
    for file in &scan.task_files {
        let source = std::fs::read_to_string(file).unwrap();
        let tasks = extract::python::extract_all_python_tasks(&source).unwrap();
        for task in tasks {
            registry.insert(task);
        }
    }
    
    assert!(registry.len() >= 2);
    assert!(registry.get("submit_expense").is_some());
    
    // Parse workflow
    let wf_source = std::fs::read_to_string(&scan.workflow_file).unwrap();
    let wf = parse::python::parse_python_workflow(&wf_source).unwrap();
    
    assert_eq!(wf.name, "expense-approval");
}

#[test]
fn test_rag_agent_node_extraction() {
    let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../examples/rag-agent-cf");
    
    if !project_dir.exists() {
        println!("Skipping: rag-agent-cf not found");
        return;
    }
    
    // Scan
    let scan = scan::scan_project(&project_dir, ProjectLanguage::Node).unwrap();
    assert!(scan.task_files.len() >= 3);
    
    // Extract tasks
    let mut registry = extract::TaskRegistry::new();
    for file in &scan.task_files {
        let source = std::fs::read_to_string(file).unwrap();
        let tasks = extract::node::extract_all_node_tasks(&source).unwrap();
        for task in tasks {
            registry.insert(task);
        }
    }
    
    assert!(registry.len() >= 2);
}
```

- [ ] **Step 2: Run integration tests**

```bash
cd /home/wj/work/sayiir/sayiir-openflow
cargo test integration_test
```

Expected: All tests PASS

- [ ] **Step 3: Commit**

```bash
git add tests/integration_test.rs
git commit -m "test: add integration tests for multi-language extraction"
```

---

## Success Criteria Verification

- [ ] **Verify video-pipeline-rs extraction**

```bash
cd /home/wj/work/sayiir/examples/video-pipeline-rs
cargo-sayiir-openflow sayiir-openflow export
cat workflow.json | grep -c '"id"'
```

Expected: Count >= 10 (all tasks extracted)

- [ ] **Verify approval-workflow-py extraction**

```bash
cd /home/wj/work/sayiir/examples/approval-workflow-py
cargo-sayiir-openflow sayiir-openflow export
cat workflow.json | grep '"submit_expense"'
```

Expected: Found in JSON

- [ ] **Verify rag-agent-cf extraction**

```bash
cd /home/wj/work/sayiir/examples/rag-agent-cf
cargo-sayiir-openflow sayiir-openflow export
cat workflow.json
```

Expected: Valid JSON with workflow data

- [ ] **Verify only one CLI binary exists**

```bash
ls -la /home/wj/work/sayiir/sayiir-openflow/src/bin/
```

Expected: Only `cargo-sayiir-openflow.rs` present

---

## Final Commit

- [ ] **Run all tests**

```bash
cd /home/wj/work/sayiir/sayiir-openflow
cargo test
```

Expected: All tests PASS

- [ ] **Final commit**

```bash
git add .
git commit -m "feat: complete multi-file source extraction for Rust/Python/Node

- Multi-file scanner with language-specific glob patterns
- Task registry for centralized task storage
- Enhanced Rust extraction with extract_all_rust_tasks
- Python @task extraction and Flow() parsing
- Node.js task() extraction and flow() parsing
- Updated CLI with full multi-file pipeline
- Integration tests for all three languages
- Removed old sayiir-openflow binary

Closes sayiir-openflow source extraction spec"
```
