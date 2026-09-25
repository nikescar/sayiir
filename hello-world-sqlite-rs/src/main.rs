use sayiir_runtime::prelude::*;
use sayiir_core::error::BoxError;
use sayiir_diesel::DieselBackend;

#[task]
async fn greet(name: String) -> Result<String, BoxError> {
    Ok(format!("Hello, {}!", name))
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // Create SQLite backend (in-memory for demo)
    let backend = DieselBackend::new("sqlite::memory:").await?;

    // Create checkpointing runner with the backend
    let runner = CheckpointingRunner::new(backend);

    let workflow = workflow! {
        name: "hello",
        steps: [greet]
    }
    .unwrap();

    // Run workflow with persistent backend and checkpointing
    let instance_id = "hello-world-1";
    let status = runner.run(&workflow, instance_id, "World".to_string()).await?;

    println!("Workflow status: {:?}", status);
    Ok(())
}
