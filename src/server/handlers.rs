use std::path::Path;

use axum::body::Body;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "dist/"]
struct Assets;

pub async fn static_assets(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    serve_asset(path).unwrap_or_else(|| serve_asset("index.html").unwrap_or_else(not_found))
}

fn serve_asset(path: &str) -> Option<Response> {
    let asset = Assets::get(path)?;
    let mime = mime_guess::from_path(Path::new(path)).first_or_octet_stream();
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.as_ref())
        .body(Body::from(asset.data.into_owned()))
        .unwrap_or_else(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to build asset response",
            )
                .into_response()
        });
    Some(response)
}

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "not found").into_response()
}
