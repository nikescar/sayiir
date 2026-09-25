use sayiir_openflow::{export_mermaid, export_openflow_json, OpenFlowModule, OpenFlowModuleValue, OpenFlowSpec, OpenFlowValue};
use serde_json::json;
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

    info!("🎬 Exporting video pipeline workflow to OpenFlow format with embedded code");

    let mut serde_json_dep = serde_json::Map::new();
    serde_json_dep.insert("serde_json".to_string(), json!("1.0"));

    let spec = OpenFlowSpec {
        summary: "Video processing pipeline with parallel transcoding and content moderation".to_string(),
        value: OpenFlowValue {
            modules: vec![
                OpenFlowModule {
                    id: "download_video".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "download_video".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(r#"use serde_json::{json, Value};

fn run(input: Value) -> Result<Value, String> {
    let upload_id = input["upload_id"].as_str().ok_or("missing upload_id")?;
    let user_id = input["user_id"].as_str().ok_or("missing user_id")?;
    let source_url = input["source_url"].as_str().ok_or("missing source_url")?;

    // Simulate video metadata extraction
    let video_file = json!({
        "upload_id": upload_id,
        "user_id": user_id,
        "local_path": format!("/tmp/{}/source.mp4", upload_id),
        "size_bytes": 52428800,
        "format": "h264",
        "width": 1920,
        "height": 1080,
        "duration_secs": 120.5
    });

    Ok(video_file)
}"#.to_string()),
                        dependencies: Some(serde_json_dep.clone()),
                    },
                },
                OpenFlowModule {
                    id: "validate_upload".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "validate_upload".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(r#"use serde_json::Value;

fn run(input: Value) -> Result<Value, String> {
    let size_bytes = input["size_bytes"].as_u64().ok_or("missing size_bytes")?;
    let width = input["width"].as_u64().ok_or("missing width")?;
    let height = input["height"].as_u64().ok_or("missing height")?;

    let max_size: u64 = 10 * 1024 * 1024 * 1024;

    if size_bytes > max_size {
        return Err(format!("file too large: {} bytes", size_bytes));
    }
    if width == 0 || height == 0 {
        return Err("no video stream detected".to_string());
    }

    Ok(input)
}"#.to_string()),
                        dependencies: Some(serde_json_dep.clone()),
                    },
                },
                OpenFlowModule {
                    id: "transcode_720p".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "transcode_720p".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(r#"use serde_json::{json, Value};

fn run(input: Value) -> Result<Value, String> {
    let upload_id = input["upload_id"].as_str().ok_or("missing upload_id")?;
    let duration = input["duration_secs"].as_f64().unwrap_or(0.0);

    let bitrate_kbps = 2500.0;
    let estimated_size = ((bitrate_kbps / 8.0) * duration * 1024.0) as u64;

    let result = json!({
        "resolution": "720p",
        "path": format!("/tmp/{}/720p.mp4", upload_id),
        "duration_secs": duration * 1.05,
        "file_size": estimated_size
    });

    Ok(result)
}"#.to_string()),
                        dependencies: Some(serde_json_dep.clone()),
                    },
                },
                OpenFlowModule {
                    id: "transcode_1080p".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "transcode_1080p".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(r#"use serde_json::{json, Value};

fn run(input: Value) -> Result<Value, String> {
    let upload_id = input["upload_id"].as_str().ok_or("missing upload_id")?;
    let duration = input["duration_secs"].as_f64().unwrap_or(0.0);

    let bitrate_kbps = 5000.0;
    let estimated_size = ((bitrate_kbps / 8.0) * duration * 1024.0) as u64;

    let result = json!({
        "resolution": "1080p",
        "path": format!("/tmp/{}/1080p.mp4", upload_id),
        "duration_secs": duration * 1.1,
        "file_size": estimated_size
    });

    Ok(result)
}"#.to_string()),
                        dependencies: Some(serde_json_dep.clone()),
                    },
                },
                OpenFlowModule {
                    id: "transcode_4k".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "transcode_4k".to_string(),
                        language: Some("rust".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(r#"use serde_json::{json, Value};

fn run(input: Value) -> Result<Value, String> {
    let upload_id = input["upload_id"].as_str().ok_or("missing upload_id")?;
    let width = input["width"].as_u64().ok_or("missing width")?;
    let height = input["height"].as_u64().ok_or("missing height")?;
    let duration = input["duration_secs"].as_f64().unwrap_or(0.0);

    if height < 2160 || width < 3840 {
        return Ok(json!(null));
    }

    let bitrate_kbps = 20000.0;
    let estimated_size = ((bitrate_kbps / 8.0) * duration * 1024.0) as u64;

    let result = json!({
        "resolution": "4k",
        "path": format!("/tmp/{}/4k.mp4", upload_id),
        "duration_secs": duration * 1.5,
        "file_size": estimated_size
    });

    Ok(result)
}"#.to_string()),
                        dependencies: Some(serde_json_dep.clone()),
                    },
                },
                OpenFlowModule {
                    id: "generate_thumbnails".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "generate_thumbnails".to_string(),
                        language: Some("python".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(r#"def run(input_data):
    upload_id = input_data.get('upload_id')
    if not upload_id:
        raise ValueError('missing upload_id')

    paths = [
        f"/tmp/{upload_id}/thumb_01.jpg",
        f"/tmp/{upload_id}/thumb_02.jpg",
        f"/tmp/{upload_id}/thumb_03.jpg"
    ]

    return {"paths": paths}"#.to_string()),
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "moderate_content".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "moderate_content".to_string(),
                        language: Some("python".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(r#"import random

def run(input_data):
    is_approved = random.random() < 0.9
    confidence = 0.85 + (random.random() * 0.14)

    verdict = "approved" if is_approved else "rejected"
    flags = [] if is_approved else ["policy_violation"]

    return {
        "verdict": verdict,
        "confidence": round(confidence, 2),
        "flags": flags
    }"#.to_string()),
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "merge_results".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "merge_results".to_string(),
                        language: Some("node".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(r#"function run(input) {
    const transcodes = [];

    if (input.transcode_720p) {
        transcodes.push(input.transcode_720p);
    }
    if (input.transcode_1080p) {
        transcodes.push(input.transcode_1080p);
    }
    if (input.transcode_4k) {
        transcodes.push(input.transcode_4k);
    }

    const upload_id = transcodes.length > 0
        ? transcodes[0].path.split('/')[2]
        : 'unknown';

    return {
        upload_id: upload_id,
        transcodes: transcodes,
        thumbnails: input.thumbnails || { paths: [] },
        moderation: input.moderation || { verdict: 'unknown' }
    };
}"#.to_string()),
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "check_moderation".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "check_moderation".to_string(),
                        language: Some("node".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(r#"function run(input) {
    const verdict = input.moderation?.verdict || 'unknown';
    return { route: verdict };
}"#.to_string()),
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "upload_to_cdn".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "upload_to_cdn".to_string(),
                        language: Some("node".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(r#"function run(input) {
    const uploadId = input.upload_id;
    const transcodes = input.transcodes || [];
    const thumbnails = input.thumbnails?.paths || [];

    const videoUrls = transcodes.map(tc =>
        `https://cdn.example.com/${uploadId}/${tc.resolution}.mp4`
    );

    const thumbnailUrls = thumbnails.map((_, i) =>
        `https://cdn.example.com/${uploadId}/thumb_${i}.jpg`
    );

    return {
        upload_id: uploadId,
        video_urls: videoUrls,
        thumbnail_urls: thumbnailUrls
    };
}"#.to_string()),
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "update_database".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "update_database".to_string(),
                        language: Some("python".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(r#"def run(input_data):
    upload_id = input_data.get('upload_id')
    video_urls = input_data.get('video_urls', [])

    return {
        "upload_id": upload_id,
        "status": "published",
        "video_count": len(video_urls),
        "updated_at": "2026-09-25T12:00:00Z"
    }"#.to_string()),
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "notify_user".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "notify_user".to_string(),
                        language: Some("python".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(r#"def run(input_data):
    upload_id = input_data.get('upload_id')
    video_urls = input_data.get('video_urls', [])

    message = f"Your video {upload_id} is ready! {len(video_urls)} formats available."

    return {
        "notification_sent": True,
        "message": message,
        "upload_id": upload_id
    }"#.to_string()),
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "cleanup_artifacts".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "cleanup_artifacts".to_string(),
                        language: Some("python".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(r#"def run(input_data):
    upload_id = input_data.get('upload_id')
    transcodes = input_data.get('transcodes', [])
    thumbnails = input_data.get('thumbnails', {}).get('paths', [])

    total_artifacts = len(transcodes) + len(thumbnails)

    return {
        "upload_id": upload_id,
        "artifacts_deleted": total_artifacts,
        "cleanup_complete": True
    }"#.to_string()),
                        dependencies: None,
                    },
                },
                OpenFlowModule {
                    id: "notify_rejection".to_string(),
                    value: OpenFlowModuleValue::Script {
                        path: "notify_rejection".to_string(),
                        language: Some("python".to_string()),
                        entry_point: Some("run".to_string()),
                        code: Some(r#"def run(input_data):
    upload_id = input_data.get('upload_id', 'unknown')

    message = f"Video {upload_id} was rejected due to policy violations."

    return {
        "notification_sent": True,
        "message": message,
        "upload_id": upload_id,
        "status": "rejected"
    }"#.to_string()),
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

    info!("🏁 Export complete with {} embedded task implementations", spec.value.modules.len());

    Ok(())
}
