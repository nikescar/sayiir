-- Index cleanup for SQLite

-- Drop unused indexes
DROP INDEX IF EXISTS idx_snapshots_status;
DROP INDEX IF EXISTS idx_snapshots_task;
DROP INDEX IF EXISTS idx_snapshots_updated;
DROP INDEX IF EXISTS idx_snapshots_position;
DROP INDEX IF EXISTS idx_tasks_status;
DROP INDEX IF EXISTS idx_claims_expires;
DROP INDEX IF EXISTS idx_claims_worker;

-- Make task_tags partial (SQLite 3.30+ supports partial indexes)
DROP INDEX IF EXISTS idx_sayiir_task_tags;
CREATE INDEX idx_sayiir_task_tags
    ON sayiir_workflow_snapshots (task_tags)
    WHERE status = 'InProgress';
