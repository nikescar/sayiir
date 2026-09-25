use sayiir_openflow::*;
use std::time::Duration;

#[tokio::test]
async fn test_execution_timeout() {
    let module = OpenFlowModule {
        id: "slow_task".to_string(),
        value: OpenFlowModuleValue::Script {
            path: "slow_task".to_string(),
            language: Some("python".to_string()),
            entry_point: Some("run".to_string()),
            code: Some(r#"
import time

def run(input_data):
    time.sleep(5)  # Sleep longer than timeout
    return {"result": "ok"}
"#.to_string()),
        },
    };

    let cached = compile_module(&module, "test_workflow").await.unwrap();

    let input = serde_json::json!({});
    let result = execute_task_with_timeout(&cached, "python", input, Duration::from_secs(1)).await;

    assert!(result.is_err());
    if let Err(OpenFlowError::Timeout { seconds }) = result {
        assert_eq!(seconds, 1);
    } else {
        panic!("Expected Timeout error");
    }

    // Cleanup
    std::fs::remove_dir_all(cached.cache_path).ok();
}
