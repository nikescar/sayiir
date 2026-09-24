use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use sayiir_persistence::{BackendError, SnapshotStore, SignalStore};
use sayiir_core::snapshot::{WorkflowSnapshot, SignalKind, SignalRequest};
use bytes::Bytes;

#[cfg(feature = "sqlite")]
type Connection = diesel_async::sync_connection_wrapper::SyncConnectionWrapper<diesel::SqliteConnection>;

#[cfg(feature = "postgres")]
type Connection = diesel_async::AsyncPgConnection;

#[cfg(feature = "mysql")]
type Connection = diesel_async::AsyncMysqlConnection;

use crate::{DieselError, Result};

/// Diesel-backed persistence for sayiir workflows
pub struct DieselBackend {
    pool: Pool<Connection>,
}

impl DieselBackend {
    /// Create a new DieselBackend with connection pool and run migrations
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = create_pool(database_url).await?;
        run_migrations(&pool).await?;
        Ok(Self { pool })
    }
}

async fn create_pool(database_url: &str) -> Result<Pool<Connection>> {
    let config = AsyncDieselConnectionManager::<Connection>::new(database_url);
    let pool = Pool::builder(config)
        .build()
        .map_err(|e| DieselError::ConnectionError(e.to_string()))?;
    Ok(pool)
}

async fn run_migrations(_pool: &Pool<Connection>) -> Result<()> {
    // TODO: Implement migration runner
    Ok(())
}

// SnapshotStore implementation
impl SnapshotStore for DieselBackend {
    async fn save_snapshot(&self, snapshot: &mut WorkflowSnapshot) -> std::result::Result<(), BackendError> {
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use crate::schema::sayiir_workflow_snapshots::dsl::*;

        let data_bytes = serde_json::to_vec(snapshot)
            .map_err(|e| BackendError::Serialization(e.to_string()))?;

        let mut conn = self.pool.get().await
            .map_err(|e| BackendError::Backend(format!("pool error: {}", e)))?;

        let inst_id: &str = &snapshot.instance_id;

        diesel::replace_into(sayiir_workflow_snapshots)
            .values((
                instance_id.eq(inst_id),
                status.eq(snapshot.state.as_ref()),
                data.eq(&data_bytes),
                completed_task_count.eq(snapshot.completed_task_count() as i32),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| BackendError::Backend(format!("save failed: {}", e)))?;

        Ok(())
    }

    async fn save_task_result(
        &self,
        inst_id: &str,
        task_id: &sayiir_core::TaskId,
        output: Bytes,
    ) -> std::result::Result<(), BackendError> {
        // Load snapshot, mark task completed, save back
        let mut snapshot = self.load_snapshot(inst_id).await?;
        snapshot.mark_task_completed(*task_id, output);
        self.save_snapshot(&mut snapshot).await
    }

    async fn load_snapshot(&self, inst_id: &str) -> std::result::Result<WorkflowSnapshot, BackendError> {
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use crate::schema::sayiir_workflow_snapshots::dsl::*;

        let mut conn = self.pool.get().await
            .map_err(|e| BackendError::Backend(format!("pool error: {}", e)))?;

        let row: crate::models::WorkflowSnapshot = sayiir_workflow_snapshots
            .filter(instance_id.eq(inst_id))
            .first(&mut conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => BackendError::NotFound(inst_id.to_string()),
                e => BackendError::Backend(format!("load failed: {}", e)),
            })?;

        let snapshot: WorkflowSnapshot = serde_json::from_slice(&row.data)
            .map_err(|e| BackendError::Serialization(e.to_string()))?;

        Ok(snapshot)
    }

    async fn delete_snapshot(&self, inst_id: &str) -> std::result::Result<(), BackendError> {
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use crate::schema::sayiir_workflow_snapshots::dsl::*;

        let mut conn = self.pool.get().await
            .map_err(|e| BackendError::Backend(format!("pool error: {}", e)))?;

        diesel::delete(sayiir_workflow_snapshots.filter(instance_id.eq(inst_id)))
            .execute(&mut conn)
            .await
            .map_err(|e| BackendError::Backend(format!("delete failed: {}", e)))?;

        Ok(())
    }

    async fn list_snapshots(&self) -> std::result::Result<Vec<String>, BackendError> {
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use crate::schema::sayiir_workflow_snapshots::dsl::*;

        let mut conn = self.pool.get().await
            .map_err(|e| BackendError::Backend(format!("pool error: {}", e)))?;

        let ids: Vec<String> = sayiir_workflow_snapshots
            .select(instance_id)
            .load(&mut conn)
            .await
            .map_err(|e| BackendError::Backend(format!("list failed: {}", e)))?;

        Ok(ids)
    }
}

// SignalStore implementation
impl SignalStore for DieselBackend {
    async fn store_signal(
        &self,
        inst_id: &str,
        signal_kind: SignalKind,
        request: SignalRequest,
    ) -> std::result::Result<(), BackendError> {
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use crate::schema::sayiir_workflow_signals::dsl::*;

        let mut conn = self.pool.get().await
            .map_err(|e| BackendError::Backend(format!("pool error: {}", e)))?;

        let kind_str = match signal_kind {
            SignalKind::Cancel => "cancel",
            SignalKind::Pause => "pause",
        };

        diesel::insert_into(sayiir_workflow_signals)
            .values((
                instance_id.eq(inst_id),
                kind.eq(kind_str),
                reason.eq(request.reason.as_deref()),
                requested_by.eq(request.requested_by.as_deref()),
            ))
            .on_conflict((instance_id, kind))
            .do_update()
            .set((
                reason.eq(request.reason.as_deref()),
                requested_by.eq(request.requested_by.as_deref()),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| BackendError::Backend(format!("store_signal failed: {}", e)))?;

        Ok(())
    }

    async fn get_signal(
        &self,
        inst_id: &str,
        signal_kind: SignalKind,
    ) -> std::result::Result<Option<SignalRequest>, BackendError> {
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use crate::schema::sayiir_workflow_signals::dsl::*;

        let mut conn = self.pool.get().await
            .map_err(|e| BackendError::Backend(format!("pool error: {}", e)))?;

        let kind_str = match signal_kind {
            SignalKind::Cancel => "cancel",
            SignalKind::Pause => "pause",
        };

        let result: Option<crate::models::WorkflowSignal> = sayiir_workflow_signals
            .filter(instance_id.eq(inst_id).and(kind.eq(kind_str)))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| BackendError::Backend(format!("get_signal failed: {}", e)))?;

        Ok(result.map(|sig| {
            use chrono::DateTime;
            let requested_at = DateTime::parse_from_rfc3339(&sig.created_at)
                .ok()
                .and_then(|dt| dt.with_timezone(&chrono::Utc).into())
                .unwrap_or_else(chrono::Utc::now);

            SignalRequest {
                reason: sig.reason,
                requested_by: sig.requested_by,
                requested_at,
            }
        }))
    }

    async fn clear_signal(
        &self,
        inst_id: &str,
        signal_kind: SignalKind,
    ) -> std::result::Result<(), BackendError> {
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use crate::schema::sayiir_workflow_signals::dsl::*;

        let mut conn = self.pool.get().await
            .map_err(|e| BackendError::Backend(format!("pool error: {}", e)))?;

        let kind_str = match signal_kind {
            SignalKind::Cancel => "cancel",
            SignalKind::Pause => "pause",
        };

        diesel::delete(
            sayiir_workflow_signals.filter(instance_id.eq(inst_id).and(kind.eq(kind_str)))
        )
        .execute(&mut conn)
        .await
        .map_err(|e| BackendError::Backend(format!("clear_signal failed: {}", e)))?;

        Ok(())
    }

    async fn send_event(
        &self,
        inst_id: &str,
        sig_name: &str,
        event_payload: Bytes,
    ) -> std::result::Result<(), BackendError> {
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use crate::schema::sayiir_workflow_events::dsl::*;

        let mut conn = self.pool.get().await
            .map_err(|e| BackendError::Backend(format!("pool error: {}", e)))?;

        diesel::insert_into(sayiir_workflow_events)
            .values((
                instance_id.eq(inst_id),
                signal_name.eq(sig_name),
                payload.eq(&event_payload[..]),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| BackendError::Backend(format!("send_event failed: {}", e)))?;

        Ok(())
    }

    async fn consume_event(
        &self,
        inst_id: &str,
        sig_name: &str,
    ) -> std::result::Result<Option<Bytes>, BackendError> {
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use crate::schema::sayiir_workflow_events::dsl::*;

        let mut conn = self.pool.get().await
            .map_err(|e| BackendError::Backend(format!("pool error: {}", e)))?;

        // Find oldest event for this (instance_id, signal_name) pair
        let event: Option<crate::models::WorkflowEvent> = sayiir_workflow_events
            .filter(instance_id.eq(inst_id).and(signal_name.eq(sig_name)))
            .order(id.asc())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| BackendError::Backend(format!("consume_event query failed: {}", e)))?;

        if let Some(evt) = event {
            // Delete the event
            diesel::delete(sayiir_workflow_events.filter(id.eq(evt.id)))
                .execute(&mut conn)
                .await
                .map_err(|e| BackendError::Backend(format!("consume_event delete failed: {}", e)))?;

            Ok(Some(Bytes::from(evt.payload)))
        } else {
            Ok(None)
        }
    }
}
