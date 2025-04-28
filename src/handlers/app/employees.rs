use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::{error::AppError, repositories::user::UserRepository, templates::EmployeesTemplate};

pub mod employee;

pub mod get {
    use crate::{
        app::AppState,
        models::user::{SelectManyFilter, UserRole},
        utils::date_utils::convert_date_time_to_date,
        view_models::user::UserViewModel,
    };

    use super::*;

    pub async fn employees(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let current_date_time = chrono::Local::now();
        let current_date = convert_date_time_to_date(current_date_time);
        let mut users_view_model_vec: Vec<UserViewModel> = Vec::new();
        let users = app_state
            .user_repository
            .list_users(SelectManyFilter {
                first_name: None,
                last_name: None,
                username: None,
                gender: None,
                role: Some(UserRole::Employee),
                tax_rate: None,
            })
            .await?;

        for user in users {
            let age =
                (current_date - user.birth_date).as_seconds_f32() / (60.0 * 60.0 * 24.0 * 365.25);
            let age = age.floor() as i32;

            users_view_model_vec.push(UserViewModel {
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
            });
        }

        let template = EmployeesTemplate {
            session: auth_session,
            active_route: Some(crate::templates::ActiveRoute::Employees),
            employees: users_view_model_vec,
        };
        let html = template.render().unwrap();
        Ok(Html(html))
    }
}
