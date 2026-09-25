use sayiir_openflow::deps::{parse_cargo_deps, parse_python_deps, parse_node_deps};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_parse_cargo_deps() {
    let dir = TempDir::new().unwrap();
    let cargo_toml = r#"
[package]
name = "test"

[dependencies]
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }
tokio = "1.0"
    "#;
    fs::write(dir.path().join("Cargo.toml"), cargo_toml).unwrap();

    let deps = parse_cargo_deps(dir.path()).unwrap();
    assert_eq!(deps.get("serde_json"), Some(&"1.0".to_string()));
    assert_eq!(deps.get("reqwest"), Some(&"0.11".to_string()));
    assert_eq!(deps.get("tokio"), Some(&"1.0".to_string()));
}

#[test]
fn test_parse_python_deps() {
    let dir = TempDir::new().unwrap();
    let requirements = "requests==2.28.0\nnumpy>=1.23.0\npandas~=1.5.0";
    fs::write(dir.path().join("requirements.txt"), requirements).unwrap();

    let deps = parse_python_deps(dir.path()).unwrap();
    assert_eq!(deps.get("requests"), Some(&"2.28.0".to_string()));
    assert_eq!(deps.get("numpy"), Some(&"1.23.0".to_string()));
    assert_eq!(deps.get("pandas"), Some(&"1.5.0".to_string()));
}

#[test]
fn test_parse_node_deps() {
    let dir = TempDir::new().unwrap();
    let package_json = r#"{
  "name": "test",
  "dependencies": {
    "axios": "^1.0.0",
    "lodash": "~4.17.0"
  }
}"#;
    fs::write(dir.path().join("package.json"), package_json).unwrap();

    let deps = parse_node_deps(dir.path()).unwrap();
    assert_eq!(deps.get("axios"), Some(&"^1.0.0".to_string()));
    assert_eq!(deps.get("lodash"), Some(&"~4.17.0".to_string()));
}
