use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{
    extract::State,
    response::{Html, Response},
};

use crate::error::AppError;

pub mod get {
    use super::*;
    use crate::{
        app::AppState, repositories::venue::VenueRepository, templates::ManageVenueTemplate,
    };
    use axum::extract::Path;

    pub async fn manage(
        Path(venue_id): Path<i32>,
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let venue = app_state.venue_repository.get_venue_by_id(venue_id).await?;

        let template = ManageVenueTemplate {
            session: auth_session,
            active_route: None,
            venue,
        };
        let html = template.render().unwrap();
        Ok(Html(html))
    }
}

pub mod patch {
    use super::*;
    use crate::{
        app::AppState,
        models::venue::PartialVenue,
        regex::{RE_POSTAL_CODE, RE_STREET_NUMBER},
        repositories::venue::VenueRepository,
        utils::response_utils::{generate_form_errors_response, generate_htmx_redirect},
    };
    use axum::Form;

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
        venue_id: i32,
    }

    pub async fn manage(
        _auth_session: AuthSession,
        State(app_state): State<AppState>,
        params: Form<Params>,
    ) -> Result<Response, AppError> {
        match params.validate() {
            Ok(_) => (),
            Err(errors) => return Ok(generate_form_errors_response(errors)),
        };

        let _updated_venue = app_state
            .venue_repository
            .update_venue(
                params.venue_id,
                PartialVenue {
                    name: Some(params.venue_name.clone()),
                    description: params.description.clone(),
                    state: Some(params.state.clone()),
                    postal_code: Some(params.postal_code.clone()),
                    town: Some(params.town.clone()),
                    street_name: Some(params.street_name.clone()),
                    street_number: Some(params.street_number.clone()),
                    address_url: params.address_url.clone(),
                },
            )
            .await?;

        Ok(generate_htmx_redirect("/manage/venues"))
    }
}

pub mod delete {
    use super::*;
    use crate::{
        app::AppState,
        repositories::venue::VenueRepository,
        templates::{ToastTemplate, ToastType},
    };
    use askama_axum::IntoResponse;
    use axum::extract::Path;

    pub async fn manage(
        Path(venue_id): Path<i32>,
        _auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Response, AppError> {
        match app_state.venue_repository.delete_venue(venue_id).await {
            Ok(_) => Ok(Response::new("".into())),
            Err(_) => {
                let template = ToastTemplate {
                    toast_type: ToastType::Error,
                    message: "Venue cannot be deleted, because it is associated with other events."
                        .to_string(),
                };
                let html = template.render().unwrap();

                Ok(Response::builder()
                    .header("HX-Reswap", "innerHTML")
                    .status(400)
                    .body(html)
                    .unwrap()
                    .into_response())
            }
        }
    }
}
