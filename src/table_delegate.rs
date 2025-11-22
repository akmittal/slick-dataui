// QueryResultsDelegate - implements TableDelegate for displaying SQL query results with sorting and pagination
use crate::db::QueryResult;
use crate::state::GlobalAppState;
use gpui::{App, AsyncApp, Context, IntoElement, Window};
use gpui_component::table::{Column, ColumnSort, TableDelegate, TableState};

pub struct QueryResultsDelegate {
    pub results: QueryResult,
    pub columns: Vec<Column>,
    pub current_sort_col: Option<usize>,
    pub current_sort_order: ColumnSort,
    pub app_state: GlobalAppState,
}

impl QueryResultsDelegate {
    pub fn new(
        results: QueryResult,
        app_state: GlobalAppState,
        sort_column: Option<String>,
        sort_ascending: bool,
    ) -> Self {
        // Determine sort index and order
        let sort_col_index = sort_column
            .as_ref()
            .and_then(|sc| results.columns.iter().position(|c| c == sc));
        let sort_order = if sort_ascending {
            ColumnSort::Ascending
        } else {
            ColumnSort::Descending
        };

        // Create column definitions from query results
        let columns = results
            .columns
            .iter()
            .enumerate()
            .map(|(i, col_name)| {
                let mut col = Column::new(col_name, col_name).width(150.).sortable();
                if Some(i) == sort_col_index {
                    col = col.sort(sort_order);
                }
                col
            })
            .collect();

        Self {
            results,
            columns,
            current_sort_col: sort_col_index,
            current_sort_order: if sort_col_index.is_some() {
                sort_order
            } else {
                ColumnSort::Default
            },
            app_state,
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

    fn render_td(
        &self,
        row_ix: usize,
        col_ix: usize,
        _: &mut Window,
        _: &mut App,
    ) -> impl IntoElement {
        use gpui::prelude::*;
        use gpui::*;

        let content = if let Some(row) = self.results.rows.get(row_ix) {
            if let Some(cell) = row.get(col_ix) {
                cell.clone()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        div()
            .px_2()
            .py_1()
            .min_w(px(50.))
            .text_color(rgb(0xffffff))
            .child(content)
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) {
        // Store the sort state
        let column_name = if let Some(col_name) = self.results.columns.get(col_ix) {
            col_name.clone()
        } else {
            return;
        };

        // Determine sort direction and override Default to toggle based on current state
        let (sort, ascending) = match sort {
            ColumnSort::Ascending => (ColumnSort::Ascending, true),
            ColumnSort::Descending => (ColumnSort::Descending, false),
            ColumnSort::Default => {
                // If Default is sent, toggle based on current state
                if self.current_sort_col == Some(col_ix)
                    && self.current_sort_order == ColumnSort::Ascending
                {
                    (ColumnSort::Descending, false)
                } else {
                    (ColumnSort::Ascending, true)
                }
            }
        };

        self.current_sort_col = Some(col_ix);
        self.current_sort_order = sort;

        // Update columns to reflect sort state
        for (i, col) in self.columns.iter_mut().enumerate() {
            if i == col_ix {
                *col = col.clone().sort(sort);
            } else {
                *col = col.clone().sort(ColumnSort::Default);
            }
        }

        // Get the current query and client from app state
        let app_state = self.app_state.clone();
        let async_cx = cx.to_async();

        cx.spawn(move |_table_state, _: &mut AsyncApp| async move {
            let mut cx = async_cx.clone();

            let _ = app_state.0.update(&mut cx, |state, cx| {
                // Get the base query
                if let Some(base_query) = &state.current_query {
                    // Generate ORDER BY clause
                    let order_direction = if ascending { "ASC" } else { "DESC" };
                    let safe_column = column_name.replace('"', "\"\"");

                    // Check if query already has ORDER BY and remove it
                    let base_query_clean =
                        if let Some(order_by_pos) = base_query.to_uppercase().rfind("ORDER BY") {
                            base_query[..order_by_pos].trim().to_string()
                        } else {
                            base_query.clone()
                        };

                    let sorted_query = format!(
                        "{} ORDER BY \"{}\" {}",
                        base_query_clean, safe_column, order_direction
                    );

                    // Store sort state
                    state.sort_column = Some(column_name.clone());
                    state.sort_ascending = ascending;

                    // Execute the sorted query
                    if let Some(client) = &state.active_connection {
                        let client = client.clone();
                        let app_state_inner = app_state.clone();
                        let async_cx_inner = cx.to_async();

                        cx.spawn(|_, _: &mut AsyncApp| async move {
                            let mut cx = async_cx_inner.clone();
                            let result = client.execute_query(&sorted_query).await;

                            let _ = app_state_inner.0.update(&mut cx, |state, cx| {
                                match result {
                                    Ok(res) => {
                                        state.query_results = Some(res);
                                        state.result_id += 1;
                                    }
                                    Err(e) => {
                                        state.error_message = Some(format!("Sort failed: {}", e));
                                    }
                                }
                                cx.notify();
                            });
                        })
                        .detach();
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}
