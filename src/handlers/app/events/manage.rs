use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::{error::AppError, repositories::user::UserRepository};

pub mod get {
    use super::*;
    use crate::{
        app::AppState,
        error::ApiError,
        models::event::SelectManyFilter,
        repositories::{
            event::EventRepository, event_manager_relation::EventManagerRelationRepository,
            venue::VenueRepository,
        },
        templates::ManageEventsTemplate,
        view_models::event::EventViewModel,
    };

    pub async fn manage(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let current_user = match auth_session.clone().user {
            Some(user) => user,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };
        let mut events: Vec<EventViewModel> = Vec::new();
        let event_manager_relations = app_state
            .event_manager_relation_repository
            .list_managers_events(current_user.id)
            .await?;
        for relation in event_manager_relations {
            let event = app_state
                .event_repository
                .get_event_by_id(relation.event_id)
                .await?;
            let venue = app_state
                .venue_repository
                .get_venue_by_id(event.venue_id)
                .await?;
            let owner = app_state
                .user_repository
                .get_user_by_id(event.owner_id)
                .await?;
            events.push(EventViewModel {
                id: event.id,
                name: event.name,
                date_start: event.date_start,
                date_end: event.date_end,
                img_url: event.img_url,
                is_draft: event.is_draft,
                venue,
                owner,
            });
        }

        let owned_events = app_state
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
        for event in owned_events {
            let venue = app_state
                .venue_repository
                .get_venue_by_id(event.venue_id)
                .await?;
            let owner = app_state
                .user_repository
                .get_user_by_id(event.owner_id)
                .await?;
            events.push(EventViewModel {
                id: event.id,
                name: event.name,
                date_start: event.date_start,
                date_end: event.date_end,
                img_url: event.img_url,
                is_draft: event.is_draft,
                venue,
                owner,
            });
        }

        let template = ManageEventsTemplate {
            session: auth_session,
            active_route: Some(crate::templates::ActiveRoute::Manage),
            events,
        };
        let html = template.render().unwrap();
        Ok(Html(html))
    }
}
