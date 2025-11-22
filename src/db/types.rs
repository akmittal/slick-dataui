use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub schema: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub trait DatabaseClient: Send + Sync {
    async fn get_tables(&self) -> Result<Vec<Table>>;
    async fn get_columns(&self, table_name: &str) -> Result<Vec<Column>>;
    async fn execute_query(&self, query: &str) -> Result<QueryResult>;
}
