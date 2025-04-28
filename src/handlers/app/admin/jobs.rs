use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::error::AppError;

pub mod get {
    use crate::app::AppState;
    use crate::models::{employment, job_position};
    use crate::models::employment::EmploymentState;
    use crate::repositories::employment::EmploymentRepository;
    use crate::repositories::event::EventRepository;
    use crate::repositories::job_position::JobPositionRepository;
    use crate::repositories::position_category::PositionCategoryRepository;
    use crate::templates::ManageJobsTemplate;
    use crate::view_models::jobs::ManageJobPositionsViewModel;
    use super::*;

    pub async fn jobs(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let mut jobs: Vec<ManageJobPositionsViewModel> = Vec::new();

        let all_jobs = app_state
            .job_position_repository
            .list_job_positions(job_position::SelectManyFilter {
                event_id: None,
                position_category_id: None,
                salary: None,
                currency: None,
                capacity: None,
                is_opened_for_registration: None,
            })
            .await?;
        for job in all_jobs {
            let employments = app_state
                .employment_repository
                .list_employment(employment::SelectManyFilter {
                    position_id: Some(job.id),
                    user_id: None,
                    state: None,
                    rating: None,
                })
                .await?;
            let current_capacity = employments
                .iter()
                .filter(|e| {
                    e.state == EmploymentState::Accepted || e.state == EmploymentState::Done
                })
                .count();
            let category = app_state
                .position_category_repository
                .get_position_category_by_id(job.position_category_id)
                .await?;
            let event = app_state
                .event_repository
                .get_event_by_id(job.event_id)
                .await?;
            jobs.push(ManageJobPositionsViewModel {
                id: job.id,
                name: job.name,
                salary: job.salary,
                current_capacity,
                max_capacity: job.capacity,
                is_opened_for_registration: job.is_opened_for_registration,
                currency: job.currency,
                event: event.clone(),
                category,
            });
        }

        let template = ManageJobsTemplate {
            session: auth_session,
            active_route: Some(crate::templates::ActiveRoute::Manage),
            jobs,
        };
        let html = template.render().unwrap();
        Ok(Html(html))
    }
}
