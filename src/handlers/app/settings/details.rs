use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::{
    error::AppError,
    regex::{RE_DATE, RE_PHONE_NUMBER},
    repositories::user::UserRepository,
};

use axum::response::Response;

pub mod get {

    use crate::{app::AppState, templates::SettingsDetailsTemplate};

    use super::*;

    pub async fn details(
        auth_session: AuthSession,
        State(_app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let template = SettingsDetailsTemplate {
            session: auth_session,
            active_route: None,
        };
        let html = template.render().unwrap();
        Ok(Html(html))
    }
}

pub mod patch {
    use axum::Form;
    use serde::Deserialize;
    use validator::Validate;

    use crate::{
        app::AppState,
        error::ApiError,
        models::user::{Gender, PartialUser},
        templates::ToastType,
        utils::{
            date_utils::parse_date,
            response_utils::{generate_form_errors_response, generate_toast_response},
        },
    };

    use super::*;

    #[derive(Deserialize, Validate)]
    pub struct Params {
        #[validate(length(min = 3, message = "First Name has to be at least 3 characters long."))]
        first_name: Option<String>,
        #[validate(length(min = 3, message = "Last Name has to be at least 3 characters long."))]
        last_name: Option<String>,
        gender: Gender,
        #[validate(regex(path = *RE_DATE, message = "Date is not in the correct format."))]
        birth_date: String,
        #[validate(email(message = "Email is not in the correct format."))]
        email: Option<String>,
        #[validate(regex(path = *RE_PHONE_NUMBER, message = "Phone is not in the correct format."))]
        phone: Option<String>,
        #[validate(url(message = "Avatar URL is not in the correct format."))]
        avatar_url: Option<String>,
    }

    pub async fn details(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
        params: Form<Params>,
    ) -> Result<Response, AppError> {
        match params.validate() {
            Ok(_) => (),
            Err(errors) => return Ok(generate_form_errors_response(errors)),
        };
        let birth_date = match parse_date(&params.birth_date) {
            Ok(birth_date) => Some(birth_date),
            Err(_) => None,
        };
        let current_user = match auth_session.clone().user {
            Some(user) => user,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };
        let _updated_user = app_state
            .user_repository
            .update_user(
                current_user.id,
                PartialUser {
                    first_name: params.first_name.clone(),
                    last_name: params.last_name.clone(),
                    username: None,
                    gender: Some(params.gender.clone()),
                    birth_date,
                    email: params.email.clone(),
                    phone: params.phone.clone(),
                    password_hash: None,
                    role: None,
                    tax_rate: None,
                    avatar_url: params.avatar_url.clone(),
                },
            )
            .await?;

        Ok(generate_toast_response(
            ToastType::Success,
            "User details were successfully updated.".to_string(),
        ))
    }
}
