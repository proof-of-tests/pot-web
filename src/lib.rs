#![allow(non_snake_case)]
mod app;
#[cfg(feature = "ssr")]
mod handlers;
mod wasm;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    leptos::mount_to_body(app::App);
}

#[cfg(feature = "ssr")]
mod ssr_imports {
    use crate::app::App;
    use crate::handlers::{upload_proof_handler, upload_wasm_handler, validate_handler};
    use axum::{
        routing::{post, put},
        Extension, Router,
    };
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use std::sync::Arc;
    use worker::{event, Context, Env, HttpRequest, Result};

    fn router(env: Env) -> Router {
        let leptos_options = LeptosOptions::builder()
            .output_name("client")
            .site_pkg_dir("pkg")
            .build();
        let routes = generate_route_list(App);

        // build our application with a route
        let app: axum::Router<()> = Router::new()
            .leptos_routes(&leptos_options, routes, App)
            .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
            .route("/validate", post(validate_handler))
            .route("/upload_wasm", post(upload_wasm_handler))
            .route("/upload_proof", put(upload_proof_handler))
            .with_state(leptos_options)
            .layer(Extension(Arc::new(env)));
        app
    }

    #[event(start)]
    fn register() {
        // Nothing for now
    }

    #[event(fetch)]
    async fn fetch(req: HttpRequest, env: Env, _ctx: Context) -> Result<axum::http::Response<axum::body::Body>> {
        _ = console_log::init_with_level(log::Level::Debug);
        use tower_service::Service;

        console_error_panic_hook::set_once();

        Ok(router(env).call(req).await?)
    }
}
