pub mod rust;

#[cfg(feature = "python")]
pub mod python;

pub mod node;

#[derive(Debug, Clone)]
pub struct TaskSource {
    pub id: String,
    pub source_code: String,
    pub entry_point: String,
}
