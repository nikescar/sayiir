-- Index cleanup for MySQL

-- Drop unused indexes
DROP INDEX idx_snapshots_status ON sayiir_workflow_snapshots;
DROP INDEX idx_snapshots_task ON sayiir_workflow_snapshots;
DROP INDEX idx_snapshots_updated ON sayiir_workflow_snapshots;
DROP INDEX idx_snapshots_position ON sayiir_workflow_snapshots;
DROP INDEX idx_tasks_status ON sayiir_workflow_tasks;
DROP INDEX idx_claims_expires ON sayiir_workflow_claims;
DROP INDEX idx_claims_worker ON sayiir_workflow_claims;

-- MySQL doesn't support partial indexes directly, keep the full index
-- Or use a filtered index approach if MySQL 8.0.13+
