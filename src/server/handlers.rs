use std::path::{Component, Path, PathBuf};

use axum::body::Body;
use axum::extract::State;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
#[cfg(feature = "embedded-assets")]
use rust_embed::RustEmbed;

use crate::server::state::AppState;

#[cfg(feature = "embedded-assets")]
#[derive(RustEmbed)]
#[folder = "target/site/"]
struct EmbeddedAssets;

pub async fn site_assets(State(state): State<AppState>, uri: Uri) -> Response {
    let Some(relative_path) = resolve_site_asset_path(uri.path()) else {
        return not_found();
    };
    let path = Path::new(state.inner.leptos_options.site_root.as_ref()).join(&relative_path);

    serve_asset(path, &relative_path).await
}

async fn serve_asset(path: PathBuf, relative_path: &Path) -> Response {
    let bytes = match tokio::fs::read(&path).await {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return embedded_asset(relative_path).unwrap_or_else(not_found);
        }
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to read asset: {error}"),
            )
                .into_response();
        }
    };

    asset_response(bytes, &path)
}

fn asset_response(bytes: Vec<u8>, path: &Path) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.as_ref())
        .body(Body::from(bytes))
        .unwrap_or_else(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to build asset response",
            )
                .into_response()
        })
}

fn resolve_site_asset_path(uri_path: &str) -> Option<PathBuf> {
    let path = uri_path.trim_start_matches('/');
    if path.is_empty() {
        return None;
    }

    let mut relative = PathBuf::new();
    for component in Path::new(path).components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }

    Some(relative)
}

#[cfg(feature = "embedded-assets")]
fn embedded_asset(relative_path: &Path) -> Option<Response> {
    let asset_path = relative_path.to_str()?;
    let asset = EmbeddedAssets::get(asset_path)?;
    Some(asset_response(asset.data.into_owned(), relative_path))
}

#[cfg(not(feature = "embedded-assets"))]
fn embedded_asset(_relative_path: &Path) -> Option<Response> {
    None
}

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "not found").into_response()
}
