-- SQLite: Replace non-unique index with unique index
DROP INDEX IF EXISTS idx_history_instance;
CREATE UNIQUE INDEX idx_history_instance ON sayiir_workflow_snapshot_history (instance_id, version);
