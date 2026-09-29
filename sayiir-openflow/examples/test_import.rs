use sayiir_openflow::*;

fn main() -> Result<()> {
    let markdown = std::fs::read_to_string("/tmp/test-gh.md")?;
    let spec = import_mermaid(&markdown)?;
    let json = serde_json::to_string_pretty(&spec)?;
    println!("{}", json);
    Ok(())
}
