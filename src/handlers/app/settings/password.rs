use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::{error::AppError, repositories::user::UserRepository};

pub mod get {

    use crate::{app::AppState, templates::SettingsPasswordTemplate};

    use super::*;

    pub async fn password(
        auth_session: AuthSession,
        State(_app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let template = SettingsPasswordTemplate {
            session: auth_session,
            active_route: None,
        };
        let html = template.render().unwrap();
        Ok(Html(html))
    }
}

pub mod patch {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };
    use axum::{response::Response, Form};
    use password_auth::verify_password;
    use serde::Deserialize;

    use validator::Validate;

    use crate::{
        app::AppState,
        error::ApiError,
        models::user::PartialUser,
        templates::ToastType,
        utils::response_utils::{generate_form_errors_response, generate_toast_response},
    };

    use super::*;

    #[derive(Deserialize, Validate)]
    pub struct Params {
        current_password: String,
        #[validate(length(min = 8, message = "Password must be at least 8 characters long."))]
        #[validate(must_match(
            other = "new_password_confirm",
            message = "New password and new password confirmation must match."
        ))]
        new_password: String,
        new_password_confirm: String,
    }

    pub async fn password(
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
        if verify_password(params.current_password.clone(), &current_user.password_hash).is_err() {
            return Ok(generate_toast_response(
                ToastType::Error,
                "Current password is incorrect.".to_string(),
            ));
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let new_password_hash = match argon2.hash_password(params.new_password.as_bytes(), &salt) {
            Ok(password_hash) => password_hash.to_string(),
            Err(_error) => {
                return Ok(generate_toast_response(
                    ToastType::Error,
                    "Cannot generate new password.".to_string(),
                ))
            }
        };
        let _updated_user = app_state
            .user_repository
            .update_user(
                current_user.id,
                PartialUser {
                    first_name: None,
                    last_name: None,
                    username: None,
                    gender: None,
                    birth_date: None,
                    email: None,
                    phone: None,
                    password_hash: Some(new_password_hash),
                    role: None,
                    tax_rate: None,
                    avatar_url: None,
                },
            )
            .await?;

        Ok(generate_toast_response(
            ToastType::Success,
            "Your password was successfully changed.".to_string(),
        ))
    }
}
