use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, Row};
use sqlx::Column as SqlxColumnTrait;
use super::TOKIO_RUNTIME;
use crate::db::{Table, Column, QueryResult, DatabaseClient};

pub struct PostgresClient {
    pool: Pool<Postgres>,
}

impl PostgresClient {
    pub async fn new(url: &str) -> Result<Self> {
        let url = url.to_string();
        let pool = TOKIO_RUNTIME.spawn(async move {
            PgPoolOptions::new()
                .connect(&url)
                .await
        }).await??;
        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl DatabaseClient for PostgresClient {
    async fn get_tables(&self) -> Result<Vec<Table>> {
        let pool = self.pool.clone();
        TOKIO_RUNTIME.spawn(async move {
            let rows = sqlx::query("SELECT tablename FROM pg_catalog.pg_tables WHERE schemaname != 'pg_catalog' AND schemaname != 'information_schema'")
                .fetch_all(&pool)
                .await?;

            let tables = rows.into_iter().map(|row| Table {
                name: row.get(0),
                schema: Some("public".to_string()),
            }).collect();

            Ok(tables)
        }).await?
    }

    async fn get_columns(&self, table_name: &str) -> Result<Vec<Column>> {
        let pool = self.pool.clone();
        let table_name = table_name.to_string();
        TOKIO_RUNTIME.spawn(async move {
            let rows = sqlx::query("SELECT column_name, data_type, is_nullable FROM information_schema.columns WHERE table_name = $1")
                .bind(&table_name)
                .fetch_all(&pool)
                .await?;

            let columns = rows.into_iter().map(|row| Column {
                name: row.get("column_name"),
                data_type: row.get("data_type"),
                is_nullable: row.get::<String, _>("is_nullable") == "YES",
                is_primary_key: false,
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
