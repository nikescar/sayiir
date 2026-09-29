# Hello World SQLite Example

Simple demonstration of using `sayiir-diesel` with SQLite for persistent workflow execution.

## Features

- Uses `DieselBackend` with file-based SQLite (`workflow.db`)
- Demonstrates `CheckpointingRunner` for durable execution
- Simple single-task workflow with persistence
- Database file persists workflow history

## Running

```bash
# First run creates workflow.db
cargo run

# To run again, delete the database first
rm workflow.db && cargo run
```

## What it does

1. Creates a file-based SQLite backend (`workflow.db`)
2. Defines a simple "greet" task
3. Runs the workflow with automatic checkpointing
4. Uses timestamp-based instance IDs
5. Stores workflow state in the database

Each run creates a new workflow instance with a unique timestamp-based ID.

## Inspecting the database

```bash
sqlite3 workflow.db "SELECT instance_id, status FROM sayiir_workflow_snapshots;"
```

## Current Limitation

Migration tracking is not yet implemented in `sayiir-diesel`, so the database must be deleted between runs to avoid migration errors. In production, migrations would only run once on initial setup.

## Compared to hello-world-rs

- **hello-world-rs**: Uses `run_once()` with no persistence (in-memory only)
- **hello-world-sqlite-rs**: Uses `CheckpointingRunner` with SQLite persistence

This example shows how to add crash recovery and persistence to workflows.
