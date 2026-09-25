use sayiir_openflow::detect::{detect_language, ProjectLanguage};
use sayiir_openflow::error::ExportError;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_detect_rust_project() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();

    let result = detect_language(dir.path()).unwrap();
    assert!(matches!(result, ProjectLanguage::Rust));
}

#[test]
fn test_detect_python_project() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("requirements.txt"), "sayiir==0.1").unwrap();

    let result = detect_language(dir.path()).unwrap();
    assert!(matches!(result, ProjectLanguage::Python));
}

#[test]
fn test_detect_node_project() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("package.json"), r#"{"name":"test"}"#).unwrap();

    let result = detect_language(dir.path()).unwrap();
    assert!(matches!(result, ProjectLanguage::Node));
}

#[test]
fn test_no_project_detected() {
    let dir = TempDir::new().unwrap();

    let result = detect_language(dir.path());
    assert!(matches!(result, Err(ExportError::NoProjectDetected)));
}

#[test]
fn test_rust_priority_over_others() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
    fs::write(dir.path().join("package.json"), "{}").unwrap();

    let result = detect_language(dir.path()).unwrap();
    assert!(matches!(result, ProjectLanguage::Rust));
}
