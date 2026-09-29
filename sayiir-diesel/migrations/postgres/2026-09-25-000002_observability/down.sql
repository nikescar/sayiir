DROP INDEX IF EXISTS idx_snapshots_position;
ALTER TABLE sayiir_workflow_snapshots DROP COLUMN IF EXISTS delay_wake_at;
ALTER TABLE sayiir_workflow_snapshots DROP COLUMN IF EXISTS position_kind;
