use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::{
    app::AppState, handlers::app::auth::AuthSession, models::user::UserRole,
    utils::response_utils::generate_unauthorized_response,
};

pub async fn check_organizer(
    auth_session: AuthSession,
    State(_app_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let current_user = auth_session
        .clone()
        .user
        .expect("User should be logged in.");

    match current_user.role {
        UserRole::Admin | UserRole::Organizer => {
            let response = next.run(request).await;
            Ok(response)
        }
        _ => Ok(generate_unauthorized_response()),
    }
}

pub async fn check_admin(
    auth_session: AuthSession,
    State(_app_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let current_user = auth_session
        .clone()
        .user
        .expect("User should be logged in.");

    match current_user.role {
        UserRole::Admin => {
            let response = next.run(request).await;
            Ok(response)
        }
        _ => Ok(generate_unauthorized_response()),
    }
}
