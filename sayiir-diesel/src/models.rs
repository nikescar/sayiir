#![allow(missing_docs)]

use diesel::prelude::*;

/// Workflow snapshot (current state)
#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::sayiir_workflow_snapshots)]
pub struct WorkflowSnapshot {
    pub instance_id: String,
    pub status: String,
    pub definition_hash: Option<String>,
    pub current_task_id: Option<String>,
    pub completed_task_count: i32,
    pub data: Vec<u8>,
    pub error: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub updated_at: String,
}

/// Snapshot history entry
#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::sayiir_workflow_snapshot_history)]
pub struct SnapshotHistory {
    pub id: i64,
    pub instance_id: String,
    pub version: i32,
    pub status: String,
    pub current_task_id: Option<String>,
    pub data: Vec<u8>,
    pub created_at: String,
}

/// Task state for observability
#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::sayiir_workflow_tasks)]
pub struct WorkflowTask {
    pub instance_id: String,
    pub task_id: String,
    pub status: String,
    pub worker_id: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub error: Option<String>,
}

/// Cancel/pause signal
#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::sayiir_workflow_signals)]
pub struct WorkflowSignal {
    pub instance_id: String,
    pub kind: String,
    pub reason: Option<String>,
    pub requested_by: Option<String>,
    pub created_at: String,
}

/// Task claim for distributed execution
#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::sayiir_task_claims)]
pub struct TaskClaim {
    pub instance_id: String,
    pub task_id: String,
    pub worker_id: String,
    pub claimed_at: String,
    pub expires_at: Option<String>,
}

/// External event (buffered signal)
#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::sayiir_workflow_events)]
pub struct WorkflowEvent {
    pub id: i64,
    pub instance_id: String,
    pub signal_name: String,
    pub payload: Vec<u8>,
    pub created_at: String,
}
