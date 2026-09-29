use sayiir_openflow::extract::rust::extract_rust_task;
use sayiir_openflow::TaskSource;

#[test]
fn test_extract_simple_task() {
    let source = r#"
use sayiir_runtime::prelude::*;

#[task(id = "greet")]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[task(id = "other_task")]
fn other() {}
    "#;

    let result = extract_rust_task(source, "greet").unwrap();
    assert_eq!(result.id, "greet");
    assert_eq!(result.entry_point, "greet");
    assert!(result.source_code.contains("format"));
    assert!(result.source_code.contains("task"));
    assert!(result.source_code.contains("greet"));
}

#[test]
fn test_extract_async_task() {
    let source = r#"
#[task(id = "download", retries = 2, timeout = "5m")]
pub async fn download(url: String) -> Result<Vec<u8>, BoxError> {
    let bytes = reqwest::get(&url).await?.bytes().await?;
    Ok(bytes.to_vec())
}
    "#;

    let result = extract_rust_task(source, "download").unwrap();
    assert_eq!(result.id, "download");
    assert_eq!(result.entry_point, "download");
    assert!(result.source_code.contains("async fn"));
    assert!(result.source_code.contains("reqwest"));
}

#[test]
fn test_task_not_found() {
    let source = r#"
#[task(id = "foo")]
fn foo() {}
    "#;

    let result = extract_rust_task(source, "bar");
    assert!(result.is_err());
}
