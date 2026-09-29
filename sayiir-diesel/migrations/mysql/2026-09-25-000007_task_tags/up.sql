-- Add task tags column to support tag-based worker affinity routing.
-- MySQL: Use JSON for array storage
ALTER TABLE sayiir_workflow_snapshots
ADD COLUMN task_tags JSON NOT NULL DEFAULT (JSON_ARRAY());

-- MySQL 5.7+ supports JSON, create index on virtual column
-- Note: Direct JSON indexing requires MySQL 8.0.13+, we'll index the full column
CREATE INDEX idx_sayiir_task_tags
ON sayiir_workflow_snapshots ((CAST(task_tags AS CHAR(1000) ARRAY)));
