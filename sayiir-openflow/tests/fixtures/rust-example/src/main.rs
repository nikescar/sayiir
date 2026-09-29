use sayiir_runtime::prelude::*;

#[task(id = "greet")]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[task(id = "shout")]
pub fn shout(message: String) -> String {
    message.to_uppercase()
}

fn main() {
    let workflow = workflow! {
        name: "greeting-workflow",
        codec: JsonCodec,
        steps: [greet, shout]
    }
    .unwrap();

    println!("Workflow ready: {:?}", workflow.workflow_id());
}
