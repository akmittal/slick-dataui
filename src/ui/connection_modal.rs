/// Connection modal component for creating new database connections.
use gpui::prelude::*;
use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::radio::{RadioGroup, Radio};
use crate::state::{DatabaseType, ConnectionConfig};

pub struct ConnectionForm {
    pub name_input: Entity<InputState>,
    pub conn_string_input: Entity<InputState>,
    pub db_type: DatabaseType,
    pub selected_path: Option<String>,
}

impl ConnectionForm {
    pub fn new<C>(window: &mut Window, cx: &mut C) -> Self
    where
        C: AppContext,
        C::Result<Entity<InputState>>: Into<Entity<InputState>>,
    {
        Self {
            name_input: cx
                .new(|cx| InputState::new(window, cx).placeholder("Connection name"))
                .into(),
            conn_string_input: cx
                .new(|cx| {
                    InputState::new(window, cx).placeholder("Connection string or file")
                })
                .into(),
            db_type: DatabaseType::Sqlite,
            selected_path: None,
        }
    }
}

pub fn render_modal(layout: &mut super::MainLayout, cx: &mut Context<super::MainLayout>) -> impl IntoElement {
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
                .child(render_connection_name_field(layout, cx))
                .child(render_database_type_selector(layout, cx))
                .child(render_connection_string_field(layout, cx))
                .child(render_modal_actions(layout, cx))
        )
}

fn render_connection_name_field(
    layout: &mut super::MainLayout,
    _cx: &mut Context<super::MainLayout>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_sm().child("Connection Name"))
        .child(Input::new(&layout.form.name_input).appearance(true))
}

fn render_database_type_selector(
    layout: &mut super::MainLayout,
    cx: &mut Context<super::MainLayout>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_sm().child("Database Type"))
        .child(
            RadioGroup::horizontal("db_type")
                .selected_index(match layout.form.db_type {
                    DatabaseType::Sqlite => Some(0),
                    DatabaseType::Postgres => Some(1),
                })
                .children(vec![
                    Radio::new(0).label("SQLite"),
                    Radio::new(1).label("PostgreSQL"),
                ])
                .on_click(cx.listener(|this, index, _, cx| {
                    let idx = *index;
                    this.form.db_type = if idx == 0 {
                        DatabaseType::Sqlite
                    } else {
                        DatabaseType::Postgres
                    };
                    cx.notify();
                })),
        )
}

fn render_connection_string_field(
    layout: &mut super::MainLayout,
    cx: &mut Context<super::MainLayout>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_sm().child("Connection String"))
        .child(
            div()
                .flex()
                .gap_2()
                .child(
                    div()
                        .flex_1()
                        .child(Input::new(&layout.form.conn_string_input)),
                )
                .when(layout.form.db_type == DatabaseType::Sqlite, |el| {
                    el.child(
                        Button::new("browse_file")
                            .label("Browse...")
                            .on_click(cx.listener(|_this, _, _, cx| {
                                let async_cx = cx.to_async();
                                cx.spawn(|this: WeakEntity<super::MainLayout>, _: &mut AsyncApp| {
                                    async move {
                                        let mut cx = async_cx.clone();
                                        let file = rfd::AsyncFileDialog::new()
                                            .add_filter("SQLite Database", &["db", "sqlite", "sqlite3"])
                                            .pick_file()
                                            .await;

                                        if let Some(file) = file {
                                            let path_str = format!("sqlite://{}", file.path().display());
                                            let _ = this.update(&mut cx, |this, cx| {
                                                this.form.selected_path = Some(path_str);
                                                cx.notify();
                                            });
                                        }
                                    }
                                })
                                .detach();
                            })),
                    )
                })
        )
        .when_some(layout.form.selected_path.as_ref(), |el, path| {
            el.child(
                div()
                    .text_xs()
                    .text_color(rgb(0x888888))
                    .child(format!("Selected: {}", path)),
            )
        })
}

fn render_modal_actions(
    _layout: &mut super::MainLayout,
    cx: &mut Context<super::MainLayout>,
) -> impl IntoElement {
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
                })),
        )
        .child(
            Button::new("save")
                .primary()
                .label("Save")
                .on_click(cx.listener(|this, _, _, cx| {
                    let name = this.form.name_input.read(cx).value().to_string();
                    let conn_string = if let Some(ref path) = this.form.selected_path {
                        path.clone()
                    } else {
                        this.form.conn_string_input.read(cx).value().to_string()
                    };

                    let config = ConnectionConfig {
                        name,
                        db_type: this.form.db_type.clone(),
                        connection_string: conn_string,
                    };
                    this.state.0.update(cx, |state, cx| {
                        state.add_connection(config);
                        state.toggle_connecting(cx);
                    });
                    this.form.selected_path = None;
                })),
        )
}
