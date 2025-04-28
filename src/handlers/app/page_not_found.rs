use crate::{app::AppState, handlers::app::auth::AuthSession, templates::PageNotFoundTemplate};
use askama::Template;
use axum::{extract::State, response::Html};

use askama_axum::IntoResponse;
use axum::http::StatusCode;

pub async fn page_not_found(
    _auth_session: AuthSession,
    State(_app_state): State<AppState>,
) -> impl IntoResponse {
    let template = PageNotFoundTemplate {};
    let html = template.render().unwrap();
    (StatusCode::NOT_FOUND, Html(html))
}
