DROP INDEX IF EXISTS idx_sayiir_task_tags;

-- SQLite table recreation to drop column
CREATE TABLE sayiir_workflow_snapshots_new (
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
    task_priority        INTEGER     NOT NULL DEFAULT 3
);

INSERT INTO sayiir_workflow_snapshots_new SELECT
    instance_id, status, definition_hash, current_task_id,
    completed_task_count, data, error, started_at, completed_at, updated_at,
    position_kind, delay_wake_at, trace_parent, task_priority
FROM sayiir_workflow_snapshots;

DROP TABLE sayiir_workflow_snapshots;
ALTER TABLE sayiir_workflow_snapshots_new RENAME TO sayiir_workflow_snapshots;
