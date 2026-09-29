//! Export workflow to GitHub-friendly markdown
use sayiir_openflow::*;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <json_file>", args[0]);
        std::process::exit(1);
    }
    
    let json = std::fs::read_to_string(&args[1])?;
    let spec = import_openflow_json(&json)?;
    let markdown = export_mermaid(&spec)?;
    println!("{}", markdown);
    Ok(())
}
