use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::{error::AppError, repositories::user::UserRepository};

pub mod get {
    use axum::extract::Query;

    use serde::Deserialize;
    use sqlx::types::time::Date;

    use crate::{
        app::AppState,
        models::event::SelectManyFilter,
        repositories::{
            event::EventRepository, event_manager_relation::EventManagerRelationRepository,
            venue::VenueRepository,
        },
        templates::EventsContentTemplate,
        utils::date_utils::convert_date_time_to_date,
        view_models::event::{EventViewModel, IEventViewModel},
    };

    use super::*;

    #[derive(Deserialize, Debug)]
    pub struct Params {
        state: Option<String>,
        city: Option<String>,
        name: Option<String>,
    }

    pub async fn events_content(
        params: Query<Params>,
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let state = params.state.clone().filter(|state| !state.is_empty());
        let city = params.city.clone().filter(|city| !city.is_empty());
        let name = params.name.clone().filter(|name| !name.is_empty());
        let current_date_time = chrono::Local::now();
        let _states = app_state.venue_repository.list_states().await?;
        let _cities = app_state.venue_repository.list_cities().await?;
        let mut future_view_events: Vec<EventViewModel> = Vec::new();
        let future_events = app_state
            .event_repository
            .list_events(SelectManyFilter {
                date_from: Some(convert_date_time_to_date(current_date_time)),
                date_to: None,
                is_draft: None,
                venue_id: None,
                owner_id: None,
                city: city.clone(),
                state: state.clone(),
                name: name.clone(),
            })
            .await?;

        for event in future_events {
            let venue = app_state
                .venue_repository
                .get_venue_by_id(event.venue_id)
                .await?;
            let owner = app_state
                .user_repository
                .get_user_by_id(event.owner_id)
                .await?;

            // Check if user is owner or manager to show draft event.
            if event.is_draft {
                match auth_session.user {
                    Some(ref user) => {
                        let event_manager_relations = app_state
                            .event_manager_relation_repository
                            .list_event_managers(event.id)
                            .await?;

                        if event.owner_id != user.id {
                            let user_is_manager = event_manager_relations
                                .into_iter()
                                .any(|relation| relation.user_id == user.id);
                            if !user_is_manager {
                                continue;
                            }
                        }
                    }
                    None => continue,
                }
            }

            future_view_events.push(EventViewModel::new(event, venue, owner));
        }

        let mut past_view_events: Vec<EventViewModel> = Vec::new();
        let past_events = app_state
            .event_repository
            .list_events(SelectManyFilter {
                date_from: None,
                date_to: Some(
                    convert_date_time_to_date(current_date_time)
                        .previous_day()
                        .unwrap_or(Date::MIN),
                ),
                is_draft: None,
                venue_id: None,
                owner_id: None,
                city,
                state,
                name,
            })
            .await?;

        for event in past_events {
            let venue = app_state
                .venue_repository
                .get_venue_by_id(event.venue_id)
                .await?;
            let owner = app_state
                .user_repository
                .get_user_by_id(event.owner_id)
                .await?;

            // Check if user is owner or manager to show draft event.
            if event.is_draft {
                match auth_session.user {
                    Some(ref user) => {
                        let event_manager_relations = app_state
                            .event_manager_relation_repository
                            .list_event_managers(event.id)
                            .await?;

                        if event.owner_id != user.id {
                            let user_is_manager = event_manager_relations
                                .into_iter()
                                .any(|relation| relation.user_id == user.id);
                            if !user_is_manager {
                                continue;
                            }
                        }
                    }
                    None => continue,
                }
            }

            past_view_events.push(EventViewModel::new(event, venue, owner));
        }

        let template = EventsContentTemplate {
            session: auth_session,
            future_events: future_view_events,
            past_events: past_view_events,
        };
        let html = template.render().unwrap();
        Ok(Html(html))
    }
}
