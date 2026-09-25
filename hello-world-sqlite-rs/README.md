# Hello World SQLite Example

Simple demonstration of using `sayiir-diesel` with SQLite for persistent workflow execution.

## Features

- Uses `DieselBackend` with in-memory SQLite
- Demonstrates `CheckpointingRunner` for durable execution
- Simple single-task workflow with persistence

## Running

```bash
cargo run
```

## What it does

1. Creates an in-memory SQLite backend
2. Defines a simple "greet" task
3. Runs the workflow with automatic checkpointing
4. Prints the final workflow status

## Compared to hello-world-rs

- **hello-world-rs**: Uses `run_once()` with no persistence (in-memory only)
- **hello-world-sqlite-rs**: Uses `CheckpointingRunner` with SQLite persistence

This example shows how to add crash recovery and persistence to workflows.
