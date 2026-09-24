-- Restore dropped indexes

CREATE INDEX IF NOT EXISTS idx_claims_worker ON sayiir_workflow_claims (worker_id);
CREATE INDEX IF NOT EXISTS idx_claims_expires ON sayiir_workflow_claims (expires_at);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON sayiir_workflow_tasks (status);
CREATE INDEX IF NOT EXISTS idx_snapshots_position ON sayiir_workflow_snapshots (position_kind);
CREATE INDEX IF NOT EXISTS idx_snapshots_updated ON sayiir_workflow_snapshots (updated_at);
CREATE INDEX IF NOT EXISTS idx_snapshots_task ON sayiir_workflow_snapshots (current_task_id);
CREATE INDEX IF NOT EXISTS idx_snapshots_status ON sayiir_workflow_snapshots (status);

-- Restore non-partial index
DROP INDEX IF EXISTS idx_sayiir_task_tags;
CREATE INDEX idx_sayiir_task_tags ON sayiir_workflow_snapshots (task_tags);
