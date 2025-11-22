// QueryResultsDelegate - implements TableDelegate for displaying SQL query results
use gpui::{App, Window, IntoElement};
use gpui_component::table::{TableDelegate, Column};
use crate::db::QueryResult;

pub struct QueryResultsDelegate {
    results: QueryResult,
    columns: Vec<Column>,
}

impl QueryResultsDelegate {
    pub fn new(results: QueryResult) -> Self {
        // Create column definitions from query results
        let columns = results.columns.iter().map(|col_name| {
            Column::new(col_name, col_name)
                .width(150.)
                .sortable()
        }).collect();

        Self {
            results,
            columns,
        }
    }
}

impl TableDelegate for QueryResultsDelegate {
    fn columns_count(&self, _: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.results.rows.len()
    }

    fn column(&self, col_ix: usize, _: &App) -> &Column {
        &self.columns[col_ix]
    }

    fn render_td(&self, row_ix: usize, col_ix: usize, _: &mut Window, _: &mut App) -> impl IntoElement {
        if let Some(row) = self.results.rows.get(row_ix) {
            if let Some(cell) = row.get(col_ix) {
                return cell.clone();
            }
        }
        String::new()
    }
}
