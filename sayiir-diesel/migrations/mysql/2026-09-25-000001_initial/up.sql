-- Workflow snapshots: current state of each workflow instance (1:1 with instance).
CREATE TABLE IF NOT EXISTS sayiir_workflow_snapshots (
    instance_id          VARCHAR(255) PRIMARY KEY,
    status               VARCHAR(50)  NOT NULL,
    definition_hash      VARCHAR(255),
    current_task_id      VARCHAR(255),
    completed_task_count INT          NOT NULL DEFAULT 0,
    data                 BLOB         NOT NULL,
    error                TEXT,
    started_at           TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at         TIMESTAMP    NULL,
    updated_at           TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_snapshots_status (status),
    INDEX idx_snapshots_task (current_task_id),
    INDEX idx_snapshots_updated (updated_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Snapshot history: append-only log of every checkpoint (1:N per instance).
CREATE TABLE IF NOT EXISTS sayiir_workflow_snapshot_history (
    id              BIGINT       AUTO_INCREMENT PRIMARY KEY,
    instance_id     VARCHAR(255) NOT NULL,
    version         INT          NOT NULL,
    status          VARCHAR(50)  NOT NULL,
    current_task_id VARCHAR(255),
    data            BLOB         NOT NULL,
    created_at      TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_history_instance (instance_id, version)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Individual task states for observability (1:N per instance).
CREATE TABLE IF NOT EXISTS sayiir_workflow_tasks (
    instance_id  VARCHAR(255) NOT NULL,
    task_id      VARCHAR(255) NOT NULL,
    status       VARCHAR(50)  NOT NULL DEFAULT 'pending',
    worker_id    VARCHAR(255),
    started_at   TIMESTAMP    NULL,
    completed_at TIMESTAMP    NULL,
    error        TEXT,
    PRIMARY KEY (instance_id, task_id),
    INDEX idx_tasks_status (status)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Cancel/pause signals (at most one per kind per instance).
CREATE TABLE IF NOT EXISTS sayiir_workflow_signals (
    instance_id  VARCHAR(255) NOT NULL,
    kind         VARCHAR(50)  NOT NULL,
    reason       TEXT,
    requested_by VARCHAR(255),
    created_at   TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (instance_id, kind)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Distributed worker task claims (one active claim per task).
CREATE TABLE IF NOT EXISTS sayiir_task_claims (
    instance_id VARCHAR(255) NOT NULL,
    task_id     VARCHAR(255) NOT NULL,
    worker_id   VARCHAR(255) NOT NULL,
    claimed_at  TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at  TIMESTAMP    NULL,
    PRIMARY KEY (instance_id, task_id),
    INDEX idx_claims_expires (expires_at),
    INDEX idx_claims_worker (worker_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- External events (signals) buffered per (instance_id, signal_name) in FIFO order.
CREATE TABLE IF NOT EXISTS sayiir_workflow_events (
    id           BIGINT       AUTO_INCREMENT PRIMARY KEY,
    instance_id  VARCHAR(255) NOT NULL,
    signal_name  VARCHAR(255) NOT NULL,
    payload      BLOB         NOT NULL,
    created_at   TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_events_instance_signal (instance_id, signal_name, id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
