pub mod rust;

#[cfg(feature = "python")]
pub mod python;

pub mod node;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TaskSource {
    pub id: String,
    pub source_code: String,
    pub entry_point: String,
}

pub struct TaskRegistry {
    tasks: HashMap<String, TaskSource>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    pub fn insert(&mut self, task: TaskSource) {
        self.tasks.insert(task.id.clone(), task);
    }

    pub fn get(&self, task_id: &str) -> Option<&TaskSource> {
        self.tasks.get(task_id)
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_registry() {
        let mut registry = TaskRegistry::new();

        registry.insert(TaskSource {
            id: "task1".to_string(),
            entry_point: "run".to_string(),
            source_code: "fn run() {}".to_string(),
        });

        registry.insert(TaskSource {
            id: "task2".to_string(),
            entry_point: "execute".to_string(),
            source_code: "fn execute() {}".to_string(),
        });

        assert_eq!(registry.len(), 2);
        assert!(registry.get("task1").is_some());
        assert!(registry.get("task2").is_some());
        assert!(registry.get("task3").is_none());

        // Overwrite test
        registry.insert(TaskSource {
            id: "task1".to_string(),
            entry_point: "new_run".to_string(),
            source_code: "fn new_run() {}".to_string(),
        });

        assert_eq!(registry.len(), 2);
        assert_eq!(registry.get("task1").unwrap().entry_point, "new_run");
    }
}
