use sayiir_openflow::*;
use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_run_workflow_timeout() {
    let spec = OpenFlowSpec {
        summary: "Slow workflow".to_string(),
        value: OpenFlowValue {
            modules: vec![OpenFlowModule {
                id: "slow_task".to_string(),
                value: OpenFlowModuleValue::Script {
                    path: "slow_task".to_string(),
                    language: Some("rust".to_string()),
                    entry_point: Some("slow".to_string()),
                    code: Some(
                        r#"fn slow(input: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    std::thread::sleep(std::time::Duration::from_secs(5));
    Ok(input)
}"#
                        .to_string(),
                    ),
                    dependencies: None,
                },
            }],
        },
    };

    let input = json!({"data": "test"});
    let result = run_workflow_with_timeout(&spec, input, Duration::from_secs(1)).await;

    // Should timeout
    assert!(result.is_err());
    match result.unwrap_err() {
        OpenFlowError::Timeout { seconds } => {
            assert_eq!(seconds, 1);
        }
        other => panic!("Expected Timeout error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_run_workflow_sufficient_timeout() {
    let spec = OpenFlowSpec {
        summary: "Fast workflow".to_string(),
        value: OpenFlowValue {
            modules: vec![OpenFlowModule {
                id: "fast_task".to_string(),
                value: OpenFlowModuleValue::Script {
                    path: "fast_task".to_string(),
                    language: Some("rust".to_string()),
                    entry_point: Some("fast".to_string()),
                    code: Some(
                        r#"fn fast(input: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    Ok(serde_json::json!({"result": "done"}))
}"#
                        .to_string(),
                    ),
                    dependencies: None,
                },
            }],
        },
    };

    let input = json!({"data": "test"});
    let result = run_workflow_with_timeout(&spec, input, Duration::from_secs(10)).await;

    // Should succeed
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["result"], "done");
}
