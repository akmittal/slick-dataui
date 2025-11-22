use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, sqlite::SqlitePoolOptions, Pool, Postgres, Sqlite, Row, Column as SqlxColumn};
use std::sync::Arc;
use once_cell::sync::Lazy;

// Global Tokio runtime for database operations
static TOKIO_RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime")
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub schema: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[async_trait::async_trait]
pub trait DatabaseClient: Send + Sync {
    async fn get_tables(&self) -> Result<Vec<Table>>;
    async fn get_columns(&self, table_name: &str) -> Result<Vec<Column>>;
    async fn execute_query(&self, query: &str) -> Result<QueryResult>;
}

pub struct SqliteClient {
    pool: Pool<Sqlite>,
}

impl SqliteClient {
    pub async fn new(url: &str) -> Result<Self> {
        // Use global Tokio runtime for SQLx operations
        let url = url.to_string();
        let pool = TOKIO_RUNTIME.block_on(async move {
            SqlitePoolOptions::new()
                .connect(&url)
                .await
        })?;
        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl DatabaseClient for SqliteClient {
    async fn get_tables(&self) -> Result<Vec<Table>> {
        // Execute within Tokio runtime context
        let pool = self.pool.clone();
        TOKIO_RUNTIME.block_on(async move {
            let rows = sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
                .fetch_all(&pool)
                .await?;

            let tables = rows.into_iter().map(|row| {
                Table {
                    name: row.get(0),
                    schema: None,
                }
            }).collect();

            Ok(tables)
        })
    }

    async fn get_columns(&self, table_name: &str) -> Result<Vec<Column>> {
        let pool = self.pool.clone();
        let table_name = table_name.to_string();
        TOKIO_RUNTIME.block_on(async move {
            let query = format!("PRAGMA table_info({})", table_name);
            let rows = sqlx::query(&query)
                .fetch_all(&pool)
                .await?;

            let columns = rows.into_iter().map(|row| {
                Column {
                    name: row.get("name"),
                    data_type: row.get("type"),
                    is_nullable: row.get::<i32, _>("notnull") == 0,
                    is_primary_key: row.get::<i32, _>("pk") == 1,
                }
            }).collect();

            Ok(columns)
        })
    }

    async fn execute_query(&self, query: &str) -> Result<QueryResult> {
        let pool = self.pool.clone();
        let query = query.to_string();
        TOKIO_RUNTIME.block_on(async move {
            let rows = sqlx::query(&query)
                .fetch_all(&pool)
                .await?;

            if rows.is_empty() {
                return Ok(QueryResult { columns: vec![], rows: vec![] });
            }

            let columns: Vec<String> = rows[0].columns().iter().map(|c| c.name().to_string()).collect();
            let mut result_rows = Vec::new();

            for row in rows {
                let mut current_row = Vec::new();
                for (i, _) in columns.iter().enumerate() {
                    let val: String = row.try_get(i).unwrap_or_else(|_| "NULL".to_string()); 
                     current_row.push(val);
                }
                result_rows.push(current_row);
            }

            Ok(QueryResult { columns, rows: result_rows })
        })
    }
}

pub struct PostgresClient {
    pool: Pool<Postgres>,
}

impl PostgresClient {
    pub async fn new(url: &str) -> Result<Self> {
        // Use global Tokio runtime for SQLx operations
        let url = url.to_string();
        let pool = TOKIO_RUNTIME.block_on(async move {
            PgPoolOptions::new()
                .connect(&url)
                .await
        })?;
        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl DatabaseClient for PostgresClient {
    async fn get_tables(&self) -> Result<Vec<Table>> {
        let pool = self.pool.clone();
        TOKIO_RUNTIME.block_on(async move {
            let rows = sqlx::query("SELECT tablename FROM pg_catalog.pg_tables WHERE schemaname != 'pg_catalog' AND schemaname != 'information_schema'")
                .fetch_all(&pool)
                .await?;

            let tables = rows.into_iter().map(|row| {
                Table {
                    name: row.get(0),
                    schema: Some("public".to_string()), // Simplified
                }
            }).collect();

            Ok(tables)
        })
    }

    async fn get_columns(&self, table_name: &str) -> Result<Vec<Column>> {
        let pool = self.pool.clone();
        let table_name = table_name.to_string();
        TOKIO_RUNTIME.block_on(async move {
            let rows = sqlx::query("SELECT column_name, data_type, is_nullable FROM information_schema.columns WHERE table_name = $1")
                .bind(&table_name)
                .fetch_all(&pool)
                .await?;

            let columns = rows.into_iter().map(|row| {
                Column {
                    name: row.get("column_name"),
                    data_type: row.get("data_type"),
                    is_nullable: row.get::<String, _>("is_nullable") == "YES",
                    is_primary_key: false, // Need more complex query for PK
                }
            }).collect();

            Ok(columns)
        })
    }

    async fn execute_query(&self, query: &str) -> Result<QueryResult> {
        let pool = self.pool.clone();
        let query = query.to_string();
        TOKIO_RUNTIME.block_on(async move {
            let rows = sqlx::query(&query)
                .fetch_all(&pool)
                .await?;

            if rows.is_empty() {
                return Ok(QueryResult { columns: vec![], rows: vec![] });
            }

            let columns: Vec<String> = rows[0].columns().iter().map(|c| c.name().to_string()).collect();
            let mut result_rows = Vec::new();

            for row in rows {
                let mut current_row = Vec::new();
                for (i, _) in columns.iter().enumerate() {
                     // Simplified string conversion
                     // In real world we need to handle types
                     let val: String = row.try_get(i).unwrap_or_else(|_| "NULL".to_string());
                     current_row.push(val);
                }
                result_rows.push(current_row);
            }

            Ok(QueryResult { columns, rows: result_rows })
        })
    }
}
