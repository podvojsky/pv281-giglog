use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::Query,
    http::StatusCode,
    response::Response,
    response::{Html, Redirect},
    Form,
};
use serde::Deserialize;

use crate::{
    auth::{Backend, Credentials},
    error::AppError,
    regex::{RE_DATE, RE_PHONE_NUMBER},
    templates::{LoginViewTemplate, RegisterTemplate},
};

pub type AuthSession = axum_login::AuthSession<Backend>;

#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}

pub mod get {
    use super::*;

    pub async fn login(
        auth_session: AuthSession,
        Query(NextUrl { next }): Query<NextUrl>,
    ) -> Result<Response, AppError> {
        if auth_session.clone().user.is_some() {
            return Ok(Redirect::to("/").into_response());
        };

        let template = LoginViewTemplate {
            session: auth_session,
            next,
        };
        let html = template.render().unwrap();
        Ok(Html(html).into_response())
    }

    pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.logout().await {
            Ok(_) => Redirect::to("/").into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    pub async fn register() -> Result<Html<String>, AppError> {
        let template = RegisterTemplate {};
        let html = template.render().unwrap();
        Ok(Html(html))
    }
}

pub mod post {

    use crate::{
        app::AppState,
        models::user::{CreateUser, UserRole},
        repositories::user::UserRepository,
        templates::{RegisterSuccessTemplate, ToastType},
        utils::{
            date_utils::parse_date,
            response_utils::{
                generate_form_errors_response, generate_htmx_redirect, generate_toast_response,
            },
        },
    };

    use super::*;
    use crate::models::user::Gender;
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };
    use axum::{extract::State, response::Response};

    use sqlx::types::time::Date;
    use validator::Validate;

    pub async fn login(
        mut auth_session: AuthSession,
        Form(creds): Form<Credentials>,
    ) -> Result<Response, StatusCode> {
        let user = match auth_session.authenticate(creds.clone()).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                let mut login_url = "/login".to_string();
                if let Some(next) = creds.next {
                    login_url = format!("{}?next={}", login_url, next);
                    return Ok(generate_htmx_redirect(&login_url));
                };
                return Ok(generate_toast_response(
                    ToastType::Error,
                    "Login credentials are incorrect".to_string(),
                ));
            }
            Err(_) => {
                return Ok(generate_toast_response(
                    ToastType::Error,
                    "Login credentials are incorrect".to_string(),
                ));
            }
        };
        if auth_session.login(&user).await.is_err() {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        let redirect_url = creds.next.as_deref().unwrap_or("/");
        Ok(generate_htmx_redirect(redirect_url))
    }

    #[derive(Deserialize, Validate)]
    pub struct Params {
        #[validate(length(min = 3, message = "First Name has to be at least 3 characters long."))]
        first_name: String,
        #[validate(length(min = 3, message = "Last Name has to be at least 3 characters long."))]
        last_name: String,
        gender: Gender,
        #[validate(regex(path = *RE_DATE, message = "Date is not in the correct format."))]
        birth_date: String,
        #[validate(email(message = "Email is not in the correct format."))]
        email: String,
        #[validate(regex(path = *RE_PHONE_NUMBER, message = "Phone is not in the correct format."))]
        phone: String,
        #[validate(length(min = 3, message = "Username has to be at least 3 characters long."))]
        username: String,
        #[validate(must_match(
            other = "password_confirm",
            message = "Password and password confirmation must match."
        ))]
        #[validate(length(min = 8, message = "Password must be at least 8 characters long."))]
        password: String,
        password_confirm: String,
    }

    pub async fn register(
        State(app_state): State<AppState>,
        params: Form<Params>,
    ) -> Result<Response, StatusCode> {
        match params.validate() {
            Ok(_) => (),
            Err(errors) => return Ok(generate_form_errors_response(errors)),
        };
        let birth_date = match parse_date(&params.birth_date) {
            Ok(birth_date) => birth_date,
            Err(_) => Date::MIN,
        };
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
            role: UserRole::Employee,
            tax_rate: 0.15,
            avatar_url: None,
        };
        match new_user.validate() {
            Ok(_) => (),
            Err(errors) => return Ok(generate_form_errors_response(errors)),
        };
        let created_user_result = app_state.user_repository.create_user(new_user).await;
        match created_user_result {
            Ok(_) => (),
            Err(_) => {
                return Ok(generate_toast_response(
                    ToastType::Error,
                    format!("User with username '{}' already exists.", params.username),
                ));
            }
        }
        let template = RegisterSuccessTemplate {};
        let html = template.render().unwrap();
        Ok(Html(html).into_response())
    }
}
