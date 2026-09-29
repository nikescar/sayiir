use sayiir_openflow::*;

fn main() -> Result<()> {
    let json = std::fs::read_to_string("examples/hello-world-node/hello-world-pure.json")?;
    let spec = import_openflow_json(&json)?;
    let markdown = export_mermaid(&spec)?;
    println!("{}", markdown);
    Ok(())
}
