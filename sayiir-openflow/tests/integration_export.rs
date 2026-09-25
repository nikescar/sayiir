use std::process::Command;
use std::path::Path;

#[test]
fn test_export_rust_example() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rust-example");

    // Run CLI
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-sayiir-openflow"))
        .args(["sayiir-openflow", "export", "--dry-run"])
        .current_dir(&fixture_dir)
        .output()
        .expect("Failed to run CLI");

    // Verify exit code
    assert!(
        output.status.success(),
        "CLI failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✓ Detected Rust project"));
    assert!(stdout.contains("✓ Found workflow: greeting-workflow"));
    assert!(stdout.contains("✓ Extracted greet"));
    assert!(stdout.contains("✓ Extracted shout"));
}

#[test]
fn test_export_rust_example_files() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rust-example");

    // Run CLI (without dry-run)
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-sayiir-openflow"))
        .args(["sayiir-openflow", "export"])
        .current_dir(&fixture_dir)
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());

    // Verify files were created
    assert!(fixture_dir.join("workflow.json").exists());
    assert!(fixture_dir.join("workflow.md").exists());

    // Verify JSON structure
    let json = std::fs::read_to_string(fixture_dir.join("workflow.json")).unwrap();
    assert!(json.contains("\"summary\": \"greeting-workflow\""));
    assert!(json.contains("\"id\": \"greet\""));
    assert!(json.contains("\"id\": \"shout\""));
    assert!(json.contains("\"language\": \"rust\""));

    // Verify Mermaid structure
    let mermaid = std::fs::read_to_string(fixture_dir.join("workflow.md")).unwrap();
    assert!(mermaid.contains("flowchart"));
    assert!(mermaid.contains("greet"));
    assert!(mermaid.contains("shout"));

    // Cleanup
    std::fs::remove_file(fixture_dir.join("workflow.json")).ok();
    std::fs::remove_file(fixture_dir.join("workflow.md")).ok();
}
