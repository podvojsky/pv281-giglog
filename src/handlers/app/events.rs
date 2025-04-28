use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::{error::AppError, repositories::user::UserRepository, templates::EventsTemplate};

pub mod event;
pub mod manage;

pub mod get {
    use crate::{
        app::AppState,
        models::event::SelectManyFilter,
        repositories::{
            event::EventRepository, event_manager_relation::EventManagerRelationRepository,
            venue::VenueRepository,
        },
        utils::date_utils::convert_date_time_to_date,
        view_models::event::{EventViewModel, IEventViewModel},
    };

    use sqlx::types::{chrono, time::Date};

    use super::*;

    pub async fn events(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let current_date_time = chrono::Local::now();
        let states = app_state.venue_repository.list_states().await?;
        let cities = app_state.venue_repository.list_cities().await?;
        let mut future_view_events: Vec<EventViewModel> = Vec::new();
        let future_events = app_state
            .event_repository
            .list_events(SelectManyFilter {
                date_from: Some(convert_date_time_to_date(current_date_time)),
                date_to: None,
                is_draft: None,
                venue_id: None,
                owner_id: None,
                city: None,
                state: None,
                name: None,
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
                city: None,
                state: None,
                name: None,
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

        let template = EventsTemplate {
            session: auth_session,
            active_route: Some(crate::templates::ActiveRoute::Events),
            future_events: future_view_events,
            past_events: past_view_events,
            states,
            cities,
        };
        let html = template.render().unwrap();
        Ok(Html(html))
    }
}

pub mod post {
    use super::*;
    use crate::{
        app::AppState,
        error::ApiError,
        models::event::CreateEvent,
        repositories::event::EventRepository,
        templates::ToastType,
        utils::{
            date_utils::parse_date,
            response_utils::{
                generate_form_errors_response, generate_htmx_redirect, generate_toast_response,
                CheckboxState,
            },
        },
    };
    use axum::response::Response;
    use axum::Form;

    use serde::Deserialize;

    use validator::Validate;

    #[derive(Deserialize, Validate)]
    pub struct Params {
        #[validate(length(
            min = 3,
            max = 32,
            message = "Event name has to be 3 to 32 characters long."
        ))]
        event_name: String,
        create_as_draft: Option<CheckboxState>,
        #[validate(length(min = 1, message = "Beginning date is required."))]
        date_start: String,
        #[validate(length(min = 1, message = "End date is required."))]
        date_end: String,
        #[validate(url(message = "Hero image URL is not in the correct format."))]
        hero_img_url: String,
        venue_id: Option<i32>,
        #[validate(length(
            max = 300,
            message = "Event description is too long. Maximum is 300 characters."
        ))]
        description: String,
    }

    pub async fn events(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
        params: Form<Params>,
    ) -> Result<Response, AppError> {
        let current_user = match auth_session.clone().user {
            Some(user) => user,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };
        match params.validate() {
            Ok(_) => (),
            Err(errors) => return Ok(generate_form_errors_response(errors)),
        }
        let create_as_draft = match params.create_as_draft {
            Some(_create_as_draft) => true,
            None => false,
        };
        let venue_id = match params.venue_id {
            Some(venue_id) => venue_id,
            None => {
                return Ok(generate_toast_response(
                    ToastType::Error,
                    "Venue is required.".to_string(),
                ))
            }
        };
        let date_start = match parse_date(&params.date_start) {
            Ok(date_start) => date_start,
            Err(_) => {
                return Ok(generate_toast_response(
                    ToastType::Error,
                    "Beginning date in not in the correct format.".to_string(),
                ))
            }
        };
        let date_end = match parse_date(&params.date_end) {
            Ok(date_end) => date_end,
            Err(_) => {
                return Ok(generate_toast_response(
                    ToastType::Error,
                    "End date in not in the correct format.".to_string(),
                ))
            }
        };
        if date_start > date_end {
            return Ok(generate_toast_response(
                ToastType::Error,
                "Beginning date cannot be later than end date.".to_string(),
            ));
        }

        let _new_event = app_state
            .event_repository
            .create_event(CreateEvent {
                name: params.event_name.clone(),
                date_start,
                date_end,
                img_url: params.hero_img_url.clone(),
                description: params.description.clone(),
                is_draft: create_as_draft,
                venue_id,
                owner_id: current_user.id,
            })
            .await?;
        Ok(generate_htmx_redirect("/manage/events"))
    }
}

pub mod patch {
    use super::*;
    use crate::{
        app::AppState,
        error::ApiError,
        models::{event::PartialEvent, user::UserRole},
        repositories::{
            event::EventRepository, event_manager_relation::EventManagerRelationRepository,
        },
        templates::ToastType,
        utils::{
            date_utils::parse_date,
            response_utils::{
                generate_form_errors_response, generate_htmx_redirect, generate_toast_response,
                generate_unauthorized_response, CheckboxState,
            },
        },
    };
    use axum::response::Response;
    use axum::Form;

    use serde::Deserialize;

    use validator::Validate;

    #[derive(Deserialize, Validate)]
    pub struct Params {
        #[validate(length(
            min = 3,
            max = 32,
            message = "Event name has to be 3 to 32 characters long."
        ))]
        event_name: String,
        create_as_draft: Option<CheckboxState>,
        #[validate(length(min = 1, message = "Beginning date is required."))]
        date_start: String,
        #[validate(length(min = 1, message = "End date is required."))]
        date_end: String,
        #[validate(url(message = "Hero image URL is not in the correct format."))]
        hero_img_url: String,
        venue_id: Option<i32>,
        #[validate(length(
            max = 300,
            message = "Event description is too long. Maximum is 300 characters."
        ))]
        description: String,
        event_id: i32,
    }

    pub async fn events(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
        params: Form<Params>,
    ) -> Result<Response, AppError> {
        let current_user = match auth_session.clone().user {
            Some(user) => user,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };

        // Check if current user can update event
        let event = app_state
            .event_repository
            .get_event_by_id(params.event_id)
            .await?;
        if current_user.id != event.owner_id && current_user.role != UserRole::Admin {
            let current_user_managed_events = app_state
                .event_manager_relation_repository
                .list_managers_events(current_user.id)
                .await?;
            if !current_user_managed_events
                .into_iter()
                .any(|event_manager_relation| event_manager_relation.event_id == params.event_id)
            {
                return Ok(generate_unauthorized_response());
            }
        }

        match params.validate() {
            Ok(_) => (),
            Err(errors) => return Ok(generate_form_errors_response(errors)),
        }
        let create_as_draft = match params.create_as_draft {
            Some(_create_as_draft) => true,
            None => false,
        };
        let venue_id = match params.venue_id {
            Some(venue_id) => venue_id,
            None => {
                return Ok(generate_toast_response(
                    ToastType::Error,
                    "Venue is required.".to_string(),
                ))
            }
        };
        let date_start = match parse_date(&params.date_start) {
            Ok(date_start) => date_start,
            Err(_) => {
                return Ok(generate_toast_response(
                    ToastType::Error,
                    "Beginning date in not in the correct format.".to_string(),
                ))
            }
        };
        let date_end = match parse_date(&params.date_end) {
            Ok(date_end) => date_end,
            Err(_) => {
                return Ok(generate_toast_response(
                    ToastType::Error,
                    "End date in not in the correct format.".to_string(),
                ))
            }
        };
        if date_start > date_end {
            return Ok(generate_toast_response(
                ToastType::Error,
                "Beginning date cannot be later than end date.".to_string(),
            ));
        }

        let _updated_event = app_state
            .event_repository
            .update_event(
                params.event_id,
                PartialEvent {
                    name: Some(params.event_name.clone()),
                    date_start: Some(date_start),
                    date_end: Some(date_end),
                    img_url: Some(params.hero_img_url.clone()),
                    description: Some(params.description.clone()),
                    is_draft: Some(create_as_draft),
                    venue_id: Some(venue_id),
                    owner_id: None,
                },
            )
            .await?;
        Ok(generate_htmx_redirect("/manage/events"))
    }
}
