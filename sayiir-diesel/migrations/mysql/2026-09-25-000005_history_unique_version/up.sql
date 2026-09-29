-- MySQL: Replace non-unique index with unique index
DROP INDEX idx_history_instance ON sayiir_workflow_snapshot_history;
CREATE UNIQUE INDEX idx_history_instance ON sayiir_workflow_snapshot_history (instance_id, version);
