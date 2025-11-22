use once_cell::sync::Lazy;

// Keep a small module root here; implementations live in submodules for maintainability.
// The global runtime is exposed to submodules via `crate::db::TOKIO_RUNTIME`.
pub(crate) static TOKIO_RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime")
});

mod types;
mod sqlite;
mod postgres;

pub use types::{Table, Column, QueryResult, DatabaseClient};
pub use sqlite::SqliteClient;
pub use postgres::PostgresClient;
