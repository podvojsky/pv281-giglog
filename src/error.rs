use askama_axum::{IntoResponse, Response};
use axum::http::StatusCode;
use serde::Serialize;
use thiserror::Error;

pub struct AppError(pub anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("repository error: {0}")]
    GenericError(String),
    #[error("connection pool error: {0}")]
    ConnectionError(String),
    #[error("no record found")]
    NotFound,
}

/// User-facing error type
#[derive(Error, Debug, Serialize)]
pub enum ApiError {
    #[error("internal server error")]
    InternalServerError,
    #[error("not found")]
    NotFound,
    #[allow(dead_code)]
    #[error("bad request")]
    BadRequest,
}

impl From<RepositoryError> for ApiError {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::ConnectionError(_) | RepositoryError::GenericError(_) => {
                Self::InternalServerError
            }
            RepositoryError::NotFound => Self::NotFound,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = match self {
            ApiError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::BadRequest => StatusCode::BAD_REQUEST,
        };

        let error_body = serde_json::json!({ "error": self.to_string() });
        (status_code, axum::Json(error_body)).into_response()
    }
}

impl From<sqlx::Error> for RepositoryError {
    fn from(err: sqlx::Error) -> Self {
        RepositoryError::ConnectionError(err.to_string())
    }
}

impl From<anyhow::Error> for RepositoryError {
    fn from(err: anyhow::Error) -> Self {
        RepositoryError::GenericError(err.to_string())
    }
}
