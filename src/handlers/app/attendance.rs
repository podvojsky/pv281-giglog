use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::error::AppError;

pub mod get {

    use crate::{
        app::AppState,
        error::ApiError,
        repositories::event::EventRepository,
        templates::{ActiveRoute, AttendanceTemplate},
    };

    use super::*;

    pub async fn attendance(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let current_user = match auth_session.clone().user {
            Some(user) => user,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };
        let events = app_state
            .event_repository
            .list_events_worked_by_user(current_user.id)
            .await?;

        let template = AttendanceTemplate {
            session: auth_session,
            active_route: Some(ActiveRoute::Attendance),
            events,
        };
        let html = template.render().unwrap();
        Ok(Html(html))
    }
}
