use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::error::AppError;

pub mod get {
    use super::*;
    use crate::{
        app::AppState,
        error::ApiError,
        models::{
            employment::{self, EmploymentState},
            event::{Event, SelectManyFilter},
            job_position::{self},
        },
        repositories::{
            employment::EmploymentRepository, event::EventRepository,
            event_manager_relation::EventManagerRelationRepository,
            job_position::JobPositionRepository, position_category::PositionCategoryRepository,
        },
        templates::ManageJobsTemplate,
        view_models::jobs::ManageJobPositionsViewModel,
    };

    pub async fn manage(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let current_user = match auth_session.clone().user {
            Some(user) => user,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };
        let mut jobs: Vec<ManageJobPositionsViewModel> = Vec::new();
        let mut events = app_state
            .event_repository
            .list_events(SelectManyFilter {
                date_from: None,
                date_to: None,
                is_draft: None,
                venue_id: None,
                owner_id: Some(current_user.id),
                city: None,
                state: None,
                name: None,
            })
            .await?;
        let mut managed_events: Vec<Event> = Vec::new();
        let event_manager_relations = app_state
            .event_manager_relation_repository
            .list_managers_events(current_user.id)
            .await?;
        for relation in event_manager_relations {
            let event = app_state
                .event_repository
                .get_event_by_id(relation.event_id)
                .await?;
            managed_events.push(event);
        }
        events.append(&mut managed_events);

        for event in events {
            let event_jobs = app_state
                .job_position_repository
                .list_job_positions(job_position::SelectManyFilter {
                    event_id: Some(event.id),
                    position_category_id: None,
                    salary: None,
                    currency: None,
                    capacity: None,
                    is_opened_for_registration: None,
                })
                .await?;
            for job in event_jobs {
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
