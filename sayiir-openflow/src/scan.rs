use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::{ProjectLanguage, error::{ExportResult, ExportError}};

pub struct ProjectScan {
    pub workflow_file: PathBuf,
    pub task_files: Vec<PathBuf>,
}

pub fn scan_project(project_dir: &Path, language: ProjectLanguage) -> ExportResult<ProjectScan> {
    match language {
        ProjectLanguage::Rust => scan_rust_project(project_dir),
        ProjectLanguage::Python => scan_python_project(project_dir),
        ProjectLanguage::Node => scan_node_project(project_dir),
    }
}

fn scan_rust_project(project_dir: &Path) -> ExportResult<ProjectScan> {
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

fn scan_python_project(project_dir: &Path) -> ExportResult<ProjectScan> {
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

fn scan_node_project(project_dir: &Path) -> ExportResult<ProjectScan> {
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

    // Workflow file priority: src/workflow*.ts, src/index.ts, index.ts, src/main.ts, main.ts, first file
    let workflow_file = node_files
        .iter()
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.starts_with("workflow") && (s.ends_with(".ts") || s.ends_with(".js")))
                .unwrap_or(false)
        })
        .cloned()
        .or_else(|| {
            [
                src_dir.join("index.ts"),
                project_dir.join("index.ts"),
                src_dir.join("main.ts"),
                project_dir.join("main.ts"),
            ]
            .iter()
            .find(|p| p.exists())
            .cloned()
        })
        .or_else(|| node_files.first().cloned())
        .ok_or_else(|| ExportError::NoWorkflowFound)?;

    Ok(ProjectScan {
        workflow_file,
        task_files: node_files,
    })
}
