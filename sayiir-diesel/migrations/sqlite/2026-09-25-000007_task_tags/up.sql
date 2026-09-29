-- Add task tags column to support tag-based worker affinity routing.
-- SQLite: Use JSON text for array storage
ALTER TABLE sayiir_workflow_snapshots
ADD COLUMN task_tags TEXT NOT NULL DEFAULT '[]';

-- SQLite 3.38+ supports JSON functions, create index if supported
CREATE INDEX IF NOT EXISTS idx_sayiir_task_tags
ON sayiir_workflow_snapshots (task_tags);
