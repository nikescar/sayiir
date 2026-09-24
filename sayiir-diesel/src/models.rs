#![allow(missing_docs)]

use diesel::prelude::*;

/// Database row for workflow execution state
#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::workflow_executions)]
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_id: String,
    pub status: String,
    pub input: String,
    pub output: Option<String>,
    pub error: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

/// Database row for workflow event log
#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::workflow_events)]
pub struct WorkflowEvent {
    pub id: String,
    pub execution_id: String,
    pub event_type: String,
    pub payload: String,
    pub timestamp: String,
}

/// Database row for scheduled delays
#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::workflow_timers)]
pub struct WorkflowTimer {
    pub id: String,
    pub execution_id: String,
    pub fire_at: String,
    pub payload: String,
}

/// Database row for external signals
#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::workflow_signals)]
pub struct WorkflowSignal {
    pub id: String,
    pub execution_id: String,
    pub signal_name: String,
    pub payload: String,
    pub received_at: String,
}
