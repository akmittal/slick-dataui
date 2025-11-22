mod db;
mod error;
mod persistence;
mod state;
mod table_delegate;
mod ui;

use gpui::{AppContext, Application, WindowOptions};
use gpui_component::Root;
use state::GlobalAppState;
use ui::MainLayout;

fn main() {
    let _guard = sentry::init((
        option_env!("SENTRY_DSN"),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    let app = Application::new().with_assets(gpui_component_assets::Assets);

    app.run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let app_state_entity = cx.new(|_| state::AppState::new());
                let app_state = GlobalAppState::new(app_state_entity.clone());
                let view = cx.new(|cx| MainLayout::new(app_state, window, cx));
                // This first level on the window, should be a Root.
                cx.new(|cx| Root::new(view, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
