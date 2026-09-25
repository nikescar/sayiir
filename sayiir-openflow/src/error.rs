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
        language: String,
        install_url: String,
    },

    /// Compilation errors
    #[error("Compilation failed for {module_id}:\n{stderr}")]
    CompilationError {
        module_id: String,
        stderr: String,
    },

    /// Execution errors
    #[error("Task execution failed: {0}")]
    ExecutionError(String),

    /// Timeout
    #[error("Task timeout after {seconds}s")]
    Timeout { seconds: u64 },

    /// IO errors
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for OpenFlow operations
pub type Result<T> = std::result::Result<T, OpenFlowError>;
