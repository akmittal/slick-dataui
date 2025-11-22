/// UI components and layouts for the application.
/// This module is organized into submodules for maintainability:
/// - connection_modal: New connection dialog
/// - sidebar: Left sidebar with connections and tables
/// - editor: SQL query editor
/// - results: Query results display
/// - main_layout: Main UI layout orchestrator
pub mod connection_modal;
pub mod editor;
pub mod results;
pub mod sidebar;

use crate::state::GlobalAppState;
use crate::table_delegate::QueryResultsDelegate;
use gpui::prelude::*;
use gpui::*;
use gpui_component::input::InputState;
use gpui_component::table::TableState;

pub use connection_modal::ConnectionForm;

/// Main application layout component
pub struct MainLayout {
    pub state: GlobalAppState,
    pub form: ConnectionForm,
    pub query_input: Entity<InputState>,
    pub table_state: Option<Entity<TableState<QueryResultsDelegate>>>,
    pub last_result_id: usize,
}

impl MainLayout {
    pub fn new<C>(state: GlobalAppState, window: &mut Window, cx: &mut C) -> Self
    where
        C: AppContext,
        C::Result<Entity<InputState>>: Into<Entity<InputState>>,
    {
        let query_input = cx
            .new(|cx| {
                InputState::new(window, cx)
                    .multi_line()
                    .code_editor("sql")
                    .line_number(true)
                    .searchable(true)
                    .placeholder("-- Enter your SQL query here...")
            })
            .into();

        Self {
            state,
            form: ConnectionForm::new(window, cx),
            query_input,
            table_state: None,
            last_result_id: 0,
        }
    }
}

impl Render for MainLayout {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Create or recreate table state when results change
        let current_result_id = self.state.0.read(cx).result_id;
        if current_result_id != self.last_result_id {
            self.last_result_id = current_result_id;
            let app_state = self.state.0.read(cx);
            if let Some(results) = app_state.query_results.clone() {
                let sort_column = app_state.sort_column.clone();
                let sort_ascending = app_state.sort_ascending;

                let delegate = QueryResultsDelegate::new(
                    results,
                    self.state.clone(),
                    sort_column,
                    sort_ascending,
                );
                let state = cx.new(|cx| TableState::new(delegate, window, cx));
                self.table_state = Some(state);
            } else {
                self.table_state = None;
            }
        }

        div()
            .size_full()
            .flex()
            .flex_row()
            .bg(rgb(0x1e1e1e))
            .text_color(rgb(0xffffff))
            .child(sidebar::render_sidebar(self, cx))
            .child(editor::render_editor_section(self, cx))
            .children(if self.state.0.read(cx).is_connecting {
                Some(connection_modal::render_modal(self, cx))
            } else {
                None
            })
    }
}
