use gpui::prelude::*;
use gpui::*;
use gpui::AsyncApp;
use crate::state::GlobalAppState;

use gpui_component::input::{Input, InputState};
use gpui_component::button::{Button, ButtonVariants};
use crate::state::{DatabaseType, ConnectionConfig};
use crate::db::{DatabaseClient, SqliteClient, PostgresClient};
use std::sync::Arc;

pub struct ConnectionForm {
    pub name_input: Entity<InputState>,
    pub conn_string_input: Entity<InputState>,
    pub db_type: DatabaseType,
}

impl ConnectionForm {
    fn new<C>(window: &mut Window, cx: &mut C) -> Self 
    where 
        C: AppContext,
        C::Result<Entity<InputState>>: Into<Entity<InputState>>
    {
        Self {
            name_input: cx.new(|cx| InputState::new(window, cx)).into(),
            conn_string_input: cx.new(|cx| InputState::new(window, cx)).into(),
            db_type: DatabaseType::Sqlite,
        }
    }
}

pub struct MainLayout {
    state: GlobalAppState,
    form: ConnectionForm,
    query_input: Entity<InputState>,
}

impl MainLayout {
    pub fn new<C>(state: GlobalAppState, window: &mut Window, cx: &mut C) -> Self 
    where 
        C: AppContext,
        C::Result<Entity<InputState>>: Into<Entity<InputState>>
    {
        Self { 
            state,
            form: ConnectionForm::new(window, cx),
            query_input: cx.new(|cx| InputState::new(window, cx)).into(),
        }
    }

    fn render_modal(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .size_full()
            .bg(black().opacity(0.8))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w_96()
                    .bg(rgb(0x252526))
                    .border_1()
                    .border_color(rgb(0x454545))
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(div().text_xl().child("New Connection"))
                    .child(
                        Input::new(&self.form.name_input)
                    )
                    .child(
                        Input::new(&self.form.conn_string_input)
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .justify_end()
                            .child(
                                Button::new("cancel")
                                    .label("Cancel")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.state.0.update(cx, |state, cx| {
                                            state.toggle_connecting(cx);
                                        });
                                    }))
                            )
                            .child(
                                Button::new("save")
                                    .primary()
                                    .label("Save")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        let name = this.form.name_input.read(cx).value().to_string();
                                        let conn_string = this.form.conn_string_input.read(cx).value().to_string();
                                        
                                        let config = ConnectionConfig {
                                            name,
                                            db_type: this.form.db_type.clone(),
                                            connection_string: conn_string,
                                        };
                                        this.state.0.update(cx, |state, cx| {
                                            state.connections.push(config);
                                            state.toggle_connecting(cx);
                                        });
                                    }))
                            )
                    )
            )
    }
}


impl Render for MainLayout {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_row()
            .bg(rgb(0x1e1e1e))
            .text_color(rgb(0xffffff))
            .child(
                div()
                    .w_64()
                    .h_full()
                    .border_r_1()
                    .border_color(rgb(0x333333))
                    .child(
                        div()
                            .p_4()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .child("Connections")
                                    .child(
                                        Button::new("add_conn")
                                            .label("+")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.state.0.update(cx, |state, cx| {
                                                    state.toggle_connecting(cx);
                                                });
                                            }))
                                    )
                            )
                            .children(
                                self.state.0.read(cx).connections.iter().enumerate().map(|(i, conn)| {
                                    div()
                                        .id(i)
                                        .child(conn.name.clone())
                                        .cursor_pointer()
                                        .on_click(cx.listener({
                                            let conn = conn.clone();
                                            move |this, _, _, cx| {
                                                let app_state = this.state.0.clone();
                                                let conn = conn.clone();
                                                
                                                let async_cx = cx.to_async();
                                                cx.spawn(|_, _: &mut AsyncApp| async move {
                                                    let mut cx = async_cx.clone();
                                                    // Update state to connecting
                                                    let _ = app_state.update(&mut cx, |state, cx| {
                                                        state.is_connecting = true;
                                                        state.error_message = None;
                                                        cx.notify();
                                                    });

                                                    let client_result = match conn.db_type {
                                                        DatabaseType::Sqlite => {
                                                            SqliteClient::new(&conn.connection_string).await
                                                                .map(|c| Arc::new(c) as Arc<dyn DatabaseClient>)
                                                        },
                                                        DatabaseType::Postgres => {
                                                            PostgresClient::new(&conn.connection_string).await
                                                                .map(|c| Arc::new(c) as Arc<dyn DatabaseClient>)
                                                        }
                                                    };

                                                    match client_result {
                                                        Ok(client) => {
                                                            let tables_result = client.get_tables().await;
                                                            let _ = app_state.update(&mut cx, |state, cx| {
                                                                state.is_connecting = false;
                                                                match tables_result {
                                                                    Ok(tables) => {
                                                                        state.active_connection = Some(client);
                                                                        state.active_connection_name = Some(conn.name);
                                                                        state.tables = tables;
                                                                    },
                                                                    Err(e) => {
                                                                        state.error_message = Some(format!("Failed to fetch tables: {}", e));
                                                                    }
                                                                }
                                                                cx.notify();
                                                            });
                                                        },
                                                        Err(e) => {
                                                            let _ = app_state.update(&mut cx, |state, cx| {
                                                                state.is_connecting = false;
                                                                state.error_message = Some(format!("Failed to connect: {}", e));
                                                                cx.notify();
                                                            });
                                                        }
                                                    }
                                                }).detach();
                                            }
                                        }))
                                })
                            )
                            .child(
                                div()
                                    .mt_4()
                                    .child("Tables")
                                    .children(
                                        self.state.0.read(cx).tables.iter().map(|table| {
                                            div().child(table.name.clone()).ml_2()
                                        })
                                    )
                            )
                    )
            )
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .h_1_2()
                            .border_b_1()
                            .border_color(rgb(0x333333))
                            .p_4()
                            .child("Query Editor")
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(
                                        Input::new(&self.query_input)
                                    )
                                    .child(
                                        Button::new("run_query")
                                            .primary()
                                            .label("Run")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                let app_state = this.state.0.clone();
                                                // Get query from input
                                                let query = this.query_input.read(cx).value().to_string();
                                                
                                                // Read active connection on main thread
                                                let client_opt = this.state.0.read(cx).active_connection.clone();

                                                let async_cx = cx.to_async();
                                                cx.spawn(|_, _: &mut AsyncApp| async move {
                                                    let mut cx = async_cx.clone();
                                                    
                                                    if let Some(client) = client_opt {
                                                        let result = client.execute_query(&query).await;
                                                        let _ = app_state.update(&mut cx, |state, cx| {
                                                            match result {
                                                                Ok(res) => state.query_results = Some(res),
                                                                Err(e) => state.error_message = Some(format!("Query failed: {}", e)),
                                                            }
                                                            cx.notify();
                                                        });
                                                    }
                                                }).detach();
                                            }))
                                    )
                            )
                    )
                    .child(
                        div()
                            .flex_1()
                            .p_4()
                            .child("Results")
                            .child(
                                if let Some(results) = &self.state.0.read(cx).query_results {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(
                                            div()
                                                .flex()
                                                .children(
                                                    results.columns.iter().map(|col| {
                                                        div().w_32().border_1().child(col.clone())
                                                    })
                                                )
                                        )
                                        .children(
                                            results.rows.iter().map(|row| {
                                                div()
                                                    .flex()
                                                    .children(
                                                        row.iter().map(|cell| {
                                                            div().w_32().border_1().child(cell.clone())
                                                        })
                                                    )
                                            })
                                        )
                                } else {
                                    div().child("No results")
                                }
                            )
                    )
            )
            .children(
                if self.state.0.read(cx).is_connecting {
                    Some(self.render_modal(cx))
                } else {
                    None
                }
            )
    }
}
