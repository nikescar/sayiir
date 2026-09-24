//! Diesel persistence backend for Sayiir workflow engine
//!
//! Supports SQLite, PostgreSQL, and MySQL via feature flags.
//!
//! # Features
//! - `sqlite` - SQLite backend (via diesel-async)
//! - `postgres` - PostgreSQL backend (via diesel-async)
//! - `mysql` - MySQL backend (via diesel-async)
//!
//! # Example
//! ```no_run
//! use sayiir_diesel::DieselBackend;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let backend = DieselBackend::new("sqlite::memory:").await?;
//!     Ok(())
//! }
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs)]

use sayiir_persistence::{PersistentBackend, SnapshotStore, SignalStore};
use thiserror::Error;

#[allow(missing_docs)]
mod schema;

pub use schema::*;

/// Diesel backend errors
#[derive(Error, Debug)]
pub enum DieselError {
    /// Database connection error
    #[error("database connection failed: {0}")]
    ConnectionError(String),

    /// Migration error
    #[error("migration failed: {0}")]
    MigrationError(String),

    /// Query execution error
    #[error("query failed: {0}")]
    QueryError(String),
}

/// Result type for Diesel backend operations
pub type Result<T> = std::result::Result<T, DieselError>;

/// Diesel backend for sayiir persistence
pub struct DieselBackend {
    // Will be implemented in later tasks
}

impl DieselBackend {
    /// Create a new Diesel backend
    ///
    /// # Arguments
    /// - `database_url` - Database connection string (e.g., "sqlite::memory:")
    pub async fn new(_database_url: &str) -> Result<Self> {
        todo!("Implement in Task 1.2")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        // Placeholder test to make cargo test pass
        assert!(true);
    }
}
