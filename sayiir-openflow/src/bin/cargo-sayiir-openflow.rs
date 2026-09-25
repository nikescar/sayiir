use clap::Parser;
use sayiir_openflow::*;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cargo-sayiir-openflow")]
#[command(bin_name = "cargo-sayiir-openflow")]
enum Cli {
    #[command(name = "sayiir-openflow")]
    SayiirOpenflow(Args),
}

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    /// Export workflow to OpenFlow JSON and Mermaid markdown
    Export {
        /// Input file (auto-detect if not specified)
        #[arg(value_name = "FILE")]
        input: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "both")]
        format: Format,

        /// JSON output path
        #[arg(short, long, default_value = "workflow.json")]
        output: PathBuf,

        /// Mermaid output path
        #[arg(short, long, default_value = "workflow.md")]
        mermaid_output: PathBuf,

        /// Dry run (don't write files)
        #[arg(long)]
        dry_run: bool,
    },

    /// Import OpenFlow workflow and generate standalone project
    Import {
        /// Input OpenFlow JSON or Mermaid file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output directory for generated project
        #[arg(short, long, default_value = "workflow")]
        output_dir: PathBuf,
    },

    /// Run OpenFlow workflow directly
    Run {
        /// Input OpenFlow JSON or Mermaid file
        #[arg(value_name = "FILE")]
        workflow: PathBuf,

        /// JSON input for workflow
        #[arg(short, long)]
        input: Option<String>,

        /// Timeout in seconds per task
        #[arg(short, long, default_value = "30")]
        timeout: u64,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum Format {
    Json,
    Mermaid,
    Both,
}

fn main() -> error::ExportResult<()> {
    let Cli::SayiirOpenflow(args) = Cli::parse();

    match args.command {
        Command::Export {
            input,
            format,
            output,
            mermaid_output,
            dry_run,
        } => {
            export_workflow(input, format, output, mermaid_output, dry_run)?;
        }
        Command::Import { input, output_dir } => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(import_workflow_cmd(input, output_dir))?;
        }
        Command::Run {
            workflow,
            input,
            timeout,
        } => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(run_workflow_cmd(workflow, input, timeout))?;
        }
    }

    Ok(())
}

fn export_workflow(
    _input: Option<PathBuf>,
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

    // 6. Match tasks to workflow and clean with Tree-sitter
    let mut matched_tasks = Vec::new();
    for task_name in &task_names {
        if let Some(task_src) = task_registry.get_by_name(task_name) {
            // Clean code using Tree-sitter (strip decorators/attributes/wrappers)
            let clean_code = match language {
                ProjectLanguage::Rust => {
                    sayiir_openflow::extract_clean::extract_rust_function(&task_src.source_code, &task_src.entry_point)
                        .unwrap_or_else(|| task_src.source_code.clone())
                }
                ProjectLanguage::Python => {
                    sayiir_openflow::extract_clean::extract_python_function(&task_src.source_code, &task_src.entry_point)
                        .unwrap_or_else(|| task_src.source_code.clone())
                }
                ProjectLanguage::Node => {
                    sayiir_openflow::extract_clean::extract_javascript_function(&task_src.source_code, &task_src.entry_point)
                        .unwrap_or_else(|| task_src.source_code.clone())
                }
            };

            println!("✓ Extracted {} ({} lines → {} lines clean)",
                task_src.id,
                task_src.source_code.lines().count(),
                clean_code.lines().count()
            );

            matched_tasks.push(TaskMetadata {
                id: task_src.id.clone(),
                language: lang_enum,
                source_code: clean_code,
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

    // 8. Export
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

fn generate_mermaid_stub(spec: &OpenFlowSpec) -> String {
    // Stub implementation for Mermaid export
    // TODO: Implement proper Mermaid flowchart generation in Task 8
    let mut mermaid = String::from("flowchart TD\n");
    mermaid.push_str(&format!("    Start[{}]\n", spec.summary));
    for (i, module) in spec.value.modules.iter().enumerate() {
        mermaid.push_str(&format!("    Task{}[{}]\n", i, module.id));
        if i == 0 {
            mermaid.push_str(&format!("    Start --> Task{}\n", i));
        } else {
            mermaid.push_str(&format!("    Task{} --> Task{}\n", i - 1, i));
        }
    }
    if !spec.value.modules.is_empty() {
        mermaid.push_str(&format!("    Task{} --> End[Done]\n", spec.value.modules.len() - 1));
    } else {
        mermaid.push_str("    Start --> End[Done]\n");
    }
    mermaid
}

async fn import_workflow_cmd(input: PathBuf, output_dir: PathBuf) -> error::ExportResult<()> {
    // Read input file
    let content = std::fs::read_to_string(&input).map_err(|e| error::ExportError::FileReadError {
        path: input.clone(),
        source: e,
    })?;

    // Parse spec (auto-detect JSON vs Mermaid)
    let spec = if input.extension().and_then(|s| s.to_str()) == Some("md") {
        import_mermaid(&content)?
    } else {
        import_openflow_json(&content)?
    };

    println!("✓ Parsed workflow: {}", spec.summary);
    println!("  Modules: {}", spec.value.modules.len());

    // Import workflow
    import_workflow(&spec, &output_dir).await?;

    println!("✓ Generated project in {}", output_dir.display());
    Ok(())
}

async fn run_workflow_cmd(
    workflow: PathBuf,
    input: Option<String>,
    timeout: u64,
) -> error::ExportResult<()> {
    use std::time::Duration;

    // Read workflow file
    let content = std::fs::read_to_string(&workflow).map_err(|e| error::ExportError::FileReadError {
        path: workflow.clone(),
        source: e,
    })?;

    // Parse spec (auto-detect JSON vs Mermaid)
    let spec = if workflow.extension().and_then(|s| s.to_str()) == Some("md") {
        import_mermaid(&content)?
    } else {
        import_openflow_json(&content)?
    };

    println!("✓ Loaded workflow: {}", spec.summary);

    // Parse input JSON
    let input_value = if let Some(json_str) = input {
        serde_json::from_str(&json_str).map_err(|e| {
            error::ExportError::InvalidWorkflowSyntax(format!("Invalid input JSON: {}", e))
        })?
    } else {
        serde_json::json!({})
    };

    // Run workflow
    println!("→ Running workflow...");
    let result = run_workflow_with_timeout(&spec, input_value, Duration::from_secs(timeout)).await?;

    // Print result
    println!("\n✓ Workflow completed:");
    println!("{}", serde_json::to_string_pretty(&result).unwrap());

    Ok(())
}
