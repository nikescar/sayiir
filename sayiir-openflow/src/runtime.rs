use crate::{OpenFlowError, OpenFlowModuleValue, OpenFlowSpec, Result};
use std::collections::HashSet;

/// Check if required language runtimes are installed
pub fn check_runtimes(spec: &OpenFlowSpec) -> Result<()> {
    let mut required_runtimes = HashSet::new();

    for module in &spec.value.modules {
        if let OpenFlowModuleValue::Script {
            language: Some(lang),
            ..
        } = &module.value
        {
            required_runtimes.insert(lang.as_str());
        }
    }

    for runtime in required_runtimes {
        check_runtime_installed(runtime)?;
    }

    Ok(())
}

fn check_runtime_installed(language: &str) -> Result<()> {
    let (command, install_url) = match language {
        "rust" => ("cargo", "https://rustup.rs"),
        "node" => ("node", "https://nodejs.org"),
        "python" => ("python3", "https://www.python.org"),
        _ => {
            return Err(OpenFlowError::Unsupported(format!(
                "Unknown language: {}",
                language
            )));
        }
    };

    if which::which(command).is_err() {
        return Err(OpenFlowError::MissingRuntime {
            language: language.to_string(),
            install_url: install_url.to_string(),
        });
    }

    Ok(())
}
