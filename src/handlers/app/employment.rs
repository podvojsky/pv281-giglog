pub mod post {
    use askama::Template;
    use askama_axum::IntoResponse;
    use axum::{
        extract::State,
        response::{Html, Response},
        Form,
    };
    use serde::Deserialize;
    use validator::Validate;

    use crate::{
        app::AppState,
        error::AppError,
        handlers::app::auth::AuthSession,
        models::{
            employment::{self, CreateEmployment, EmploymentState},
            user::{self, UserRole},
        },
        repositories::{employment::EmploymentRepository, user::UserRepository},
        templates::{JobEmployeesTemplate, ToastType},
        utils::response_utils::generate_toast_response,
        view_models::jobs::ManageJobEmployeeViewModel,
    };

    #[derive(Deserialize, Validate)]
    pub struct Params {
        job_id: i32,
        employee_id: i32,
    }

    pub async fn employment(
        _auth_session: AuthSession,
        State(app_state): State<AppState>,
        params: Form<Params>,
    ) -> Result<Response, AppError> {
        let _new_employment = match app_state
            .employment_repository
            .create_employment(CreateEmployment {
                rating: 0,
                state: EmploymentState::Accepted,
                user_id: params.employee_id,
                position_id: params.job_id,
            })
            .await
        {
            Ok(new_employment) => new_employment,
            Err(_) => {
                return Ok(generate_toast_response(
                    ToastType::Error,
                    "Job capacity is full.".to_string(),
                ))
            }
        };

        let mut employees: Vec<ManageJobEmployeeViewModel> = Vec::new();
        let employments = app_state
            .employment_repository
            .list_employment(employment::SelectManyFilter {
                position_id: Some(params.job_id),
                user_id: None,
                state: Some(EmploymentState::Accepted),
                rating: None,
            })
            .await?;
        for employment in employments {
            let employee = app_state
                .user_repository
                .get_user_by_id(employment.user_id)
                .await?;
            employees.push(ManageJobEmployeeViewModel {
                id: employee.id,
                first_name: employee.first_name,
                last_name: employee.last_name,
                username: employee.username,
                gender: employee.gender,
                birth_date: employee.birth_date,
                email: employee.email,
                phone: employee.phone,
                password_hash: employee.password_hash,
                role: employee.role,
                tax_rate: employee.tax_rate,
                avatar_url: employee.avatar_url,
                employment,
            });
        }

        let possible_employees = app_state
            .user_repository
            .list_users(user::SelectManyFilter {
                first_name: None,
                last_name: None,
                username: None,
                gender: None,
                role: Some(UserRole::Employee),
                tax_rate: None,
            })
            .await?;
        let possible_employees = possible_employees
            .into_iter()
            .filter(|possible_employee| {
                !employees
                    .iter()
                    .any(|employee| employee.id == possible_employee.id)
            })
            .collect();

        let template = JobEmployeesTemplate {
            possible_employees,
            employees,
            job_id: params.job_id,
        };
        let html = template.render().unwrap();
        Ok(Html(html).into_response())
    }
}

pub mod delete {
    use askama::Template;
    use askama_axum::IntoResponse;
    use axum::{
        extract::{Query, State},
        response::{Html, Response},
    };
    use serde::Deserialize;
    use validator::Validate;

    use crate::{
        app::AppState,
        error::AppError,
        handlers::app::auth::AuthSession,
        models::{
            employment::{self, EmploymentState},
            user::{self, UserRole},
        },
        repositories::{employment::EmploymentRepository, user::UserRepository},
        templates::JobEmployeesTemplate,
        view_models::jobs::ManageJobEmployeeViewModel,
    };

    #[derive(Deserialize, Validate)]
    pub struct Params {
        job_id: i32,
        #[allow(dead_code)]
        employee_id: i32,
        employment_id: i32,
    }

    pub async fn employment(
        _auth_session: AuthSession,
        State(app_state): State<AppState>,
        params: Query<Params>,
    ) -> Result<Response, AppError> {
        app_state
            .employment_repository
            .delete_employment(params.employment_id)
            .await?;

        let mut employees: Vec<ManageJobEmployeeViewModel> = Vec::new();
        let employments = app_state
            .employment_repository
            .list_employment(employment::SelectManyFilter {
                position_id: Some(params.job_id),
                user_id: None,
                state: Some(EmploymentState::Accepted),
                rating: None,
            })
            .await?;
        for employment in employments {
            let employee = app_state
                .user_repository
                .get_user_by_id(employment.user_id)
                .await?;
            employees.push(ManageJobEmployeeViewModel {
                id: employee.id,
                first_name: employee.first_name,
                last_name: employee.last_name,
                username: employee.username,
                gender: employee.gender,
                birth_date: employee.birth_date,
                email: employee.email,
                phone: employee.phone,
                password_hash: employee.password_hash,
                role: employee.role,
                tax_rate: employee.tax_rate,
                avatar_url: employee.avatar_url,
                employment,
            });
        }

        let possible_employees = app_state
            .user_repository
            .list_users(user::SelectManyFilter {
                first_name: None,
                last_name: None,
                username: None,
                gender: None,
                role: Some(UserRole::Employee),
                tax_rate: None,
            })
            .await?;
        let possible_employees = possible_employees
            .into_iter()
            .filter(|possible_employee| {
                !employees
                    .iter()
                    .any(|employee| employee.id == possible_employee.id)
            })
            .collect();

        let template = JobEmployeesTemplate {
            possible_employees,
            employees,
            job_id: params.job_id,
        };
        let html = template.render().unwrap();
        Ok(Html(html).into_response())
    }
}
