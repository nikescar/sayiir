-- Add position_kind for queryable workflow position without blob deserialization.
ALTER TABLE sayiir_workflow_snapshots ADD COLUMN position_kind TEXT;

CREATE INDEX idx_snapshots_position ON sayiir_workflow_snapshots (position_kind(255));

-- Add delay_wake_at so dashboards can show when parked workflows will resume.
ALTER TABLE sayiir_workflow_snapshots ADD COLUMN delay_wake_at DATETIME(6);
