use thiserror::Error;

/// OpenFlow import/export errors
#[derive(Error, Debug)]
pub enum OpenFlowError {
    /// JSON parsing error
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Invalid workflow structure
    #[error("Invalid workflow: {0}")]
    InvalidWorkflow(String),

    /// Unsupported feature
    #[error("Unsupported: {0}")]
    Unsupported(String),

    /// Missing runtime
    #[error("Runtime not found: {language}. Install from: {install_url}")]
    MissingRuntime {
        /// Language runtime that is missing
        language: String,
        /// URL to installation instructions
        install_url: String,
    },

    /// Compilation errors
    #[error("Compilation failed for {module_id}:\n{stderr}")]
    CompilationError {
        /// Module ID
        module_id: String,
        /// Compiler error output
        stderr: String,
    },

    /// Dependency installation errors
    #[error("Dependency installation failed for {module_id} ({language}): {dependency}\n{stderr}")]
    DependencyError {
        /// Module ID
        module_id: String,
        /// Language
        language: String,
        /// Dependency name
        dependency: String,
        /// Error output
        stderr: String,
    },

    /// Execution errors
    #[error("Task execution failed: {0}")]
    ExecutionError(String),

    /// Timeout
    #[error("Task timeout after {seconds}s")]
    Timeout {
        /// Timeout duration in seconds
        seconds: u64,
    },

    /// IO errors
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for OpenFlow operations
pub type Result<T> = std::result::Result<T, OpenFlowError>;

// ─── Export-specific errors ─────────────────────────────────────────────────

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("No workflow project detected\n → Expected Cargo.toml, package.json, or requirements.txt")]
    NoProjectDetected,

    #[error("Task '{task_id}' not found\n → Check task definition exists")]
    TaskNotFound { task_id: String },

    #[error("No workflow definition found\n → Expected workflow! macro (Rust), Flow().build() (Python/Node)")]
    NoWorkflowFound,

    #[error("Invalid workflow syntax: {0}")]
    InvalidWorkflowSyntax(String),

    #[error("Failed to read file {path}: {source}")]
    FileReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write file {path}: {source}")]
    FileWriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("OpenFlow error: {0}")]
    OpenFlowError(#[from] OpenFlowError),
}

pub type ExportResult<T> = std::result::Result<T, ExportError>;
