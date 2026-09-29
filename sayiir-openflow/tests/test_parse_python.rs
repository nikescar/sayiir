use sayiir_openflow::parse::python::parse_python_workflow;

#[test]
fn test_parse_python_workflow() {
    let source = r#"
workflow = (
    Flow("expense-approval")
    .then(submit_expense)
    .wait_for_signal("approval", timeout=timedelta(hours=48))
    .then(process_approved)
    .build()
)
"#;

    let wf = parse_python_workflow(source).unwrap();

    assert_eq!(wf.name, "expense-approval");
    assert_eq!(wf.task_names, vec!["submit_expense", "process_approved"]);
}
