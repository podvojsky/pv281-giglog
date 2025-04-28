use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::error::AppError;

pub mod get {
    use super::*;
    use crate::{
        app::AppState, models::venue, repositories::venue::VenueRepository,
        templates::CreateEventTemplate,
    };

    pub async fn create(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let venues = app_state
            .venue_repository
            .list_venues(venue::SelectManyFilter {
                name: None,
                description: None,
                state: None,
                postal_code: None,
                town: None,
                street_name: None,
                street_number: None,
            })
            .await?;

        let template = CreateEventTemplate {
            session: auth_session,
            active_route: None,
            venues,
        };
        let html = template.render().unwrap();
        Ok(Html(html))
    }
}
