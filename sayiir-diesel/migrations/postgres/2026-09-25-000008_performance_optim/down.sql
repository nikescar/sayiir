-- Reverse the breaking schema changes
-- WARNING: This is lossy for BYTEA→TEXT conversions

-- Reverse sayiir_workflow_claims → sayiir_task_claims
ALTER TABLE sayiir_workflow_claims DROP CONSTRAINT sayiir_workflow_claims_pkey;
ALTER TABLE sayiir_workflow_claims ADD PRIMARY KEY (instance_id, task_id);
ALTER TABLE sayiir_workflow_claims
    ALTER COLUMN task_id TYPE TEXT
        USING encode(task_id, 'hex');
ALTER TABLE sayiir_workflow_claims RENAME TO sayiir_task_claims;

DROP VIEW IF EXISTS sayiir_workflow_snapshots_hex;
DROP INDEX IF EXISTS idx_snapshots_inprogress;

ALTER TABLE sayiir_workflow_tasks
    DROP COLUMN IF EXISTS output,
    ALTER COLUMN task_id TYPE TEXT
        USING encode(task_id, 'hex');

ALTER TABLE sayiir_workflow_snapshot_history
    DROP COLUMN IF EXISTS data_hash,
    ALTER COLUMN current_task_id TYPE TEXT
        USING encode(current_task_id, 'hex');

ALTER TABLE sayiir_workflow_snapshots
    DROP COLUMN IF EXISTS data_hash,
    DROP COLUMN IF EXISTS history_version,
    ALTER COLUMN data SET NOT NULL,
    ALTER COLUMN current_task_id TYPE TEXT
        USING encode(current_task_id, 'hex'),
    ALTER COLUMN definition_hash TYPE TEXT
        USING encode(definition_hash, 'hex');
