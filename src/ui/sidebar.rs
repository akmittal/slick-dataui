use crate::db::{DatabaseClient, PostgresClient, SqliteClient};
use crate::state::DatabaseType;
/// Sidebar component: connections list and tables list.
use gpui::prelude::*;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::collapsible::Collapsible;
use std::sync::Arc;

pub fn render_sidebar(
    layout: &mut super::MainLayout,
    cx: &mut Context<super::MainLayout>,
) -> impl IntoElement {
    div()
        .w_64()
        .h_full()
        .border_r_1()
        .border_color(rgb(0x333333))
        .child(render_sidebar_content(layout, cx))
}

fn render_sidebar_content(
    layout: &mut super::MainLayout,
    cx: &mut Context<super::MainLayout>,
) -> impl IntoElement {
    div()
        .p_4()
        .flex()
        .flex_col()
        .gap_2()
        .child(render_connections_header(layout, cx))
        .children(render_connections_list(layout, cx))
        .child(
            div().when_some(layout.state.0.read(cx).error_message.clone(), |el, msg| {
                el.my_2().text_sm().text_color(rgb(0xff5555)).child(msg)
            }),
        )
        .child(render_tables_section(layout, cx))
}

fn render_connections_header(
    _layout: &mut super::MainLayout,
    cx: &mut Context<super::MainLayout>,
) -> impl IntoElement {
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
                })),
        )
}

fn render_connections_list(
    layout: &super::MainLayout,
    cx: &mut Context<super::MainLayout>,
) -> Vec<impl IntoElement> {
    layout
        .state
        .0
        .read(cx)
        .connections
        .iter()
        .enumerate()
        .map(|(i, conn)| {
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
                            let _ = app_state.update(&mut cx, |_state, cx| {
                                cx.notify();
                            });

                            if conn.connection_string.is_empty() {
                                let _ = app_state.update(&mut cx, |state, cx| {
                                    state.is_connecting = false;
                                    state.error_message = Some("Connection string/password missing. Please delete and re-create the connection.".to_string());
                                    cx.notify();
                                });
                                return;
                            }

                            println!("Connecting to '{}' with string: '{}'", conn.name, conn.connection_string);
                            let client_result = match conn.db_type {
                                DatabaseType::Sqlite => {
                                    SqliteClient::new(&conn.connection_string)
                                        .await
                                        .map(|c| Arc::new(c) as Arc<dyn DatabaseClient>)
                                }
                                DatabaseType::Postgres => {
                                    PostgresClient::new(&conn.connection_string)
                                        .await
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
                                            }
                                            Err(e) => {
                                                state.error_message =
                                                    Some(format!("Failed to fetch tables: {}", e));
                                            }
                                        }
                                        cx.notify();
                                    });
                                }
                                Err(e) => {
                                    let _ = app_state.update(&mut cx, |state, cx| {
                                        state.is_connecting = false;
                                        println!("Connection failed: {}", e);
                                        state.error_message = Some(format!("Failed to connect: {}", e));
                                        cx.notify();
                                    });
                                }
                            }
                        })
                        .detach();
                    }
                }))
        })
        .collect()
}

fn render_tables_section(
    layout: &mut super::MainLayout,
    cx: &mut Context<super::MainLayout>,
) -> impl IntoElement {
    div().mt_4().child("Tables").child(
        Collapsible::new()
            .open(true)
            .content(render_tables_list(layout, cx)),
    )
}

fn render_tables_list(
    layout: &mut super::MainLayout,
    cx: &mut Context<super::MainLayout>,
) -> impl IntoElement {
    div().children(
        layout
            .state
            .0
            .read(cx)
            .tables
            .iter()
            .enumerate()
            .map(|(i, table)| {
                let table = table.clone();
                Button::new(("table", i))
                    .label(table.name.clone())
                    .ml_2()
                    .cursor_pointer()
                    .on_click(cx.listener({
                        let table = table.clone();
                        move |this, event: &gpui::ClickEvent, _, cx| {
                            if event.click_count() != 2 {
                                return;
                            }

                            let app_state = this.state.0.clone();
                            let client_opt = this.state.0.read(cx).active_connection.clone();
                            let table_name = table.name.clone();

                            let async_cx = cx.to_async();
                            cx.spawn(|_, _: &mut AsyncApp| async move {
                                let mut cx = async_cx.clone();
                                if let Some(client) = client_opt {
                                    let safe_name = table_name.replace('"', "\"\"");
                                    let query = format!("SELECT * FROM \"{}\"", safe_name);
                                    let result = client.execute_query(&query).await;
                                    let _ = app_state.update(&mut cx, |state, cx| {
                                        match result {
                                            Ok(res) => state.query_results = Some(res),
                                            Err(e) => {
                                                state.error_message = Some(format!(
                                                    "Failed to fetch table data: {}",
                                                    e
                                                ))
                                            }
                                        }
                                        cx.notify();
                                    });
                                }
                            })
                            .detach();
                        }
                    }))
            }),
    )
}
