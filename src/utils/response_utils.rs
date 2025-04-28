use crate::templates::{FormErrorsTemplate, ToastTemplate, ToastType, UnauthorizedTemplate};
use askama::Template;
use askama_axum::IntoResponse;
use axum::response::Response;
use serde::{Deserialize, Serialize};
use validator::ValidationErrors;

pub fn generate_toast_response(
    toast_type: ToastType,
    message: String,
) -> axum::http::Response<axum::body::Body> {
    let status = match toast_type {
        ToastType::Success => 200,
        ToastType::Error => 400,
    };
    let template = ToastTemplate {
        toast_type,
        message,
    };
    let html = template.render().unwrap();
    Response::builder()
        .status(status)
        .body(html)
        .unwrap()
        .into_response()
}

pub fn generate_form_errors_response(
    errors: ValidationErrors,
) -> axum::http::Response<axum::body::Body> {
    let template = FormErrorsTemplate {
        validation_errors: errors,
    };
    let html = template.render().unwrap();
    Response::builder()
        .status(400)
        .body(html)
        .unwrap()
        .into_response()
}

pub fn generate_htmx_redirect(to: &str) -> axum::http::Response<axum::body::Body> {
    Response::builder()
        .header("HX-Redirect", to)
        .body(String::new())
        .unwrap()
        .into_response()
}

pub fn generate_unauthorized_response() -> axum::http::Response<axum::body::Body> {
    let template = UnauthorizedTemplate {};
    let html = template.render().unwrap();
    Response::builder()
        .status(401)
        .body(html)
        .unwrap()
        .into_response()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckboxState {
    On,
    Off,
}
