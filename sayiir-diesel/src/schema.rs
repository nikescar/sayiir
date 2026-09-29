// @generated automatically by Diesel CLI.

diesel::table! {
    sayiir_workflow_snapshots (instance_id) {
        instance_id -> Text,
        status -> Text,
        definition_hash -> Nullable<Text>,
        current_task_id -> Nullable<Text>,
        completed_task_count -> Integer,
        data -> Binary,
        error -> Nullable<Text>,
        started_at -> Text,
        completed_at -> Nullable<Text>,
        updated_at -> Text,
    }
}

diesel::table! {
    sayiir_workflow_snapshot_history (id) {
        id -> BigInt,
        instance_id -> Text,
        version -> Integer,
        status -> Text,
        current_task_id -> Nullable<Text>,
        data -> Binary,
        created_at -> Text,
    }
}

diesel::table! {
    sayiir_workflow_tasks (instance_id, task_id) {
        instance_id -> Text,
        task_id -> Text,
        status -> Text,
        worker_id -> Nullable<Text>,
        started_at -> Nullable<Text>,
        completed_at -> Nullable<Text>,
        error -> Nullable<Text>,
    }
}

diesel::table! {
    sayiir_workflow_signals (instance_id, kind) {
        instance_id -> Text,
        kind -> Text,
        reason -> Nullable<Text>,
        requested_by -> Nullable<Text>,
        created_at -> Text,
    }
}

diesel::table! {
    sayiir_task_claims (instance_id, task_id) {
        instance_id -> Text,
        task_id -> Text,
        worker_id -> Text,
        claimed_at -> Text,
        expires_at -> Nullable<Text>,
    }
}

diesel::table! {
    sayiir_workflow_events (id) {
        id -> BigInt,
        instance_id -> Text,
        signal_name -> Text,
        payload -> Binary,
        created_at -> Text,
    }
}

diesel::joinable!(sayiir_workflow_snapshot_history -> sayiir_workflow_snapshots (instance_id));
diesel::joinable!(sayiir_workflow_tasks -> sayiir_workflow_snapshots (instance_id));

diesel::allow_tables_to_appear_in_same_query!(
    sayiir_workflow_snapshots,
    sayiir_workflow_snapshot_history,
    sayiir_workflow_tasks,
    sayiir_workflow_signals,
    sayiir_task_claims,
    sayiir_workflow_events,
);
