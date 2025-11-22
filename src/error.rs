/// Error handling and custom error types for the application.
use std::fmt;

#[derive(Debug, Clone)]
pub enum AppError {
    DatabaseConnection(String),
    QueryExecution(String),
    FileDialog(String),
    TableFetch(String),
    InvalidInput(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DatabaseConnection(msg) => write!(f, "Database connection failed: {}", msg),
            AppError::QueryExecution(msg) => write!(f, "Query execution failed: {}", msg),
            AppError::FileDialog(msg) => write!(f, "File dialog error: {}", msg),
            AppError::TableFetch(msg) => write!(f, "Failed to fetch tables: {}", msg),
            AppError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

/// Result type for application operations
pub type AppResult<T> = Result<T, AppError>;
