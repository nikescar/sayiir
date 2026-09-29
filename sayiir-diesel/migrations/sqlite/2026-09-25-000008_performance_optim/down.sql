-- Reverse migration 008 for SQLite

-- Reverse sayiir_workflow_claims → sayiir_task_claims
CREATE TABLE sayiir_task_claims (
    instance_id TEXT        NOT NULL,
    task_id     TEXT        NOT NULL,
    worker_id   TEXT        NOT NULL,
    claimed_at  TEXT        NOT NULL DEFAULT (datetime('now')),
    expires_at  TEXT,
    PRIMARY KEY (instance_id, task_id)
);

INSERT INTO sayiir_task_claims
SELECT instance_id, task_id, worker_id, claimed_at, expires_at
FROM sayiir_workflow_claims;

DROP TABLE sayiir_workflow_claims;

-- Remove new columns from tasks
CREATE TABLE sayiir_workflow_tasks_new (
    instance_id  TEXT        NOT NULL,
    task_id      TEXT        NOT NULL,
    status       TEXT        NOT NULL DEFAULT 'pending',
    worker_id    TEXT,
    started_at   TEXT,
    completed_at TEXT,
    error        TEXT,
    PRIMARY KEY (instance_id, task_id)
);

INSERT INTO sayiir_workflow_tasks_new SELECT
    instance_id, task_id, status, worker_id, started_at, completed_at, error
FROM sayiir_workflow_tasks;

DROP TABLE sayiir_workflow_tasks;
ALTER TABLE sayiir_workflow_tasks_new RENAME TO sayiir_workflow_tasks;
CREATE INDEX IF NOT EXISTS idx_tasks_status ON sayiir_workflow_tasks (status);

-- Remove data_hash from history
CREATE TABLE sayiir_workflow_snapshot_history_new (
    id              INTEGER     PRIMARY KEY AUTOINCREMENT,
    instance_id     TEXT        NOT NULL,
    version         INT         NOT NULL,
    status          TEXT        NOT NULL,
    current_task_id TEXT,
    data            BLOB        NOT NULL,
    created_at      TEXT        NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO sayiir_workflow_snapshot_history_new SELECT
    id, instance_id, version, status, current_task_id, data, created_at
FROM sayiir_workflow_snapshot_history;

DROP TABLE sayiir_workflow_snapshot_history;
ALTER TABLE sayiir_workflow_snapshot_history_new RENAME TO sayiir_workflow_snapshot_history;
CREATE INDEX idx_history_instance ON sayiir_workflow_snapshot_history (instance_id, version);

-- Restore old snapshots table (data becomes NOT NULL again)
DROP INDEX IF EXISTS idx_snapshots_inprogress;

CREATE TABLE sayiir_workflow_snapshots_old (
    instance_id          TEXT        PRIMARY KEY,
    status               TEXT        NOT NULL,
    definition_hash      TEXT,
    current_task_id      TEXT,
    completed_task_count INT         NOT NULL DEFAULT 0,
    data                 BLOB        NOT NULL,
    error                TEXT,
    started_at           TEXT        NOT NULL DEFAULT (datetime('now')),
    completed_at         TEXT,
    updated_at           TEXT        NOT NULL DEFAULT (datetime('now')),
    position_kind        TEXT,
    delay_wake_at        TEXT,
    trace_parent         TEXT,
    task_priority        INTEGER     NOT NULL DEFAULT 3,
    task_tags            TEXT        NOT NULL DEFAULT '[]'
);

INSERT INTO sayiir_workflow_snapshots_old SELECT
    instance_id, status, definition_hash, current_task_id,
    completed_task_count, COALESCE(data, X''), error, started_at, completed_at, updated_at,
    position_kind, delay_wake_at, trace_parent, task_priority, task_tags
FROM sayiir_workflow_snapshots;

DROP TABLE sayiir_workflow_snapshots;
ALTER TABLE sayiir_workflow_snapshots_old RENAME TO sayiir_workflow_snapshots;

CREATE INDEX IF NOT EXISTS idx_snapshots_position ON sayiir_workflow_snapshots (position_kind);
CREATE INDEX IF NOT EXISTS idx_sayiir_task_tags ON sayiir_workflow_snapshots (task_tags);
