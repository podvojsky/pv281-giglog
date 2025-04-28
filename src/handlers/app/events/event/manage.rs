use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::{error::AppError, repositories::user::UserRepository};

pub mod get {
    use super::*;
    use crate::{
        app::AppState,
        error::ApiError,
        models::{
            user::{self, User, UserRole},
            venue::SelectManyFilter,
        },
        repositories::{
            event::EventRepository, event_manager_relation::EventManagerRelationRepository,
            venue::VenueRepository,
        },
        templates::ManageEventTemplate,
        utils::response_utils::generate_unauthorized_response,
        view_models::event::ManageEventViewModel,
    };
    use askama_axum::IntoResponse;
    use axum::extract::Path;
    use axum::response::Response;

    pub async fn manage(
        Path(event_id): Path<i32>,
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Response, AppError> {
        let current_user = match auth_session.clone().user {
            Some(user) => user,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };

        let event = app_state.event_repository.get_event_by_id(event_id).await?;

        // Check if current user can update event
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

        let venue = app_state
            .venue_repository
            .get_venue_by_id(event.venue_id)
            .await?;
        let venues = app_state
            .venue_repository
            .list_venues(SelectManyFilter {
                name: None,
                description: None,
                state: None,
                postal_code: None,
                town: None,
                street_name: None,
                street_number: None,
            })
            .await?;
        let venues = venues
            .into_iter()
            .filter(|filter_venue| filter_venue.id != venue.id)
            .collect();
        let mut managers: Vec<User> = Vec::new();
        let event_manager_relations = app_state
            .event_manager_relation_repository
            .list_event_managers(event_id)
            .await?;
        for event_manager_relation in event_manager_relations {
            let manager = app_state
                .user_repository
                .get_user_by_id(event_manager_relation.user_id)
                .await?;
            managers.push(manager);
        }
        let possible_managers = app_state
            .user_repository
            .list_users(user::SelectManyFilter {
                first_name: None,
                last_name: None,
                username: None,
                gender: None,
                role: Some(UserRole::Organizer),
                tax_rate: None,
            })
            .await?;
        let possible_managers = possible_managers
            .into_iter()
            .filter(|possible_manager| {
                !managers
                    .iter()
                    .any(|manager| manager.id == possible_manager.id)
            })
            .collect();

        let template = ManageEventTemplate {
            session: auth_session,
            active_route: Some(crate::templates::ActiveRoute::Events),
            event: ManageEventViewModel {
                id: event.id,
                name: event.name,
                date_start: event.date_start,
                date_end: event.date_end,
                img_url: event.img_url,
                is_draft: event.is_draft,
                venue,
                owner: current_user,
                description: event.description,
            },
            venues,
            managers,
            possible_managers,
        };
        let html = template.render().unwrap();
        Ok(Html(html).into_response())
    }
}
