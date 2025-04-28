use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::error::AppError;

use askama_axum::IntoResponse;

pub mod post {
    use axum::extract::Path;
    use axum::response::Response;

    use crate::{
        app::AppState,
        models::{
            employment::{CreateEmployment, EmploymentState},
            job_position::JobPositionViewModel,
        },
        repositories::{
            employment::EmploymentRepository, event::EventRepository,
            job_position::JobPositionRepository, position_category::PositionCategoryRepository,
        },
        templates::JobStateTemplate,
        utils::date_utils::is_date_in_past,
    };

    use super::*;

    pub async fn job_state(
        Path(job_id): Path<i32>,
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Response, AppError> {
        let current_user_id = auth_session.clone().user.map(|user| user.id);
        let new_employment = app_state
            .employment_repository
            .create_employment(CreateEmployment {
                rating: 0,
                state: EmploymentState::Pending,
                user_id: current_user_id.unwrap_or(-1),
                position_id: job_id,
            })
            .await?;
        let job_position = app_state
            .job_position_repository
            .get_job_position_by_id(job_id)
            .await?;
        let job_category = app_state
            .position_category_repository
            .get_position_category_by_id(job_position.position_category_id)
            .await;
        let event = app_state
            .event_repository
            .get_event_by_id(job_position.event_id)
            .await?;

        let template = JobStateTemplate {
            session: auth_session,
            job: JobPositionViewModel {
                id: job_position.id,
                name: job_position.name,
                description: job_position.description,
                instructions_html: job_position.instructions_html,
                salary: job_position.salary,
                current_capacity: 0,
                max_capacity: 0,
                is_opened_for_registration: job_position.is_opened_for_registration,
                employment_state: Some(new_employment.state),
                position_category: match job_category {
                    Ok(category) => Some(category),
                    Err(_error) => None,
                },
            },
            is_in_past: is_date_in_past(event.date_start),
        };
        let html = template.render().unwrap();
        Ok(Html(html).into_response())
    }
}
