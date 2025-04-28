use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::{error::AppError, repositories::user::UserRepository, templates::EventTemplate};

pub mod create;
pub mod manage;

pub mod get {
    use super::*;
    use crate::{
        app::AppState,
        models::{
            employment::{self, EmploymentState},
            job_position::{JobPositionViewModel, SelectManyFilter},
        },
        repositories::{
            employment::EmploymentRepository, event::EventRepository,
            job_position::JobPositionRepository, position_category::PositionCategoryRepository,
            venue::VenueRepository,
        },
        utils::date_utils::is_date_in_past,
        view_models::event::EventDetailViewModel,
    };
    use askama_axum::IntoResponse;
    use axum::extract::Path;
    use axum::response::Response;

    pub async fn event(
        Path(event_id): Path<i32>,
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Response, AppError> {
        let current_user_id = auth_session.clone().user.map(|user| user.id);
        let event = app_state.event_repository.get_event_by_id(event_id).await?;
        let venue = app_state
            .venue_repository
            .get_venue_by_id(event.venue_id)
            .await?;
        let owner = app_state
            .user_repository
            .get_user_by_id(event.owner_id)
            .await?;
        let job_positions = app_state
            .job_position_repository
            .list_job_positions(SelectManyFilter {
                event_id: Some(event.id),
                position_category_id: None,
                salary: None,
                currency: None,
                capacity: None,
                is_opened_for_registration: None,
            })
            .await?;
        let mut job_positions_view_model_vec: Vec<JobPositionViewModel> = Vec::new();

        for job_position in job_positions {
            let job_category = app_state
                .position_category_repository
                .get_position_category_by_id(job_position.position_category_id)
                .await;
            let mut employments = app_state
                .employment_repository
                .list_employment(employment::SelectManyFilter {
                    position_id: Some(job_position.id),
                    user_id: None,
                    state: Some(EmploymentState::Accepted),
                    rating: None,
                })
                .await?;
            let mut employments_done = app_state
                .employment_repository
                .list_employment(employment::SelectManyFilter {
                    position_id: Some(job_position.id),
                    user_id: None,
                    state: Some(EmploymentState::Done),
                    rating: None,
                })
                .await?;
            employments.append(&mut employments_done);
            let employment_state = match current_user_id {
                Some(current_user_id) => {
                    app_state
                        .employment_repository
                        .list_employment(employment::SelectManyFilter {
                            position_id: Some(job_position.id),
                            user_id: Some(current_user_id),
                            state: None,
                            rating: None,
                        })
                        .await?
                }
                None => Vec::new(),
            };

            let employment_state = employment_state
                .first()
                .map(|employment| employment.state.clone());

            job_positions_view_model_vec.push(JobPositionViewModel {
                id: job_position.id,
                name: job_position.name,
                description: job_position.description,
                instructions_html: job_position.instructions_html,
                salary: job_position.salary,
                current_capacity: employments.len() as i32,
                max_capacity: job_position.capacity,
                is_opened_for_registration: job_position.is_opened_for_registration,
                employment_state,
                position_category: match job_category {
                    Ok(category) => Some(category),
                    Err(_error) => None,
                },
            });
        }

        let template = EventTemplate {
            session: auth_session,
            active_route: Some(crate::templates::ActiveRoute::Events),
            is_in_past: is_date_in_past(event.date_start),
            event: EventDetailViewModel {
                id: event.id,
                name: event.name,
                date_start: event.date_start,
                date_end: event.date_end,
                img_url: event.img_url,
                description: event.description,
                is_draft: event.is_draft,
                venue,
                owner,
                job_positions: job_positions_view_model_vec,
            },
        };
        let html = template.render().unwrap();
        Ok(Html(html).into_response())
    }
}

pub mod delete {
    use super::*;
    use crate::{
        app::AppState,
        error::ApiError,
        models::user::UserRole,
        repositories::{
            event::EventRepository, event_manager_relation::EventManagerRelationRepository,
        },
        utils::response_utils::generate_unauthorized_response,
    };
    use axum::extract::Path;
    use axum::response::Response;

    pub async fn event(
        Path(event_id): Path<i32>,
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Response, AppError> {
        let current_user = match auth_session.clone().user {
            Some(user) => user,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };

        // Check if current user can delete event
        let event = app_state.event_repository.get_event_by_id(event_id).await?;
        if current_user.id != event.owner_id && current_user.role != UserRole::Admin {
            let current_user_managed_events = app_state
                .event_manager_relation_repository
                .list_managers_events(current_user.id)
                .await?;
            if !current_user_managed_events
                .into_iter()
                .any(|event_manager_relation| event_manager_relation.event_id == event_id)
            {
                return Ok(generate_unauthorized_response());
            }
        }

        app_state.event_repository.delete_event(event_id).await?;

        Ok(Response::new("".into()))
    }
}
