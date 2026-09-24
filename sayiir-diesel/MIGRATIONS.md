# Sayiir-Diesel Migrations

All migrations from sayiir-postgres have been ported to sayiir-diesel for PostgreSQL, MySQL, and SQLite.

## Migration List

### 000001_initial
- **Tables**: sayiir_workflow_snapshots, sayiir_workflow_snapshot_history, sayiir_workflow_tasks, sayiir_workflow_signals, sayiir_task_claims, sayiir_workflow_events
- **Status**: ✅ Implemented for all databases

### 000002_observability  
- **Added columns**: `position_kind`, `delay_wake_at` to snapshots
- **Purpose**: Dashboard observability without blob deserialization
- **Status**: ✅ Implemented for all databases
- **Database notes**:
  - PostgreSQL: TIMESTAMPTZ for delay_wake_at
  - MySQL: DATETIME(6) for delay_wake_at
  - SQLite: TEXT for delay_wake_at (ISO 8601)

### 000004_trace_context
- **Added columns**: `trace_parent` to snapshots
- **Purpose**: Distributed tracing support
- **Status**: ✅ Implemented for all databases

### 000005_history_unique_version
- **Changes**: Made `idx_history_instance` UNIQUE
- **Purpose**: Guarantee monotonic history without row locks
- **Status**: ✅ Implemented for all databases

### 000006_task_priority
- **Added columns**: `task_priority` SMALLINT DEFAULT 3
- **Purpose**: Priority-based task scheduling
- **Priorities**: 1=Critical, 3=Normal, 5=Minimal
- **Status**: ✅ Implemented for all databases
- **Database notes**:
  - PostgreSQL/MySQL: SMALLINT
  - SQLite: INTEGER

### 000007_task_tags
- **Added columns**: `task_tags` for worker affinity routing
- **Indexes**: GIN index (PostgreSQL), regular index (others)
- **Status**: ✅ Implemented for all databases
- **Database notes**:
  - PostgreSQL: TEXT[] array with GIN index
  - MySQL: JSON with functional index
  - SQLite: TEXT (JSON string) with regular index

### 000008_performance_optim (BREAKING)
- **BREAKING CHANGES**: Major schema rework, requires draining in-flight workflows
- **Column type changes**:
  - PostgreSQL: TEXT → BYTEA for definition_hash, current_task_id, task_id (SHA-256 digest)
  - MySQL: TEXT → VARBINARY for same columns
  - SQLite: Kept as TEXT (no BYTEA equivalent)
- **Column additions**:
  - `history_version` INT (snapshot local counter)
  - `data_hash` BYTEA/VARBINARY/BLOB
  - `output` BYTEA/VARBINARY/BLOB (tasks table)
- **Column modifications**:
  - `data` column becomes nullable (history is canonical store)
- **Table rename**: `sayiir_task_claims` → `sayiir_workflow_claims`
- **Primary key change**: `(instance_id, task_id)` → `(instance_id)` only
- **New objects**:
  - PostgreSQL: `sayiir_workflow_snapshots_hex` view for ops
  - PostgreSQL: pgcrypto extension
  - All: `idx_snapshots_inprogress` partial index
- **Status**: ✅ Implemented for all databases

### 000009_index_cleanup
- **Dropped indexes**:
  - `idx_snapshots_status` (superseded by partial idx_snapshots_inprogress)
  - `idx_snapshots_task` (redundant with PK)
  - `idx_snapshots_updated` (high write churn, no read benefit)
  - `idx_snapshots_position` (low cardinality, redundant)
  - `idx_tasks_status` (not used in read paths)
  - `idx_claims_expires` (not used in queries)
  - `idx_claims_worker` (not used in queries)
- **Index modifications**:
  - Made `idx_sayiir_task_tags` partial WHERE status='InProgress' (PostgreSQL/SQLite)
  - MySQL: kept full index (no partial index support before 8.0.13)
- **Table tuning**:
  - PostgreSQL: SET fillfactor=80 on snapshots (reduce page splits)
- **Status**: ✅ Implemented for all databases

### 000010_dispatch_indexes
- **Added indexes**: `idx_snapshots_inprogress_updated` on updated_at WHERE status='InProgress'
- **Purpose**: Support two-arm dispatch (priority + oldest-first)
- **Status**: ✅ Implemented for all databases
- **Database notes**:
  - PostgreSQL/SQLite: Partial index with WHERE clause
  - MySQL: Full index (no WHERE support before 8.0.13)

## Database-Specific Adaptations

### PostgreSQL
- Full feature support including:
  - BYTEA for binary data
  - TEXT[] arrays with GIN indexes
  - Partial indexes with WHERE clauses
  - pgcrypto extension for SHA-256
  - TIMESTAMPTZ for timestamps
  - Views for hex rendering

### MySQL
- Adaptations:
  - VARBINARY instead of BYTEA
  - JSON instead of TEXT[] arrays
  - Functional indexes on JSON
  - DATETIME(6) instead of TIMESTAMPTZ
  - SHA2() function for hashing
  - Full indexes where partial not supported

### SQLite
- Adaptations:
  - BLOB instead of BYTEA
  - TEXT (JSON string) instead of arrays
  - TEXT (ISO 8601) for timestamps
  - Table recreation for DROP COLUMN (before 3.35.0)
  - Partial indexes supported (3.30+)
  - Kept TEXT for IDs (no SHA-256 digest, simpler)

## Migration Execution

All migrations run automatically on `DieselBackend::new()`:

```rust
let backend = DieselBackend::new("postgresql://...").await?;
```

Migrations are executed in order:
1. 000001_initial
2. 000002_observability
3. 000004_trace_context
4. 000005_history_unique_version
5. 000006_task_priority
6. 000007_task_tags
7. 000008_performance_optim (BREAKING)
8. 000009_index_cleanup
9. 000010_dispatch_indexes

## Migration 008 Breaking Change Notice

⚠️ **IMPORTANT**: Migration 008 is a BREAKING schema change.

**Before running:**
- Drain all in-flight workflows
- Stop all workers
- For PostgreSQL: Truncate or dedupe `sayiir_task_claims` if multiple claims per instance exist

**Changes:**
- ID columns change from TEXT to BYTEA/VARBINARY (PostgreSQL/MySQL)
- Table renamed: `sayiir_task_claims` → `sayiir_workflow_claims`
- Primary key narrowed: `(instance_id, task_id)` → `(instance_id)`
- `data` column becomes nullable
- Pre-migration history rows will have NULL `data_hash`

**Rollback:**
- Down migrations provided but are LOSSY for BYTEA→TEXT conversions
- Backup database before running migration 008

## Schema Parity with sayiir-postgres

✅ sayiir-diesel now has **full schema parity** with sayiir-postgres migration 010.

All features supported:
- Priority-based task scheduling
- Tag-based worker affinity routing
- Distributed tracing (trace_parent)
- Observability (position_kind, delay_wake_at)
- Performance optimizations (BYTEA IDs, partial indexes)
- Workflow claims (renamed table, narrowed PK)
- Task output storage (separate from snapshot blob)

## Testing

All migrations tested with SQLite feature:
```bash
cd sayiir-diesel && cargo test --features sqlite
```

PostgreSQL and MySQL require system libraries:
- PostgreSQL: libpq
- MySQL: libmysqlclient

## Files

### PostgreSQL
- migrations/postgres/2026-09-25-000001_initial/
- migrations/postgres/2026-09-25-000002_observability/
- migrations/postgres/2026-09-25-000004_trace_context/
- migrations/postgres/2026-09-25-000005_history_unique_version/
- migrations/postgres/2026-09-25-000006_task_priority/
- migrations/postgres/2026-09-25-000007_task_tags/
- migrations/postgres/2026-09-25-000008_performance_optim/
- migrations/postgres/2026-09-25-000009_index_cleanup/
- migrations/postgres/2026-09-25-000010_dispatch_indexes/

### MySQL
- migrations/mysql/2026-09-25-000001_initial/
- migrations/mysql/2026-09-25-000002_observability/
- migrations/mysql/2026-09-25-000004_trace_context/
- migrations/mysql/2026-09-25-000005_history_unique_version/
- migrations/mysql/2026-09-25-000006_task_priority/
- migrations/mysql/2026-09-25-000007_task_tags/
- migrations/mysql/2026-09-25-000008_performance_optim/
- migrations/mysql/2026-09-25-000009_index_cleanup/
- migrations/mysql/2026-09-25-000010_dispatch_indexes/

### SQLite
- migrations/sqlite/2026-09-25-000001_initial/
- migrations/sqlite/2026-09-25-000002_observability/
- migrations/sqlite/2026-09-25-000004_trace_context/
- migrations/sqlite/2026-09-25-000005_history_unique_version/
- migrations/sqlite/2026-09-25-000006_task_priority/
- migrations/sqlite/2026-09-25-000007_task_tags/
- migrations/sqlite/2026-09-25-000008_performance_optim/
- migrations/sqlite/2026-09-25-000009_index_cleanup/
- migrations/sqlite/2026-09-25-000010_dispatch_indexes/
