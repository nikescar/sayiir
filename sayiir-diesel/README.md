# sayiir-diesel

Multi-database persistence backend for the [Sayiir](https://github.com/sayiir/sayiir) durable workflow engine.

## Overview

Provides [`DieselBackend`](https://docs.rs/sayiir-diesel/latest/sayiir_diesel/struct.DieselBackend.html), a production-grade implementation of `SnapshotStore` and `SignalStore` backed by PostgreSQL, MySQL, or SQLite via [diesel-async](https://crates.io/crates/diesel-async).

Choose your database backend with feature flags. All three backends share 100% schema parity and provide identical functionality with database-appropriate optimizations.

## Features

- **Codec-generic** — Serialize snapshots with any codec (JSON for debuggability, rkyv/bincode for speed). The data column is always `BYTEA`/`BLOB`.
- **ACID transactions** — Composite signal operations use proper locking for true atomicity.
- **Snapshot history** — Every checkpoint is appended to an immutable history table for debugging and auditing.
- **Observability-ready** — Indexed metadata columns (`status`, `current_task_id`, `completed_task_count`, `error`, timestamps) plus a denormalized `sayiir_workflow_tasks` table enable monitoring without deserializing blobs.
- **Priority scheduling** — `task_priority` column (1=Critical, 3=Normal, 5=Minimal) for priority-based task dispatch.
- **Tag-based routing** — `task_tags` array/JSON for worker affinity and capability-based routing.
- **Distributed tracing** — `trace_parent` column for OpenTelemetry trace context propagation.
- **Performance optimized** — BYTEA/VARBINARY IDs (PostgreSQL/MySQL), partial indexes, GIN indexes, optimized dispatch queries.
- **Multi-database** — One crate, three backends with identical APIs.

## Quick Start

### SQLite (Development)

```rust
use sayiir_diesel::DieselBackend;
use sayiir_persistence::SnapshotStore;
use sayiir_core::snapshot::WorkflowSnapshot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = DieselBackend::new("sqlite::memory:").await?;
    
    let mut snapshot = WorkflowSnapshot::new(
        "order-123".to_string(),
        "hash-abc".to_string()
    );
    backend.save_snapshot(&mut snapshot).await?;
    
    let loaded = backend.load_snapshot("order-123").await?;
    Ok(())
}
```

**Cargo.toml:**
```toml
[dependencies]
sayiir-diesel = { version = "1.0", features = ["sqlite"] }
```

### PostgreSQL (Production)

```rust
use sayiir_diesel::DieselBackend;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = DieselBackend::new(
        "postgresql://user:pass@localhost/sayiir"
    ).await?;
    
    // Use SnapshotStore and SignalStore traits...
    Ok(())
}
```

**Cargo.toml:**
```toml
[dependencies]
sayiir-diesel = { version = "1.0", features = ["postgres"] }
```

**System requirements:** `libpq` (PostgreSQL client library)

### MySQL (Production)

```rust
use sayiir_diesel::DieselBackend;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = DieselBackend::new(
        "mysql://user:pass@localhost/sayiir"
    ).await?;
    
    // Use SnapshotStore and SignalStore traits...
    Ok(())
}
```

**Cargo.toml:**
```toml
[dependencies]
sayiir-diesel = { version = "1.0", features = ["mysql"] }
```

**System requirements:** `libmysqlclient` (MySQL client library)

## Schema

`DieselBackend::new()` runs migrations automatically. The schema consists of:

| Table | Purpose |
|---|---|
| `sayiir_workflow_snapshots` | Current snapshot per workflow instance (1:1) |
| `sayiir_workflow_snapshot_history` | Immutable append-only snapshot history (1:N) |
| `sayiir_workflow_tasks` | Denormalized task metadata for querying (1:N) |
| `sayiir_workflow_claims` | Distributed workflow claim tracking (1:1) |
| `sayiir_workflow_signals` | Cancel/pause signal state (N per instance) |
| `sayiir_workflow_events` | External event queue for signal delivery (N per instance) |

### Schema Parity

All three databases implement the **same logical schema** with 100% feature parity:

| Feature | PostgreSQL | MySQL | SQLite |
|---------|-----------|-------|--------|
| Binary IDs (BYTEA) | ✅ SHA-256 digest | ✅ SHA2() VARBINARY | ⚠️ TEXT (hex) |
| Task tags array | ✅ TEXT[] + GIN | ✅ JSON + index | ✅ JSON text + index |
| Partial indexes | ✅ WHERE clause | ⚠️ Full index (8.0.13+) | ✅ WHERE clause |
| UNIQUE history | ✅ | ✅ | ✅ |
| Priority scheduling | ✅ | ✅ | ✅ |
| Distributed tracing | ✅ | ✅ | ✅ |
| Observability columns | ✅ | ✅ | ✅ |

**Note:** SQLite keeps IDs as TEXT for simplicity (no SHA-256 digest). MySQL uses full indexes where partial indexes aren't supported before 8.0.13.

## Migrations

All migrations run automatically on first connection. The migration history:

1. **000001_initial** - Base tables
2. **000002_observability** - position_kind, delay_wake_at
3. **000004_trace_context** - trace_parent for distributed tracing
4. **000005_history_unique_version** - UNIQUE constraint on history
5. **000006_task_priority** - Priority scheduling column
6. **000007_task_tags** - Tag-based routing array/JSON
7. **000008_performance_optim** - ⚠️ **BREAKING**: BYTEA conversions, table rename
8. **000009_index_cleanup** - Drop unused indexes
9. **000010_dispatch_indexes** - Dispatch optimization indexes

### Migration 008 Breaking Changes

Migration 008 introduces **breaking schema changes** for PostgreSQL and MySQL:

**PostgreSQL:**
- `definition_hash`, `current_task_id`, `task_id` → BYTEA (SHA-256 digest)
- `sayiir_task_claims` → `sayiir_workflow_claims` (table renamed)
- Primary key: `(instance_id, task_id)` → `(instance_id)` only
- `data` column becomes nullable

**MySQL:**
- Same changes as PostgreSQL but using VARBINARY + SHA2()

**SQLite:**
- Table rename and PK change only (kept TEXT for IDs)

**⚠️ Before upgrading:**
- Drain all in-flight workflows
- Stop all workers
- Backup your database

See [MIGRATIONS.md](MIGRATIONS.md) for detailed migration documentation.

## Database Version Support

### PostgreSQL
**Minimum version:** PostgreSQL 13  
**Tested versions:** 13, 14, 15, 16, 17  
**Required extensions:** pgcrypto (auto-installed)

### MySQL
**Minimum version:** MySQL 5.7 or MariaDB 10.2  
**Tested versions:** MySQL 8.0, MariaDB 10.11  
**Notes:** Full partial index support requires MySQL 8.0.13+

### SQLite
**Minimum version:** SQLite 3.30.0  
**Tested versions:** 3.30+, 3.45+  
**Notes:** Partial indexes require 3.30+, JSON functions work best with 3.38+

## Connection Pooling

All backends use [deadpool](https://crates.io/crates/deadpool) for connection pooling with sensible defaults:

- **Max connections:** 10
- **Timeouts:** 5s wait, 10s create, 2s recycle
- **Automatic reconnection** on connection loss

Connection pools are created automatically by `DieselBackend::new()`.

## Database Selection Guide

### SQLite - Development & Testing
**Pros:**
- Zero setup, file-based or in-memory
- Perfect for tests and local development
- No external dependencies

**Cons:**
- Single writer (not suitable for distributed workers)
- No true concurrency for writes

**Use when:** Testing, local development, single-process applications

### PostgreSQL - Production Default
**Pros:**
- Best performance and scalability
- Full feature set (BYTEA, TEXT[], partial indexes)
- MVCC for true concurrent reads/writes
- Battle-tested in production

**Cons:**
- Requires PostgreSQL installation
- More complex setup

**Use when:** Production deployments, distributed workers, high concurrency

### MySQL - Alternative Production
**Pros:**
- Widely available and supported
- Good performance with proper tuning
- Compatible with cloud MySQL services (AWS RDS, Google Cloud SQL)

**Cons:**
- Some feature adaptations (JSON instead of arrays)
- Requires MySQL/MariaDB installation

**Use when:** Existing MySQL infrastructure, cloud MySQL managed services

## Performance Characteristics

### Write Performance
- **PostgreSQL:** Best (MVCC, HOT updates with fillfactor=80)
- **MySQL:** Good (InnoDB with proper tuning)
- **SQLite:** Moderate (single writer serialization)

### Read Performance
- **PostgreSQL:** Best (partial indexes, BYTEA efficiency)
- **MySQL:** Good (with proper indexes)
- **SQLite:** Good (simple queries), Moderate (complex joins)

### Concurrent Workers
- **PostgreSQL:** Excellent (100+ workers tested)
- **MySQL:** Good (50+ workers tested)
- **SQLite:** Limited (1 writer at a time)

## Testing

Run tests for each database:

```bash
# SQLite (no external dependencies)
cargo test --features sqlite

# PostgreSQL (requires libpq)
cargo test --features postgres

# MySQL (requires libmysqlclient)
cargo test --features mysql
```

## Comparison with sayiir-postgres

`sayiir-diesel` is the multi-database successor to `sayiir-postgres`:

| Feature | sayiir-postgres | sayiir-diesel |
|---------|----------------|---------------|
| PostgreSQL support | ✅ | ✅ |
| MySQL support | ❌ | ✅ |
| SQLite support | ❌ | ✅ |
| Schema parity | ✅ | ✅ (all 3 DBs) |
| Performance | Excellent | Excellent (PostgreSQL) |
| Migration system | sqlx | Embedded SQL |

**Migration path:** Both crates implement identical schemas. Switching is as simple as changing the crate dependency and connection string.

## Documentation

Full API docs are available on [docs.rs](https://docs.rs/sayiir-diesel).

See also:
- [MIGRATIONS.md](MIGRATIONS.md) - Detailed migration guide
- [examples/](examples/) - Usage examples

## License

MIT
