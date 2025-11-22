// QueryResultsDelegate - implements TableDelegate for displaying SQL query results with sorting and pagination
use gpui::{App, Window, IntoElement, Context};
use gpui_component::table::{TableDelegate, Column, ColumnSort, TableState};
use crate::db::QueryResult;

pub struct QueryResultsDelegate {
    pub results: QueryResult,
    pub columns: Vec<Column>,
    pub current_sort_col: Option<usize>,
    pub current_sort_order: ColumnSort,
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
            current_sort_col: None,
            current_sort_order: ColumnSort::Default,
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

    fn perform_sort(&mut self, col_ix: usize, sort: ColumnSort, _: &mut Window, _: &mut Context<TableState<Self>>) {
        self.current_sort_col = Some(col_ix);
        self.current_sort_order = sort;

        match sort {
            ColumnSort::Ascending => {
                self.results.rows.sort_by(|a, b| {
                    if let (Some(a_val), Some(b_val)) = (a.get(col_ix), b.get(col_ix)) {
                        a_val.cmp(b_val)
                    } else {
                        std::cmp::Ordering::Equal
                    }
                });
            }
            ColumnSort::Descending => {
                self.results.rows.sort_by(|a, b| {
                    if let (Some(a_val), Some(b_val)) = (a.get(col_ix), b.get(col_ix)) {
                        b_val.cmp(a_val)
                    } else {
                        std::cmp::Ordering::Equal
                    }
                });
            }
            ColumnSort::Default => {
                // Reset to original order (no sort applied)
            }
        }
    }
}
