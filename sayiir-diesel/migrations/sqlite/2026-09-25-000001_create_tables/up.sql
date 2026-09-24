-- Workflow execution state
CREATE TABLE workflow_executions (
    id TEXT PRIMARY KEY NOT NULL,
    workflow_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('running', 'completed', 'failed')),
    input TEXT NOT NULL,
    output TEXT,
    error TEXT,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE INDEX idx_workflow_executions_status ON workflow_executions(status);
CREATE INDEX idx_workflow_executions_workflow_id ON workflow_executions(workflow_id);

-- Workflow event log
CREATE TABLE workflow_events (
    id TEXT PRIMARY KEY NOT NULL,
    execution_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload TEXT NOT NULL,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (execution_id) REFERENCES workflow_executions(id) ON DELETE CASCADE
);

CREATE INDEX idx_workflow_events_execution_id ON workflow_events(execution_id);
CREATE INDEX idx_workflow_events_timestamp ON workflow_events(timestamp);

-- Scheduled delays
CREATE TABLE workflow_timers (
    id TEXT PRIMARY KEY NOT NULL,
    execution_id TEXT NOT NULL,
    fire_at TEXT NOT NULL,
    payload TEXT NOT NULL,
    FOREIGN KEY (execution_id) REFERENCES workflow_executions(id) ON DELETE CASCADE
);

CREATE INDEX idx_workflow_timers_fire_at ON workflow_timers(fire_at);
CREATE INDEX idx_workflow_timers_execution_id ON workflow_timers(execution_id);

-- External signals
CREATE TABLE workflow_signals (
    id TEXT PRIMARY KEY NOT NULL,
    execution_id TEXT NOT NULL,
    signal_name TEXT NOT NULL,
    payload TEXT NOT NULL,
    received_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (execution_id) REFERENCES workflow_executions(id) ON DELETE CASCADE
);

CREATE INDEX idx_workflow_signals_execution_id ON workflow_signals(execution_id);
CREATE INDEX idx_workflow_signals_signal_name ON workflow_signals(signal_name);
