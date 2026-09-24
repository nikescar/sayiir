-- Dispatch indexes for MySQL
-- MySQL doesn't support WHERE clauses in indexes before 8.0.13
-- Use a regular index on updated_at
CREATE INDEX idx_snapshots_inprogress_updated
    ON sayiir_workflow_snapshots (updated_at);
