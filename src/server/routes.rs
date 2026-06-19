use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Method, Request};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post, put};
use leptos::context::provide_context;
use tower_http::trace::TraceLayer;

use crate::server::handlers;
use crate::server::state::AppState;

pub fn router(state: AppState) -> Router {
    let mut router = Router::new();
    for (path, method) in leptos::server_fn::axum::server_fn_paths() {
        router = match method {
            Method::GET => router.route(path, get(server_fn_handler)),
            Method::POST => router.route(path, post(server_fn_handler)),
            Method::PUT => router.route(path, put(server_fn_handler)),
            Method::DELETE => router.route(path, delete(server_fn_handler)),
            Method::PATCH => router.route(path, patch(server_fn_handler)),
            _ => router,
        };
    }

    router
        .fallback(handlers::static_assets)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn server_fn_handler(State(state): State<AppState>, req: Request<Body>) -> Response {
    leptos_axum::handle_server_fns_with_context(
        move || provide_context::<AppState>(state.clone()),
        req,
    )
    .await
    .into_response()
}
