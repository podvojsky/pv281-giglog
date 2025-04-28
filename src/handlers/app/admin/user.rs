use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::{error::AppError, repositories::user::UserRepository};

pub mod get {
    use super::*;
    use crate::app::AppState;
    use crate::templates::AdminUserDetailsTemplate;
    use axum::extract::Path;

    pub async fn user(
        Path(user_id): Path<i32>,
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let user = app_state.user_repository.get_user_by_id(user_id).await?;

        let template = AdminUserDetailsTemplate {
            session: auth_session,
            active_route: Some(crate::templates::ActiveRoute::AdminPanel),
            user,
        };
        let html = template.render().unwrap();
        Ok(Html(html))
    }
}

pub mod patch {
    use crate::regex::RE_DATE;
    use crate::regex::RE_PHONE_NUMBER;
    use argon2::password_hash::rand_core::OsRng;
    use argon2::password_hash::SaltString;
    use argon2::{Argon2, PasswordHasher};
    use askama_axum::Response;
    use axum::Form;
    use serde::Deserialize;
    use validator::Validate;

    use super::*;
    use crate::models::user::UserRole;
    use crate::{
        app::AppState,
        models::user::{Gender, PartialUser},
        templates::ToastType,
        utils::{
            date_utils::parse_date,
            response_utils::{generate_form_errors_response, generate_toast_response},
        },
    };

    #[derive(Deserialize, Validate)]
    pub struct Params {
        #[validate(length(min = 3, message = "First Name has to be at least 3 characters long."))]
        first_name: Option<String>,
        #[validate(length(min = 3, message = "Last Name has to be at least 3 characters long."))]
        last_name: Option<String>,
        gender: Gender,
        role: UserRole,
        #[validate(range(min = 0.0, max = 1.0, message = "Tax rate is out of bounds <0, 1>."))]
        tax_rate: Option<f32>,
        #[validate(regex(path = *RE_DATE, message = "Date is not in the correct format."))]
        birth_date: String,
        #[validate(email(message = "Email is not in the correct format."))]
        email: Option<String>,
        #[validate(regex(path = *RE_PHONE_NUMBER, message = "Phone is not in the correct format."))]
        phone: Option<String>,
        user_id: i32,
        new_password: String,
    }

    pub async fn user(
        _auth_session: AuthSession,
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

        let new_password_hash = if params.new_password.is_empty() {
            None
        } else {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            match argon2.hash_password(params.new_password.as_bytes(), &salt) {
                Ok(password_hash) => Some(password_hash.to_string()),
                Err(_) => {
                    return Ok(generate_toast_response(
                        ToastType::Error,
                        "Cannot generate new password.".to_string(),
                    ))
                }
            }
        };

        app_state
            .user_repository
            .update_user(
                params.user_id,
                PartialUser {
                    first_name: params.first_name.clone(),
                    last_name: params.last_name.clone(),
                    username: None,
                    gender: Some(params.gender.clone()),
                    birth_date,
                    email: params.email.clone(),
                    phone: params.phone.clone(),
                    password_hash: new_password_hash,
                    role: Some(params.role.clone()),
                    tax_rate: params.tax_rate,
                    avatar_url: None,
                },
            )
            .await?;

        Ok(generate_toast_response(
            ToastType::Success,
            "User data were successfully updated.".to_string(),
        ))
    }
}

pub mod delete {
    use super::*;
    use crate::app::AppState;
    use axum::extract::Path;
    use axum::http::StatusCode;

    pub async fn user(
        _auth_session: AuthSession,
        State(app_state): State<AppState>,
        Path(user_id): Path<i32>,
    ) -> Result<StatusCode, AppError> {
        app_state.user_repository.delete_user(user_id).await?;

        Ok(StatusCode::OK)
    }
}

pub mod get_create_template {
    use crate::error::AppError;
    use crate::handlers::app::auth::AuthSession;
    use crate::templates::ActiveRoute::AdminPanel;
    use crate::templates::AdminUserCreateTemplate;
    use askama_axum::Template;
    use axum::response::{Html, IntoResponse, Response};

    pub async fn user(auth_session: AuthSession) -> Result<Response, AppError> {
        let template = AdminUserCreateTemplate {
            session: auth_session,
            active_route: Some(AdminPanel),
        };
        let html = template.render().unwrap();
        Ok(Html(html).into_response())
    }
}

pub mod post {
    use super::*;
    use crate::app::AppState;
    use crate::models::user::{CreateUser, Gender, UserRole};
    use crate::regex::RE_DATE;
    use crate::regex::RE_PHONE_NUMBER;
    use crate::templates::ToastType;
    use crate::utils::date_utils::parse_date;
    use crate::utils::response_utils::{generate_form_errors_response, generate_toast_response};
    use argon2::password_hash::rand_core::OsRng;
    use argon2::password_hash::SaltString;
    use argon2::{Argon2, PasswordHasher};
    use askama_axum::{IntoResponse, Response};
    use axum::http::StatusCode;
    use axum::response::AppendHeaders;
    use axum::Form;
    use serde::Deserialize;
    use sqlx::types::time::Date;
    use validator::Validate;

    #[derive(Deserialize, Validate)]
    pub struct Params {
        #[validate(length(min = 3, message = "First Name has to be at least 3 characters long."))]
        first_name: String,
        #[validate(length(min = 3, message = "Last Name has to be at least 3 characters long."))]
        last_name: String,
        gender: Gender,
        role: UserRole,
        #[validate(regex(path = *RE_DATE, message = "Date is not in the correct format."))]
        birth_date: String,
        #[validate(email(message = "Email is not in the correct format."))]
        email: String,
        #[validate(regex(path = *RE_PHONE_NUMBER, message = "Phone is not in the correct format."))]
        phone: String,
        #[validate(length(min = 3, message = "Username has to be at least 3 characters long."))]
        username: String,
        #[validate(length(min = 8, message = "Password must be at least 8 characters long."))]
        password: String,
        #[validate(range(min = 0.0, max = 1.0, message = "Tax rate is out of bounds <0, 1>."))]
        tax_rate: Option<f32>,
    }

    pub async fn register(
        State(app_state): State<AppState>,
        params: Form<Params>,
    ) -> Result<Response, StatusCode> {
        match params.validate() {
            Ok(_) => (),
            Err(errors) => return Ok(generate_form_errors_response(errors)),
        };
        let birth_date = parse_date(&params.birth_date).unwrap_or(Date::MIN);
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let new_password_hash = match argon2.hash_password(params.password.as_bytes(), &salt) {
            Ok(password_hash) => password_hash.to_string(),
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        };

        let new_user = CreateUser {
            first_name: params.first_name.clone(),
            last_name: params.last_name.clone(),
            username: params.username.clone(),
            gender: params.gender.clone(),
            birth_date,
            email: params.email.clone(),
            phone: params.phone.clone(),
            password_hash: new_password_hash,
            role: params.role.clone(),
            tax_rate: params.tax_rate.unwrap_or(0.15),
            avatar_url: None,
        };

        match new_user.validate() {
            Ok(_) => (),
            Err(errors) => return Ok(generate_form_errors_response(errors)),
        };
        let created_user_result = app_state.user_repository.create_user(new_user).await;
        match created_user_result {
            Ok(_) => {
                let headers = AppendHeaders([("HX-Redirect", "/admin/users")]);
                Ok((headers, "").into_response())
            }
            Err(_) => Ok(generate_toast_response(
                ToastType::Error,
                format!("User with username '{}' already exists.", params.username),
            )),
        }
    }
}
