use crate::error::{ExportError, ExportResult as Result};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectLanguage {
    Rust,
    Python,
    Node,
}

pub fn detect_language(dir: &Path) -> Result<ProjectLanguage> {
    // Priority: Cargo.toml > package.json > requirements.txt
    if dir.join("Cargo.toml").exists() {
        return Ok(ProjectLanguage::Rust);
    }
    if dir.join("package.json").exists() {
        return Ok(ProjectLanguage::Node);
    }
    if dir.join("requirements.txt").exists() || dir.join("pyproject.toml").exists() {
        return Ok(ProjectLanguage::Python);
    }

    Err(ExportError::NoProjectDetected)
}
