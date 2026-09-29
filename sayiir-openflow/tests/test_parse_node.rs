use sayiir_openflow::parse::node::parse_node_workflow;

#[test]
fn test_parse_node_workflow() {
    let source = r#"
export function buildIngestWorkflow(ctx: RagContext): Workflow<IngestInput, IngestOutput> {
    return flow<IngestInput, IngestOutput>("ingest-workflow")
        .then(prepareDoc)
        .then(embedAndIndex)
        .build();
}
"#;

    let wf = parse_node_workflow(source).unwrap();

    assert_eq!(wf.name, "ingest-workflow");
    assert_eq!(wf.task_names, vec!["prepareDoc", "embedAndIndex"]);
}
