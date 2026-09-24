-- Dispatch indexes for SQLite
CREATE INDEX IF NOT EXISTS idx_snapshots_inprogress_updated
    ON sayiir_workflow_snapshots (updated_at)
    WHERE status = 'InProgress';
