-- Workflow execution state
CREATE TABLE workflow_executions (
    id VARCHAR(255) PRIMARY KEY NOT NULL,
    workflow_id VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL CHECK(status IN ('running', 'completed', 'failed')),
    input TEXT NOT NULL,
    output TEXT,
    error TEXT,
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP NULL,
    INDEX idx_workflow_executions_status (status),
    INDEX idx_workflow_executions_workflow_id (workflow_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Workflow event log
CREATE TABLE workflow_events (
    id VARCHAR(255) PRIMARY KEY NOT NULL,
    execution_id VARCHAR(255) NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    payload TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_workflow_events_execution_id (execution_id),
    INDEX idx_workflow_events_timestamp (timestamp),
    FOREIGN KEY (execution_id) REFERENCES workflow_executions(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Scheduled delays
CREATE TABLE workflow_timers (
    id VARCHAR(255) PRIMARY KEY NOT NULL,
    execution_id VARCHAR(255) NOT NULL,
    fire_at TIMESTAMP NOT NULL,
    payload TEXT NOT NULL,
    INDEX idx_workflow_timers_fire_at (fire_at),
    INDEX idx_workflow_timers_execution_id (execution_id),
    FOREIGN KEY (execution_id) REFERENCES workflow_executions(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- External signals
CREATE TABLE workflow_signals (
    id VARCHAR(255) PRIMARY KEY NOT NULL,
    execution_id VARCHAR(255) NOT NULL,
    signal_name VARCHAR(255) NOT NULL,
    payload TEXT NOT NULL,
    received_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_workflow_signals_execution_id (execution_id),
    INDEX idx_workflow_signals_signal_name (signal_name),
    FOREIGN KEY (execution_id) REFERENCES workflow_executions(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
