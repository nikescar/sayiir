use sayiir_openflow::extract::node::extract_all_node_tasks;

#[test]
fn test_extract_all_node_tasks() {
    let source = r#"
const prepareDoc = task("ingest:prepare", async (input: IngestInput): Promise<PreparedDoc> => {
    const result = await fetch(input.url);
    return { docId: "123", chunkCount: 5 };
});

const embedAndIndex = task("ingest:embed", async (input) => {
    return { success: true };
});
"#;

    let tasks = extract_all_node_tasks(source).unwrap();

    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].id, "ingest:prepare");
    assert_eq!(tasks[0].entry_point, "prepareDoc");
    assert!(tasks[0].source_code.contains("task("));
    assert_eq!(tasks[1].id, "ingest:embed");
    assert_eq!(tasks[1].entry_point, "embedAndIndex");
}
