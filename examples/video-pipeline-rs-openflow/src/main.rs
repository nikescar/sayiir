use sayiir_openflow::{export_mermaid, export_openflow_json, OpenFlowModule, OpenFlowModuleValue, OpenFlowSpec, OpenFlowValue};
use std::fs;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("🎬 Exporting video pipeline workflow to OpenFlow format");

    let spec = OpenFlowSpec {
        summary: "Video processing pipeline with parallel transcoding and content moderation".to_string(),
        value: OpenFlowValue {
            modules: vec![
                OpenFlowModule {
                    id: "download_video".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "tasks::download_video".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "validate_upload".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "tasks::validate_upload".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "transcode_720p".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "tasks::transcode_720p".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "transcode_1080p".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "tasks::transcode_1080p".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "transcode_4k".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "tasks::transcode_4k".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "generate_thumbnails".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "tasks::generate_thumbnails".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "moderate_content".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "tasks::moderate_content".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "merge_results".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "tasks::merge_results".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "check_moderation".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "tasks::check_moderation".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "upload_to_cdn".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "tasks::upload_to_cdn".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "update_database".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "tasks::update_database".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "notify_user".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "tasks::notify_user".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "cleanup_artifacts".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "tasks::cleanup_artifacts".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "notify_rejection".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "tasks::notify_rejection".to_string(),
                        language: None,
                        entry_point: None,
                        code: None,
                        dependencies: None,
                    },
                },
            ],
        },
    };

    let json = export_openflow_json(&spec)?;
    fs::write("workflow.openflow.json", &json)?;
    info!("✓ Exported to workflow.openflow.json");

    let mermaid = export_mermaid(&spec)?;
    fs::write("workflow.mermaid.md", &mermaid)?;
    info!("✓ Exported to workflow.mermaid.md");

    info!("🏁 Export complete");

    Ok(())
}
