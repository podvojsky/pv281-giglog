use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::error::AppError;

pub mod get {
    use super::*;
    use crate::models::event::SelectManyFilter;
    use crate::repositories::event::EventRepository;
    use crate::repositories::user::UserRepository;
    use crate::repositories::venue::VenueRepository;
    use crate::templates::ManageEventsTemplate;
    use crate::{app::AppState, view_models::event::EventViewModel};

    pub async fn events(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let mut events: Vec<EventViewModel> = Vec::new();

        let empty_filter = SelectManyFilter {
            date_from: None,
            date_to: None,
            is_draft: None,
            venue_id: None,
            owner_id: None,
            city: None,
            state: None,
            name: None,
        };

        let all_events = app_state.event_repository.list_events(empty_filter).await?;

        for event in all_events {
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

        let html = template.render()?;
        Ok(Html(html))
    }
}
