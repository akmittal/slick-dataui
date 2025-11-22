use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Row};
use sqlx::Column as SqlxColumnTrait;
use super::TOKIO_RUNTIME;
use crate::db::{Table, Column, QueryResult, DatabaseClient};

pub struct SqliteClient {
    pool: Pool<Sqlite>,
}

impl SqliteClient {
    pub async fn new(url: &str) -> Result<Self> {
        let url = url.to_string();
        let pool = TOKIO_RUNTIME.spawn(async move {
            SqlitePoolOptions::new()
                .connect(&url)
                .await
        }).await??;
        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl DatabaseClient for SqliteClient {
    async fn get_tables(&self) -> Result<Vec<Table>> {
        let pool = self.pool.clone();
        TOKIO_RUNTIME.spawn(async move {
            let rows = sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
                .fetch_all(&pool)
                .await?;

            let tables = rows.into_iter().map(|row| Table {
                name: row.get(0),
                schema: None,
            }).collect();

            Ok(tables)
        }).await?
    }

    async fn get_columns(&self, table_name: &str) -> Result<Vec<Column>> {
        let pool = self.pool.clone();
        let table_name = table_name.to_string();
        TOKIO_RUNTIME.spawn(async move {
            let query = format!("PRAGMA table_info({})", table_name);
            let rows = sqlx::query(&query)
                .fetch_all(&pool)
                .await?;

            let columns = rows.into_iter().map(|row| Column {
                name: row.get("name"),
                data_type: row.get("type"),
                is_nullable: row.get::<i32, _>("notnull") == 0,
                is_primary_key: row.get::<i32, _>("pk") == 1,
            }).collect();

            Ok(columns)
        }).await?
    }

    async fn execute_query(&self, query: &str) -> Result<QueryResult> {
        let pool = self.pool.clone();
        let query = query.to_string();
        TOKIO_RUNTIME.spawn(async move {
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
        }).await?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection() {
        let result = TOKIO_RUNTIME.block_on(async {
            SqliteClient::new("sqlite::memory:").await
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_tables() {
        let client = TOKIO_RUNTIME.block_on(async {
            SqliteClient::new("sqlite::memory:").await.unwrap()
        });

        // Create a table
        TOKIO_RUNTIME.block_on(async {
            client.execute_query("CREATE TABLE test_table (id INTEGER PRIMARY KEY, name TEXT)").await.unwrap();
        });

        let tables = TOKIO_RUNTIME.block_on(async {
            client.get_tables().await.unwrap()
        });

        assert!(tables.iter().any(|t| t.name == "test_table"));
    }

    #[test]
    fn test_get_columns() {
        let client = TOKIO_RUNTIME.block_on(async {
            SqliteClient::new("sqlite::memory:").await.unwrap()
        });

        TOKIO_RUNTIME.block_on(async {
            client.execute_query("CREATE TABLE test_table (id INTEGER PRIMARY KEY, name TEXT)").await.unwrap();
        });

        let columns = TOKIO_RUNTIME.block_on(async {
            client.get_columns("test_table").await.unwrap()
        });

        assert_eq!(columns.len(), 2);
        assert!(columns.iter().any(|c| c.name == "id" && c.is_primary_key));
        assert!(columns.iter().any(|c| c.name == "name" && !c.is_primary_key));
    }

    #[test]
    fn test_execute_query() {
        let client = TOKIO_RUNTIME.block_on(async {
            SqliteClient::new("sqlite::memory:").await.unwrap()
        });

        TOKIO_RUNTIME.block_on(async {
            client.execute_query("CREATE TABLE test_table (id INTEGER PRIMARY KEY, name TEXT)").await.unwrap();
            client.execute_query("INSERT INTO test_table (name) VALUES ('Alice')").await.unwrap();
            client.execute_query("INSERT INTO test_table (name) VALUES ('Bob')").await.unwrap();
        });

        let result = TOKIO_RUNTIME.block_on(async {
            client.execute_query("SELECT * FROM test_table ORDER BY id").await.unwrap()
        });

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][1], "Alice");
        assert_eq!(result.rows[1][1], "Bob");
    }
}
