use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::api_types::ErrorResponse;
use crate::error::AppError;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let message = self.to_string();
        let status = match &self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::InvalidConfig(_) | AppError::ParseLine { .. } => StatusCode::BAD_REQUEST,
            AppError::CommandFailed { .. } => StatusCode::BAD_REQUEST,
            AppError::CommandTimedOut { .. } => StatusCode::GATEWAY_TIMEOUT,
            AppError::ConfigConflict => StatusCode::CONFLICT,
            AppError::ConfigApplyFailed { .. } => StatusCode::BAD_REQUEST,
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Auth(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
        };

        let body = Json(ErrorResponse { message });
        (status, body).into_response()
    }
}
