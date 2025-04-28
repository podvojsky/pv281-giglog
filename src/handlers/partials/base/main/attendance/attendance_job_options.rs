use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::error::AppError;

pub mod get {
    use axum::extract::Query;

    use serde::Deserialize;

    use crate::{
        app::AppState, error::ApiError, repositories::job_position::JobPositionRepository,
        templates::AttendanceJobOptionsTemplate,
    };

    use super::*;

    #[derive(Deserialize)]
    pub struct Params {
        event_id: i32,
    }

    pub async fn attendance_job_options(
        params: Query<Params>,
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let current_user = match auth_session.clone().user {
            Some(user) => user,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };

        let jobs = app_state
            .job_position_repository
            .list_job_positions_worked_by_user_on_event(current_user.id, params.event_id)
            .await?;

        let template = AttendanceJobOptionsTemplate { jobs };
        let html = template.render().unwrap();
        Ok(Html(html))
    }
}
