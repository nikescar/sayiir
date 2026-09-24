// @generated automatically by Diesel CLI.

diesel::table! {
    workflow_executions (id) {
        id -> Text,
        workflow_id -> Text,
        status -> Text,
        input -> Text,
        output -> Nullable<Text>,
        error -> Nullable<Text>,
        started_at -> Text,
        completed_at -> Nullable<Text>,
    }
}

diesel::table! {
    workflow_events (id) {
        id -> Text,
        execution_id -> Text,
        event_type -> Text,
        payload -> Text,
        timestamp -> Text,
    }
}

diesel::table! {
    workflow_timers (id) {
        id -> Text,
        execution_id -> Text,
        fire_at -> Text,
        payload -> Text,
    }
}

diesel::table! {
    workflow_signals (id) {
        id -> Text,
        execution_id -> Text,
        signal_name -> Text,
        payload -> Text,
        received_at -> Text,
    }
}

diesel::joinable!(workflow_events -> workflow_executions (execution_id));
diesel::joinable!(workflow_timers -> workflow_executions (execution_id));
diesel::joinable!(workflow_signals -> workflow_executions (execution_id));

diesel::allow_tables_to_appear_in_same_query!(
    workflow_executions,
    workflow_events,
    workflow_timers,
    workflow_signals,
);
