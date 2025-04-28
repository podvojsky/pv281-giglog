use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html, Form};
use serde::Deserialize;
use std::str::FromStr;

use crate::{
    app::AppState,
    error::{ApiError, AppError},
    models::user::{SelectManyFilter, UserRole},
    repositories::user::UserRepository,
    templates::{ActiveRoute, AdminUsersTableTemplate, AdminUsersTemplate},
    utils::{
        date_utils::convert_date_time_to_date,
        table_utils::{optional_filter, parse_filter},
    },
    view_models::user::UserViewModel,
};

async fn create_user_viewmodels(
    users_filter: SelectManyFilter,
    app_state: AppState,
) -> Result<Vec<UserViewModel>, AppError> {
    let current_date = convert_date_time_to_date(chrono::Local::now());
    let all_users = app_state.user_repository.list_users(users_filter).await?;

    Ok(all_users
        .into_iter()
        .map(|user| {
            let age = ((current_date - user.birth_date).as_seconds_f32()
                / (60.0 * 60.0 * 24.0 * 365.25))
                .floor() as i32;

            UserViewModel {
                id: user.id,
                first_name: user.first_name,
                last_name: user.last_name,
                username: user.username,
                gender: user.gender,
                age,
                email: user.email,
                phone: user.phone,
                role: user.role,
                tax_rate: user.tax_rate,
                avatar_url: user.avatar_url,
            }
        })
        .collect())
}

pub mod get {
    use super::*;

    pub async fn users(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let users_filter = SelectManyFilter {
            first_name: None,
            last_name: None,
            username: None,
            gender: None,
            role: None,
            tax_rate: None,
        };

        let users = create_user_viewmodels(users_filter, app_state).await?;

        let template = AdminUsersTemplate {
            session: auth_session,
            active_route: Some(ActiveRoute::AdminPanel),
            users,
        };

        Ok(Html(template.render().unwrap()))
    }
}

pub mod post {
    use super::*;

    #[derive(Deserialize)]
    pub struct FilterData {
        role: String,
        last_name: String,
        username: String,
    }

    fn apply_filters(
        users: Vec<UserViewModel>,
        role: Option<UserRole>,
        last_name: Option<String>,
        username: Option<String>,
    ) -> Vec<UserViewModel> {
        users
            .into_iter()
            .filter(|user| match &role {
                Some(role) => user.role.eq(role),
                None => true,
            })
            .filter(|user| match &last_name {
                Some(user_last) => user
                    .last_name
                    .to_lowercase()
                    .contains(&user_last.to_lowercase()),
                None => true,
            })
            .filter(|user| match &username {
                Some(username) => user
                    .username
                    .to_lowercase()
                    .contains(&username.to_lowercase()),
                None => true,
            })
            .collect()
    }

    pub async fn users(
        State(app_state): State<AppState>,
        _auth_session: AuthSession,
        Form(payload): Form<FilterData>,
    ) -> Result<Html<String>, AppError> {
        let role = parse_filter(
            payload.role.as_str(),
            |state| UserRole::from_str(state).map_err(|_| ApiError::NotFound),
            "All roles",
        )?;

        let last_name = optional_filter(payload.last_name);
        let username = optional_filter(payload.username);

        let users_filter = SelectManyFilter {
            first_name: None,
            last_name: None,
            username: None,
            gender: None,
            role: None,
            tax_rate: None,
        };

        let users = create_user_viewmodels(users_filter, app_state).await?;

        let filtered_users = apply_filters(users, role, last_name, username);

        let template = AdminUsersTableTemplate {
            users: filtered_users,
        };

        Ok(Html(template.render().unwrap()))
    }
}
