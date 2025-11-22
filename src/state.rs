use gpui::prelude::*;
use gpui::{AppContext, Context, Render, IntoElement, div, Window, Entity};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::db::{DatabaseClient, SqliteClient, PostgresClient, Table, QueryResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DatabaseType {
    Sqlite,
    Postgres,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConnectionConfig {
    pub name: String,
    pub db_type: DatabaseType,
    pub connection_string: String,
}

pub struct AppState {
    pub connections: Vec<ConnectionConfig>,
    pub active_connection: Option<Arc<dyn DatabaseClient>>,
    pub active_connection_name: Option<String>,
    pub tables: Vec<Table>,
    pub query_results: Option<QueryResult>,
    pub current_query: String,
    pub is_connecting: bool,
    pub error_message: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            connections: vec![],
            active_connection: None,
            active_connection_name: None,
            tables: vec![],
            query_results: None,
            current_query: String::new(),
            is_connecting: false,
            error_message: None,
        }
    }

    pub fn toggle_connecting(&mut self, _cx: &mut Context<Self>) {
        self.is_connecting = !self.is_connecting;
    }
}

impl Render for AppState {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div() // Invisible
    }
}

pub struct GlobalAppState(pub Entity<AppState>);

impl GlobalAppState {
    pub fn new(entity: Entity<AppState>) -> Self {
        Self(entity)
    }
}

pub enum AppAction {
    Connect(ConnectionConfig),
    SetTables(Vec<Table>),
    SetQueryResult(QueryResult),
    SetError(String),
}
