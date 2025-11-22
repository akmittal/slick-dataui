/// SQL query editor component.
use gpui::prelude::*;
use gpui::*;
use gpui_component::input::Input;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::table::Table;

pub fn render_editor_section(
    layout: &mut super::MainLayout,
    cx: &mut Context<super::MainLayout>,
) -> impl IntoElement {
    div()
        .flex_1()
        .h_full()
        .flex()
        .flex_col()
        .child(render_editor_header(layout, cx))
        .child(render_query_results(layout, cx))
}

fn render_editor_header(
    layout: &mut super::MainLayout,
    cx: &mut Context<super::MainLayout>,
) -> impl IntoElement {
    div()
        .h_1_2()
        .border_b_1()
        .border_color(rgb(0x333333))
        .p_4()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child("SQL Query Editor"),
        )
        .child(render_query_input(layout, cx))
}

fn render_query_input(
    layout: &mut super::MainLayout,
    cx: &mut Context<super::MainLayout>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .flex_1()
        .child(
            Input::new(&layout.query_input)
                .h(px(150.))
                .appearance(true),
        )
        .child(render_run_button(layout, cx))
}

fn render_run_button(
    _layout: &mut super::MainLayout,
    cx: &mut Context<super::MainLayout>,
) -> impl IntoElement {
    Button::new("run_query")
        .label("Run Query")
        .primary()
        .on_click(cx.listener(|this, _, _, cx| {
            let app_state = this.state.0.clone();
            let query = this.query_input.read(cx).value().to_string();
            let client_opt = this.state.0.read(cx).active_connection.clone();

            let async_cx = cx.to_async();
            cx.spawn(|_this_weak: WeakEntity<super::MainLayout>, _: &mut AsyncApp| async move {
                let mut cx = async_cx.clone();

                if let Some(client) = client_opt {
                    let result = client.execute_query(&query).await;
                    let _ = app_state.update(&mut cx, |state, cx| {
                        match result {
                            Ok(res) => {
                                state.query_results = Some(res);
                            }
                            Err(e) => state.error_message = Some(format!("Query failed: {}", e)),
                        }
                        cx.notify();
                    });
                }
            })
            .detach();
        }))
}

pub fn render_query_results(
    layout: &mut super::MainLayout,
    cx: &mut Context<super::MainLayout>,
) -> impl IntoElement {
    div()
        .flex_1()
        .p_4()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child("Query Results"),
        )
        .child(
            if let Some(table_state) = &layout.table_state {
                // Use the table state that was created in MainLayout::render
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        Table::new(table_state)
                            .stripe(true)
                            .bordered(true)
                            .scrollbar_visible(true, true)
                    )
                    .child(
                        if let Some(results) = &layout.state.0.read(cx).query_results {
                            div()
                                .flex()
                                .gap_2()
                                .justify_between()
                                .px_2()
                                .py_2()
                                .text_xs()
                                .text_color(rgb(0x888888))
                                .child(
                                    format!(
                                        "Total: {} rows | Click column headers to sort",
                                        results.rows.len()
                                    )
                                )
                                .into_element()
                        } else {
                            div().into_element()
                        }
                    )
                    .into_element()
            } else if layout.state.0.read(cx).query_results.is_some() {
                // Results exist but table not ready yet
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(0x888888))
                    .child("Loading table...")
                    .into_element()
            } else {
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(0x888888))
                    .child("No results yet. Run a query to see results here.")
                    .into_element()
            },
        )
}
