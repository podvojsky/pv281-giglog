use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::error::AppError;

pub mod get {
    use super::*;
    use crate::{app::AppState, templates::CreateVenueTemplate};

    pub async fn create(
        auth_session: AuthSession,
        State(_app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let template = CreateVenueTemplate {
            session: auth_session,
            active_route: None,
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
        models::venue::CreateVenue,
        regex::{RE_POSTAL_CODE, RE_STREET_NUMBER},
        repositories::venue::VenueRepository,
        utils::response_utils::{generate_form_errors_response, generate_htmx_redirect},
    };
    use axum::{response::Response, Form};

    use serde::Deserialize;

    use validator::Validate;

    #[derive(Deserialize, Validate)]
    pub struct Params {
        #[validate(length(
            min = 3,
            max = 32,
            message = "Venue name has to be 3 to 32 characters long."
        ))]
        venue_name: String,
        #[validate(length(
            min = 3,
            max = 32,
            message = "State has to be 3 to 32 characters long."
        ))]
        state: String,
        #[validate(length(
            min = 3,
            max = 32,
            message = "Town has to be 3 to 32 characters long."
        ))]
        town: String,
        #[validate(regex(path = *RE_POSTAL_CODE, message = "Postal code is not in the correct format."))]
        postal_code: String,
        #[validate(length(
            min = 3,
            max = 32,
            message = "Street name has to be 3 to 32 characters long."
        ))]
        street_name: String,
        #[validate(regex(path = *RE_STREET_NUMBER, message = "Street number is not in the correct format."))]
        street_number: String,
        #[validate(url(message = "Address URL is not in the correct format."))]
        address_url: Option<String>,
        #[validate(length(
            max = 300,
            message = "Venue description is too long. Maximum is 300 characters."
        ))]
        description: Option<String>,
    }

    pub async fn create(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
        params: Form<Params>,
    ) -> Result<Response, AppError> {
        let _current_user = match auth_session.clone().user {
            Some(user) => user,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };
        match params.validate() {
            Ok(_) => (),
            Err(errors) => return Ok(generate_form_errors_response(errors)),
        }

        let _new_venue = app_state
            .venue_repository
            .create_venue(CreateVenue {
                name: params.venue_name.clone(),
                description: params.description.clone(),
                state: params.state.clone(),
                postal_code: params.postal_code.clone(),
                town: params.town.clone(),
                street_name: params.street_name.clone(),
                street_number: params.street_number.clone(),
                address_url: params.address_url.clone(),
            })
            .await?;

        Ok(generate_htmx_redirect("/manage/venues"))
    }
}
