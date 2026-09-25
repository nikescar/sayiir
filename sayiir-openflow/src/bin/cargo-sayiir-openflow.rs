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
    }

    Ok(())
}

fn export_workflow(
    input: Option<PathBuf>,
    format: Format,
    output: PathBuf,
    mermaid_output: PathBuf,
    dry_run: bool,
) -> error::ExportResult<()> {
    // 1. Detect language
    let project_dir = std::env::current_dir().map_err(|e| error::ExportError::FileReadError {
        path: PathBuf::from("."),
        source: e,
    })?;

    let language = detect_language(&project_dir)?;
    println!("✓ Detected {:?} project", language);

    // 2. Determine input file
    let workflow_file = input.unwrap_or_else(|| match language {
        ProjectLanguage::Rust => PathBuf::from("src/main.rs"),
        ProjectLanguage::Python => PathBuf::from("main.py"),
        ProjectLanguage::Node => PathBuf::from("index.ts"),
    });

    // 3. Parse workflow (Rust only for now)
    let source = std::fs::read_to_string(&workflow_file).map_err(|e| {
        error::ExportError::FileReadError {
            path: workflow_file.clone(),
            source: e,
        }
    })?;

    let (workflow_name, task_names) = match language {
        ProjectLanguage::Rust => {
            let workflow = parse::rust::parse_rust_workflow(&source)?;
            (workflow.name, workflow.task_names)
        }
        _ => {
            eprintln!("⚠ Python and Node.js support not yet implemented");
            return Err(error::ExportError::NoWorkflowFound);
        }
    };

    println!("✓ Found workflow: {}", workflow_name);

    // 4. Extract tasks
    let mut tasks = Vec::new();
    for task_name in &task_names {
        match extract::rust::extract_rust_task(&source, task_name) {
            Ok(task_src) => {
                let line_count = task_src.source_code.lines().count();
                println!("✓ Extracted {} ({} lines)", task_src.id, line_count);

                tasks.push(TaskMetadata {
                    id: task_src.id,
                    language: Language::Rust,
                    source_code: task_src.source_code,
                    entry_point: task_src.entry_point,
                    dependencies: HashMap::new(), // Will add deps next
                });
            }
            Err(e) => {
                eprintln!("⚠ Task '{}' not found: {}", task_name, e);
                eprintln!("  → Exporting task name only (workflow not portable)");
            }
        }
    }

    // 5. Parse dependencies
    let deps = parse_cargo_deps(&project_dir).unwrap_or_else(|e| {
        eprintln!("⚠ Failed to parse dependencies: {}", e);
        eprintln!("  → Exporting without dependency info");
        HashMap::new()
    });

    // Merge deps into all tasks
    for task in &mut tasks {
        task.dependencies = deps.clone();
    }

    // 6. Build OpenFlowSpec
    let spec = build_openflow_spec(workflow_name, tasks);

    // 7. Export
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
        // TODO: Implement Mermaid export in Task 8
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
    mermaid.push_str(&format!("    Task{} --> End[Done]\n", spec.value.modules.len() - 1));
    mermaid
}
