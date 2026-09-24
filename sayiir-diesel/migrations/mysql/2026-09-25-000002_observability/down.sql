DROP INDEX idx_snapshots_position ON sayiir_workflow_snapshots;
ALTER TABLE sayiir_workflow_snapshots DROP COLUMN delay_wake_at;
ALTER TABLE sayiir_workflow_snapshots DROP COLUMN position_kind;
