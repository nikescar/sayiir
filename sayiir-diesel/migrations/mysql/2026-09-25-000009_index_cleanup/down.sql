-- Restore dropped indexes

CREATE INDEX idx_claims_worker ON sayiir_workflow_claims (worker_id);
CREATE INDEX idx_claims_expires ON sayiir_workflow_claims (expires_at);
CREATE INDEX idx_tasks_status ON sayiir_workflow_tasks (status);
CREATE INDEX idx_snapshots_position ON sayiir_workflow_snapshots (position_kind(255));
CREATE INDEX idx_snapshots_updated ON sayiir_workflow_snapshots (updated_at);
CREATE INDEX idx_snapshots_task ON sayiir_workflow_snapshots (current_task_id);
CREATE INDEX idx_snapshots_status ON sayiir_workflow_snapshots (status(255));
