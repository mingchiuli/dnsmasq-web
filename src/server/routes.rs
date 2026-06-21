use axum::Router;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::header::{CONTENT_TYPE, COOKIE};
use axum::http::{Method, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post, put};
use leptos::context::provide_context;
use leptos::server_fn::ServerFn;
use tower_http::trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::server::auth::SESSION_COOKIE;
use crate::server::handlers;
use crate::server::state::AppState;
use crate::server_fns::{AuthStatus, Login, Logout, SetLocale, SetupPassword};

const PUBLIC_SERVER_FN_PATHS: &[&str] = &[
    AuthStatus::PATH,
    SetLocale::PATH,
    SetupPassword::PATH,
    Login::PATH,
    Logout::PATH,
];

pub fn router(state: AppState) -> Router {
    let mut server_fn_router = Router::new();
    for (path, method) in leptos::server_fn::axum::server_fn_paths() {
        server_fn_router = match method {
            Method::GET => server_fn_router.route(path, get(server_fn_handler)),
            Method::POST => server_fn_router.route(path, post(server_fn_handler)),
            Method::PUT => server_fn_router.route(path, put(server_fn_handler)),
            Method::DELETE => server_fn_router.route(path, delete(server_fn_handler)),
            Method::PATCH => server_fn_router.route(path, patch(server_fn_handler)),
            _ => server_fn_router,
        };
    }

    Router::new()
        .merge(
            server_fn_router
                .route_layer(middleware::from_fn_with_state(state.clone(), require_auth)),
        )
        .fallback(handlers::static_assets)
        .layer(
            TraceLayer::new_for_http()
                .on_request(DefaultOnRequest::new().level(Level::DEBUG))
                .on_response(DefaultOnResponse::new().level(Level::DEBUG))
                .on_failure(DefaultOnFailure::new().level(Level::WARN)),
        )
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

async fn require_auth(State(state): State<AppState>, req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path();
    if PUBLIC_SERVER_FN_PATHS.contains(&path) {
        return next.run(req).await;
    }

    let authorized = if let Some(token) = request_cookie(&req, SESSION_COOKIE) {
        state.verify_session(&token).await
    } else {
        false
    };

    if authorized {
        next.run(req).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            [(CONTENT_TYPE, "text/plain")],
            "ServerError|unauthorized",
        )
            .into_response()
    }
}

fn request_cookie(req: &Request<Body>, cookie_name: &str) -> Option<String> {
    let cookie_header = req.headers().get(COOKIE)?.to_str().ok()?;
    cookie_header
        .split(';')
        .filter_map(|cookie| cookie.trim().split_once('='))
        .find_map(|(name, value)| {
            if name == cookie_name {
                Some(value.to_string())
            } else {
                None
            }
        })
}
