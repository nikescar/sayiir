use clap::Parser;
use sayiir_openflow::{export_mermaid, export_openflow_json, OpenFlowSpec};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sayiir-openflow")]
#[command(about = "Convert workflows to OpenFlow format", long_about = None)]
struct Cli {
    /// Input directory containing workflow code
    #[arg(long, value_name = "DIR")]
    input_dir: Option<PathBuf>,

    /// Input JSON file to convert
    #[arg(long, value_name = "FILE", conflicts_with = "input_dir")]
    input_json: Option<PathBuf>,

    /// Input Mermaid file to convert
    #[arg(long, value_name = "FILE", conflicts_with_all = ["input_dir", "input_json"])]
    input_mermaid: Option<PathBuf>,

    /// Output type: openflow, mermaidmd, or both
    #[arg(long, default_value = "both")]
    output_type: String,

    /// Output file (for single output type)
    #[arg(long, value_name = "FILE")]
    output_file: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let spec: OpenFlowSpec = if let Some(json_path) = cli.input_json {
        // Import from JSON
        let json = std::fs::read_to_string(json_path)?;
        sayiir_openflow::import_openflow_json(&json)?
    } else if let Some(mermaid_path) = cli.input_mermaid {
        // Import from Mermaid
        let mermaid = std::fs::read_to_string(mermaid_path)?;
        sayiir_openflow::import_mermaid(&mermaid)?
    } else if let Some(_input_dir) = cli.input_dir {
        // Extract from source directory
        eprintln!("⚠ Source code extraction not yet implemented");
        eprintln!("Please use --input-json or --input-mermaid for now");
        eprintln!();
        eprintln!("To convert existing workflows:");
        eprintln!("  1. Use the builder API in your code:");
        eprintln!("     use sayiir_openflow::builder::WorkflowBuilder;");
        eprintln!();
        eprintln!("  2. Or manually create OpenFlowSpec and export");
        eprintln!();
        eprintln!("See examples/builder_example.rs for usage");
        std::process::exit(1);
    } else {
        eprintln!("Error: Must specify --input-dir, --input-json, or --input-mermaid");
        std::process::exit(1);
    };

    // Export based on output type
    match cli.output_type.as_str() {
        "openflow" | "json" => {
            let json = export_openflow_json(&spec)?;
            let output = cli.output_file.unwrap_or_else(|| "workflow.json".into());
            std::fs::write(&output, json)?;
            println!("✓ Exported OpenFlow JSON to {}", output.display());
        }
        "mermaidmd" | "mermaid" => {
            let mermaid = export_mermaid(&spec)?;
            let output = cli.output_file.unwrap_or_else(|| "workflow.mmd".into());
            std::fs::write(&output, mermaid)?;
            println!("✓ Exported Mermaid to {}", output.display());
        }
        "both" => {
            let json = export_openflow_json(&spec)?;
            std::fs::write("workflow.json", json)?;
            println!("✓ Exported OpenFlow JSON to workflow.json");

            let mermaid = export_mermaid(&spec)?;
            std::fs::write("workflow.mmd", mermaid)?;
            println!("✓ Exported Mermaid to workflow.mmd");
        }
        _ => {
            eprintln!("Error: Invalid output type '{}'. Use: openflow, mermaidmd, or both", cli.output_type);
            std::process::exit(1);
        }
    }

    Ok(())
}
