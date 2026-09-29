DROP INDEX idx_sayiir_task_tags ON sayiir_workflow_snapshots;
ALTER TABLE sayiir_workflow_snapshots DROP COLUMN task_tags;
