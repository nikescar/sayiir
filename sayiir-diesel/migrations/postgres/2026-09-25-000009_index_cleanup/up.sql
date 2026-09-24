-- Index cleanup + heap-page tuning. Pure performance migration; no
-- behaviour change. Identified via a static audit of every read path in
-- src/: the dropped indexes are not referenced by any query, and were
-- either superseded by the partial idx_snapshots_inprogress (008) or
-- left over from the 001 schema before the PK on instance_id made
-- single-column secondary indexes redundant.

-- sayiir_workflow_snapshots --------------------------------------------------
-- Every save_snapshot UPDATE writes this row. Each surviving secondary
-- index on it is amortised across all writes to the table.
DROP INDEX IF EXISTS idx_snapshots_status;
DROP INDEX IF EXISTS idx_snapshots_task;
DROP INDEX IF EXISTS idx_snapshots_updated;
DROP INDEX IF EXISTS idx_snapshots_position;

-- Make task_tags GIN partial. Terminal rows are never matched by
-- find_available_tasks' tag filter, but the old non-partial GIN
-- indexed them anyway — pure write amplification for the lifetime of
-- the workflow's row.
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM pg_index i
        JOIN pg_class c ON c.oid = i.indexrelid
        JOIN pg_class t ON t.oid = i.indrelid
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = current_schema()
          AND t.relname = 'sayiir_workflow_snapshots'
          AND c.relname = 'idx_sayiir_task_tags'
          AND i.indpred IS NOT NULL
    ) THEN
        -- Already partial; no-op.
        NULL;
    ELSE
        EXECUTE 'DROP INDEX IF EXISTS idx_sayiir_task_tags';
        EXECUTE $ddl$
            CREATE INDEX idx_sayiir_task_tags
                ON sayiir_workflow_snapshots USING GIN (task_tags)
                WHERE status = 'InProgress'
        $ddl$;
    END IF;
END $$;

-- Leave 20% free space on each heap page so future row-version writes
-- have somewhere to land in-page. Reduces page splits on updates that
-- grow the row (e.g. task_tags TEXT[] gaining an entry, error becoming
-- non-NULL on failure).
ALTER TABLE sayiir_workflow_snapshots SET (fillfactor = 80);

-- sayiir_workflow_tasks ------------------------------------------------------
-- idx_tasks_status: not referenced by any read path.
DROP INDEX IF EXISTS idx_tasks_status;

-- sayiir_workflow_claims (formerly sayiir_task_claims) -----------------------
-- Neither expires_at nor worker_id is the driver of any query in src/.
DROP INDEX IF EXISTS idx_claims_expires;
DROP INDEX IF EXISTS idx_claims_worker;
