-- Performance/schema rework for MySQL
-- Similar to PostgreSQL but uses VARBINARY instead of BYTEA

-- sayiir_workflow_snapshots ---------------------------------------------------
-- Convert TEXT to VARBINARY for definition_hash and current_task_id
-- Add new columns: history_version, data_hash
-- Make data nullable
ALTER TABLE sayiir_workflow_snapshots
    MODIFY COLUMN definition_hash VARBINARY(255),
    MODIFY COLUMN current_task_id VARBINARY(32),
    MODIFY COLUMN data LONGBLOB,
    ADD COLUMN history_version INT NOT NULL DEFAULT 1,
    ADD COLUMN data_hash VARBINARY(32);

-- Update current_task_id to SHA2 hash if not NULL
UPDATE sayiir_workflow_snapshots
SET current_task_id = UNHEX(SHA2(current_task_id, 256))
WHERE current_task_id IS NOT NULL AND LENGTH(current_task_id) > 32;

UPDATE sayiir_workflow_snapshots s
SET history_version = COALESCE(
    (SELECT MAX(version) FROM sayiir_workflow_snapshot_history h
     WHERE h.instance_id = s.instance_id),
    0
);

-- sayiir_workflow_snapshot_history --------------------------------------------
ALTER TABLE sayiir_workflow_snapshot_history
    MODIFY COLUMN current_task_id VARBINARY(32),
    ADD COLUMN data_hash VARBINARY(32);

UPDATE sayiir_workflow_snapshot_history
SET current_task_id = UNHEX(SHA2(current_task_id, 256))
WHERE current_task_id IS NOT NULL AND LENGTH(current_task_id) > 32;

-- sayiir_workflow_tasks -------------------------------------------------------
ALTER TABLE sayiir_workflow_tasks
    MODIFY COLUMN task_id VARBINARY(32),
    ADD COLUMN output LONGBLOB;

UPDATE sayiir_workflow_tasks
SET task_id = UNHEX(SHA2(task_id, 256))
WHERE LENGTH(task_id) > 32;

-- Create index for in-progress workflows
CREATE INDEX idx_snapshots_inprogress
    ON sayiir_workflow_snapshots (task_priority, updated_at)
    WHERE status = 'InProgress';

-- sayiir_task_claims → sayiir_workflow_claims --------------------------------
RENAME TABLE sayiir_task_claims TO sayiir_workflow_claims;

ALTER TABLE sayiir_workflow_claims
    MODIFY COLUMN task_id VARBINARY(32);

UPDATE sayiir_workflow_claims
SET task_id = UNHEX(SHA2(task_id, 256))
WHERE LENGTH(task_id) > 32;

-- Change primary key from (instance_id, task_id) to (instance_id)
ALTER TABLE sayiir_workflow_claims DROP PRIMARY KEY;
ALTER TABLE sayiir_workflow_claims ADD PRIMARY KEY (instance_id);
