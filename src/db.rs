use once_cell::sync::Lazy;

// Keep a small module root here; implementations live in submodules for maintainability.
// The global runtime is exposed to submodules via `crate::db::TOKIO_RUNTIME`.
pub(crate) static TOKIO_RUNTIME: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));

mod postgres;
mod sqlite;
mod types;

pub use postgres::PostgresClient;
pub use sqlite::SqliteClient;
pub use types::{Column, DatabaseClient, QueryResult, Table};
