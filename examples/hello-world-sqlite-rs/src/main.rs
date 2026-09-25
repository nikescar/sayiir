use sayiir_runtime::prelude::*;
use sayiir_core::error::BoxError;
use sayiir_diesel::DieselBackend;
use std::time::SystemTime;

#[task]
async fn greet(name: String) -> Result<String, BoxError> {
    Ok(format!("Hello, {}!", name))
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // Create SQLite backend with file-based database
    // On first run, creates workflow.db with schema
    // On subsequent runs, delete workflow.db first or the migrations will fail
    let backend = DieselBackend::new("workflow.db").await?;

    // Create checkpointing runner with the backend
    let runner = CheckpointingRunner::new(backend);

    let workflow = workflow! {
        name: "hello",
        steps: [greet]
    }
    .unwrap();

    // Run workflow with persistent backend and checkpointing
    // Use timestamp-based ID so each run creates a new workflow instance
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let instance_id = format!("hello-world-{}", timestamp);

    println!("Running workflow instance: {}", instance_id);
    let status = runner.run(&workflow, &instance_id, "World".to_string()).await?;

    println!("Workflow status: {:?}", status);
    println!("\nDatabase 'workflow.db' persists workflow history.");
    println!("Note: Delete workflow.db before running again (migration tracking not yet implemented).");

    Ok(())
}
