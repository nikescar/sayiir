use sayiir_openflow::extract::python::extract_all_python_tasks;

#[test]
fn test_extract_all_python_tasks() {
    let source = r#"
@task
def submit_expense(expense: dict) -> dict:
    print(f"Expense submitted")
    return {**expense, "status": "pending"}

@task
def process_approved(approval: dict) -> str:
    return f"Approved"

def not_a_task():
    pass
"#;

    let tasks = extract_all_python_tasks(source).unwrap();

    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].id, "submit_expense");
    assert_eq!(tasks[0].entry_point, "submit_expense");
    assert!(tasks[0].source_code.contains("@task"));
    assert_eq!(tasks[1].id, "process_approved");
}
