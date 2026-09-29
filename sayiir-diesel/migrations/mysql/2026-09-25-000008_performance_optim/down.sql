-- Reverse migration 008 for MySQL
-- WARNING: This is lossy for VARBINARY→TEXT conversions

-- Reverse sayiir_workflow_claims changes
ALTER TABLE sayiir_workflow_claims DROP PRIMARY KEY;
ALTER TABLE sayiir_workflow_claims ADD PRIMARY KEY (instance_id, task_id);
ALTER TABLE sayiir_workflow_claims MODIFY COLUMN task_id TEXT;

RENAME TABLE sayiir_workflow_claims TO sayiir_task_claims;

DROP INDEX idx_snapshots_inprogress ON sayiir_workflow_snapshots;

-- Reverse sayiir_workflow_tasks changes
ALTER TABLE sayiir_workflow_tasks
    DROP COLUMN output,
    MODIFY COLUMN task_id TEXT;

-- Reverse sayiir_workflow_snapshot_history changes
ALTER TABLE sayiir_workflow_snapshot_history
    DROP COLUMN data_hash,
    MODIFY COLUMN current_task_id TEXT;

-- Reverse sayiir_workflow_snapshots changes
ALTER TABLE sayiir_workflow_snapshots
    DROP COLUMN data_hash,
    DROP COLUMN history_version,
    MODIFY COLUMN data LONGBLOB NOT NULL,
    MODIFY COLUMN current_task_id TEXT,
    MODIFY COLUMN definition_hash TEXT;
