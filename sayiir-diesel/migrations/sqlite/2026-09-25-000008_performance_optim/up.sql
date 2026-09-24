-- Performance/schema rework for SQLite
-- SQLite keeps TEXT for IDs (no BYTEA equivalent), adds new columns

-- sayiir_workflow_snapshots: add new columns, data becomes nullable
-- SQLite doesn't support ALTER COLUMN, use table recreation
CREATE TABLE sayiir_workflow_snapshots_new (
    instance_id          TEXT        PRIMARY KEY,
    status               TEXT        NOT NULL,
    definition_hash      TEXT,
    current_task_id      TEXT,
    completed_task_count INT         NOT NULL DEFAULT 0,
    data                 BLOB,
    error                TEXT,
    started_at           TEXT        NOT NULL DEFAULT (datetime('now')),
    completed_at         TEXT,
    updated_at           TEXT        NOT NULL DEFAULT (datetime('now')),
    position_kind        TEXT,
    delay_wake_at        TEXT,
    trace_parent         TEXT,
    task_priority        INTEGER     NOT NULL DEFAULT 3,
    task_tags            TEXT        NOT NULL DEFAULT '[]',
    history_version      INT         NOT NULL DEFAULT 1,
    data_hash            BLOB
);

INSERT INTO sayiir_workflow_snapshots_new SELECT
    instance_id, status, definition_hash, current_task_id,
    completed_task_count, data, error, started_at, completed_at, updated_at,
    position_kind, delay_wake_at, trace_parent, task_priority, task_tags,
    1, NULL
FROM sayiir_workflow_snapshots;

-- Update history_version from history table
UPDATE sayiir_workflow_snapshots_new
SET history_version = (
    SELECT COALESCE(MAX(version), 0)
    FROM sayiir_workflow_snapshot_history h
    WHERE h.instance_id = sayiir_workflow_snapshots_new.instance_id
);

DROP TABLE sayiir_workflow_snapshots;
ALTER TABLE sayiir_workflow_snapshots_new RENAME TO sayiir_workflow_snapshots;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_snapshots_position ON sayiir_workflow_snapshots (position_kind);
CREATE INDEX IF NOT EXISTS idx_sayiir_task_tags ON sayiir_workflow_snapshots (task_tags);
CREATE INDEX IF NOT EXISTS idx_snapshots_inprogress
    ON sayiir_workflow_snapshots (task_priority, updated_at)
    WHERE status = 'InProgress';

-- sayiir_workflow_snapshot_history: add data_hash
ALTER TABLE sayiir_workflow_snapshot_history ADD COLUMN data_hash BLOB;

-- sayiir_workflow_tasks: add output column
ALTER TABLE sayiir_workflow_tasks ADD COLUMN output BLOB;

-- sayiir_task_claims → sayiir_workflow_claims: rename and change PK
-- SQLite: recreate table with new name and PK
CREATE TABLE sayiir_workflow_claims (
    instance_id TEXT        NOT NULL PRIMARY KEY,
    task_id     TEXT        NOT NULL,
    worker_id   TEXT        NOT NULL,
    claimed_at  TEXT        NOT NULL DEFAULT (datetime('now')),
    expires_at  TEXT
);

INSERT INTO sayiir_workflow_claims
SELECT instance_id, task_id, worker_id, claimed_at, expires_at
FROM sayiir_task_claims
GROUP BY instance_id;

DROP TABLE sayiir_task_claims;
