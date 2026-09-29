use crate::error::{ExportError, ExportResult as Result};
use std::collections::HashMap;
use std::path::Path;

pub fn parse_cargo_deps(dir: &Path) -> Result<HashMap<String, String>> {
    let path = dir.join("Cargo.toml");
    let content = std::fs::read_to_string(&path).map_err(|e| ExportError::FileReadError {
        path: path.clone(),
        source: e,
    })?;

    let toml: toml::Value = toml::from_str(&content)
        .map_err(|e| ExportError::InvalidWorkflowSyntax(format!("Invalid Cargo.toml: {}", e)))?;

    let mut result = HashMap::new();

    if let Some(deps) = toml.get("dependencies").and_then(|v| v.as_table()) {
        for (name, value) in deps {
            let version = match value {
                toml::Value::String(v) => v.clone(),
                toml::Value::Table(t) => {
                    t.get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("*")
                        .to_string()
                }
                _ => continue,
            };
            result.insert(name.clone(), version);
        }
    }

    Ok(result)
}

pub fn parse_python_deps(dir: &Path) -> Result<HashMap<String, String>> {
    let path = dir.join("requirements.txt");
    let content = std::fs::read_to_string(&path).map_err(|e| ExportError::FileReadError {
        path: path.clone(),
        source: e,
    })?;

    let mut result = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse "package==version" or "package>=version" or "package~=version"
        for separator in &["==", ">=", "~=", "<=", ">", "<"] {
            if let Some(pos) = line.find(separator) {
                let package = line[..pos].trim();
                let version = line[pos + separator.len()..].trim();
                result.insert(package.to_string(), version.to_string());
                break;
            }
        }
    }

    Ok(result)
}

pub fn parse_node_deps(dir: &Path) -> Result<HashMap<String, String>> {
    let path = dir.join("package.json");
    let content = std::fs::read_to_string(&path).map_err(|e| ExportError::FileReadError {
        path: path.clone(),
        source: e,
    })?;

    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| ExportError::InvalidWorkflowSyntax(format!("Invalid package.json: {}", e)))?;

    let mut result = HashMap::new();

    if let Some(deps) = json.get("dependencies").and_then(|v| v.as_object()) {
        for (name, value) in deps {
            if let Some(version) = value.as_str() {
                result.insert(name.clone(), version.to_string());
            }
        }
    }

    Ok(result)
}
