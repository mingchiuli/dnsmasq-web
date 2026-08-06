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
use leptos_axum::{LeptosRoutes, generate_route_list, render_route_with_context};
use tower_http::{
    limit::RequestBodyLimitLayer,
    trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use crate::app::{App, Shell, ShellProps};
use crate::server::auth::{RequestAuth, SESSION_COOKIE};
use crate::server::handlers;
use crate::server::state::AppState;
use crate::server_fns::{AuthStatus, Bootstrap, Login, Logout, SetLocale, SetupPassword};

const PUBLIC_SERVER_FN_PATHS: &[&str] = &[
    AuthStatus::PATH,
    Bootstrap::PATH,
    SetLocale::PATH,
    SetupPassword::PATH,
    Login::PATH,
    Logout::PATH,
];
const SERVER_FN_BODY_LIMIT: usize = 2 * 1024 * 1024;

pub fn router(state: AppState) -> Router {
    let leptos_options = state.inner.leptos_options.clone();
    let routes = generate_route_list(App);

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

    let route_handler = render_route_with_context::<AppState, _>(
        routes.clone(),
        {
            let state = state.clone();
            move || provide_context::<AppState>(state.clone())
        },
        move || {
            Shell(ShellProps {
                options: leptos_options.clone(),
            })
        },
    );

    Router::new()
        .leptos_routes_with_handler(routes, route_handler)
        .merge(
            server_fn_router
                .route_layer(middleware::from_fn(require_auth))
                .layer(RequestBodyLimitLayer::new(SERVER_FN_BODY_LIMIT)),
        )
        .fallback(handlers::site_assets)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            resolve_request_auth,
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(false))
                .on_request(DefaultOnRequest::new().level(Level::DEBUG))
                .on_response(DefaultOnResponse::new().level(Level::DEBUG))
                .on_failure(DefaultOnFailure::new().level(Level::WARN)),
        )
        .with_state(state)
}

async fn server_fn_handler(State(state): State<AppState>, req: Request) -> Response {
    leptos_axum::handle_server_fns_with_context(
        move || provide_context::<AppState>(state.clone()),
        req,
    )
    .await
    .into_response()
}

async fn require_auth(req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path();
    if PUBLIC_SERVER_FN_PATHS.contains(&path) {
        return next.run(req).await;
    }

    let authorized = req
        .extensions()
        .get::<RequestAuth>()
        .is_some_and(|auth| auth.authenticated);

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

async fn resolve_request_auth(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let token = request_cookie(&req, SESSION_COOKIE);
    let request_auth = crate::server::auth::resolve_request_auth(&state, token).await;
    req.extensions_mut().insert(request_auth);
    next.run(req).await
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
