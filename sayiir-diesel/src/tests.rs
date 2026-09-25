#![cfg(test)]

use bytes::Bytes;
use sayiir_core::snapshot::{SignalKind, SignalRequest, WorkflowSnapshot};
use sayiir_persistence::{SignalStore, SnapshotStore};

async fn setup_backend() -> crate::DieselBackend {
    crate::DieselBackend::new(":memory:").await.unwrap()
}

fn test_definition_hash() -> sayiir_core::DefinitionHash {
    sayiir_core::DefinitionHash::from("test-definition-hash")
}

#[tokio::test]
async fn test_save_and_load_snapshot() {
    let backend = setup_backend().await;

    let mut snapshot = WorkflowSnapshot::new("test-instance-1", test_definition_hash());

    backend.save_snapshot(&mut snapshot).await.unwrap();
    let loaded = backend.load_snapshot("test-instance-1").await.unwrap();

    assert_eq!(loaded.instance_id.as_ref(), "test-instance-1");
}

#[tokio::test]
async fn test_delete_snapshot() {
    let backend = setup_backend().await;

    let mut snapshot = WorkflowSnapshot::new("test-instance-2", test_definition_hash());

    backend.save_snapshot(&mut snapshot).await.unwrap();
    backend.delete_snapshot("test-instance-2").await.unwrap();

    let result = backend.load_snapshot("test-instance-2").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_snapshots() {
    let backend = setup_backend().await;

    let mut snap1 = WorkflowSnapshot::new("test-instance-3", test_definition_hash());
    let mut snap2 = WorkflowSnapshot::new("test-instance-4", test_definition_hash());

    backend.save_snapshot(&mut snap1).await.unwrap();
    backend.save_snapshot(&mut snap2).await.unwrap();

    let list = backend.list_snapshots().await.unwrap();
    assert!(list.contains(&"test-instance-3".to_string()));
    assert!(list.contains(&"test-instance-4".to_string()));
}

#[tokio::test]
async fn test_signal_store_and_retrieve() {
    let backend = setup_backend().await;

    let mut snapshot = WorkflowSnapshot::new("test-instance-5", test_definition_hash());
    backend.save_snapshot(&mut snapshot).await.unwrap();

    let request = SignalRequest {
        reason: Some("test cancel".to_string()),
        requested_by: Some("test-user".to_string()),
        requested_at: chrono::Utc::now(),
    };

    backend
        .store_signal("test-instance-5", SignalKind::Cancel, request.clone())
        .await
        .unwrap();

    let retrieved = backend
        .get_signal("test-instance-5", SignalKind::Cancel)
        .await
        .unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().reason, Some("test cancel".to_string()));
}

#[tokio::test]
async fn test_clear_signal() {
    let backend = setup_backend().await;

    let mut snapshot = WorkflowSnapshot::new("test-instance-6", test_definition_hash());
    backend.save_snapshot(&mut snapshot).await.unwrap();

    let request = SignalRequest {
        reason: Some("test pause".to_string()),
        requested_by: Some("test-user".to_string()),
        requested_at: chrono::Utc::now(),
    };

    backend
        .store_signal("test-instance-6", SignalKind::Pause, request)
        .await
        .unwrap();
    backend
        .clear_signal("test-instance-6", SignalKind::Pause)
        .await
        .unwrap();

    let retrieved = backend
        .get_signal("test-instance-6", SignalKind::Pause)
        .await
        .unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_send_and_consume_event() {
    let backend = setup_backend().await;

    let payload = Bytes::from("test event payload");
    backend
        .send_event("test-instance-7", "test-signal", payload.clone())
        .await
        .unwrap();

    let consumed = backend
        .consume_event("test-instance-7", "test-signal")
        .await
        .unwrap();
    assert!(consumed.is_some());
    assert_eq!(consumed.unwrap(), payload);

    // Second consume should return None
    let consumed2 = backend
        .consume_event("test-instance-7", "test-signal")
        .await
        .unwrap();
    assert!(consumed2.is_none());
}

#[tokio::test]
async fn test_event_fifo_order() {
    let backend = setup_backend().await;

    let payload1 = Bytes::from("event 1");
    let payload2 = Bytes::from("event 2");
    let payload3 = Bytes::from("event 3");

    backend
        .send_event("test-instance-8", "test-signal", payload1.clone())
        .await
        .unwrap();
    backend
        .send_event("test-instance-8", "test-signal", payload2.clone())
        .await
        .unwrap();
    backend
        .send_event("test-instance-8", "test-signal", payload3.clone())
        .await
        .unwrap();

    // Should consume in FIFO order
    assert_eq!(
        backend
            .consume_event("test-instance-8", "test-signal")
            .await
            .unwrap()
            .unwrap(),
        payload1
    );
    assert_eq!(
        backend
            .consume_event("test-instance-8", "test-signal")
            .await
            .unwrap()
            .unwrap(),
        payload2
    );
    assert_eq!(
        backend
            .consume_event("test-instance-8", "test-signal")
            .await
            .unwrap()
            .unwrap(),
        payload3
    );
    assert!(
        backend
            .consume_event("test-instance-8", "test-signal")
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn test_snapshot_update() {
    let backend = setup_backend().await;

    let mut snapshot = WorkflowSnapshot::new("test-instance-9", test_definition_hash());

    backend.save_snapshot(&mut snapshot).await.unwrap();

    // Update and save again
    backend.save_snapshot(&mut snapshot).await.unwrap();

    let loaded = backend.load_snapshot("test-instance-9").await.unwrap();
    assert_eq!(loaded.instance_id.as_ref(), "test-instance-9");
}
