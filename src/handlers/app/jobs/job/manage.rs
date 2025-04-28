use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{
    extract::State,
    response::{Html, Response},
};

use crate::{error::AppError, repositories::user::UserRepository};

pub mod get {
    use super::*;
    use crate::{
        app::AppState,
        error::ApiError,
        models::{
            employment::{self, EmploymentState},
            event::{Event, SelectManyFilter},
            job_position::SalaryCurrency,
            user::{self, UserRole},
        },
        repositories::{
            employment::EmploymentRepository, event::EventRepository,
            event_manager_relation::EventManagerRelationRepository,
            job_position::JobPositionRepository, position_category::PositionCategoryRepository,
        },
        templates::ManageJobTemplate,
        utils::response_utils::generate_unauthorized_response,
        view_models::jobs::{ManageJobEmployeeViewModel, ManageJobPositionViewModel},
    };
    use askama_axum::IntoResponse;
    use axum::extract::Path;

    pub async fn manage(
        Path(job_id): Path<i32>,
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Response, AppError> {
        let current_user = match auth_session.clone().user {
            Some(user) => user,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };

        let job = app_state
            .job_position_repository
            .get_job_position_by_id(job_id)
            .await?;
        let event = app_state
            .event_repository
            .get_event_by_id(job.event_id)
            .await?;
        let category = app_state
            .position_category_repository
            .get_position_category_by_id(job.position_category_id)
            .await?;

        // Check if current user can create job in event
        if current_user.id != event.owner_id && current_user.role != UserRole::Admin {
            let current_user_managed_events = app_state
                .event_manager_relation_repository
                .list_managers_events(current_user.id)
                .await?;
            if !current_user_managed_events
                .into_iter()
                .any(|event_manager_relation| event_manager_relation.event_id == event.id)
            {
                return Ok(generate_unauthorized_response());
            }
        }

        let mut possible_events = app_state
            .event_repository
            .list_events(SelectManyFilter {
                date_from: None,
                date_to: None,
                is_draft: None,
                venue_id: None,
                owner_id: Some(current_user.id),
                city: None,
                state: None,
                name: None,
            })
            .await?;
        let mut managed_events: Vec<Event> = Vec::new();
        let event_manager_relations = app_state
            .event_manager_relation_repository
            .list_managers_events(current_user.id)
            .await?;
        for relation in event_manager_relations {
            let event = app_state
                .event_repository
                .get_event_by_id(relation.event_id)
                .await?;
            managed_events.push(event);
        }
        possible_events.append(&mut managed_events);
        let possible_events = possible_events
            .into_iter()
            .filter(|possible_event| possible_event.id != event.id)
            .collect();

        let possible_job_categories = app_state
            .position_category_repository
            .list_position_categories()
            .await?;
        let possible_job_categories = possible_job_categories
            .into_iter()
            .filter(|possible_category| possible_category.id != category.id)
            .collect();

        let mut employees: Vec<ManageJobEmployeeViewModel> = Vec::new();
        let employments = app_state
            .employment_repository
            .list_employment(employment::SelectManyFilter {
                position_id: Some(job_id),
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

        let template = ManageJobTemplate {
            session: auth_session,
            active_route: None,
            events: possible_events,
            job_categories: possible_job_categories,
            job: ManageJobPositionViewModel {
                id: job.id,
                name: job.name,
                salary: job.salary,
                capacity: job.capacity,
                is_opened_for_registration: job.is_opened_for_registration,
                currency: SalaryCurrency::CZK,
                event,
                category,
                instructions: job.instructions_html,
                description: job.description,
            },
            employees,
            possible_employees,
        };
        let html = template.render().unwrap();
        Ok(Html(html).into_response())
    }
}

pub mod patch {
    use super::*;
    use crate::{
        app::AppState,
        error::ApiError,
        models::{job_position::PartialJobPosition, user::UserRole},
        repositories::{
            event::EventRepository, event_manager_relation::EventManagerRelationRepository,
            job_position::JobPositionRepository,
        },
        templates::ToastType,
        utils::response_utils::{
            generate_form_errors_response, generate_htmx_redirect, generate_toast_response,
            generate_unauthorized_response, CheckboxState,
        },
    };
    use axum::Form;
    use bigdecimal::Signed;

    use serde::Deserialize;

    use validator::Validate;

    #[derive(Deserialize, Validate)]
    pub struct Params {
        #[validate(length(
            min = 3,
            max = 32,
            message = "Job name has to be 3 to 32 characters long."
        ))]
        job_name: String,
        opened_for_registration: Option<CheckboxState>,
        #[validate(length(min = 1, message = "Salary is required."))]
        salary: String,
        #[validate(length(min = 1, message = "Capacity is required."))]
        capacity: String,
        job_id: i32,
        event_id: Option<i32>,
        category_id: Option<i32>,
        #[validate(length(
            max = 300,
            message = "Job instructions are too long. Maximum is 300 characters."
        ))]
        instructions: String,
        #[validate(length(
            max = 300,
            message = "Job description is too long. Maximum is 300 characters."
        ))]
        description: String,
    }

    pub async fn manage(
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
        let opened_for_registration = match params.opened_for_registration {
            Some(_opened_for_registration) => true,
            None => false,
        };
        let salary = match params.salary.parse::<f32>() {
            Ok(salary) => salary,
            Err(_) => {
                return Ok(generate_toast_response(
                    ToastType::Error,
                    "Salary must be a number".to_string(),
                ))
            }
        };
        if salary.is_negative() {
            return Ok(generate_toast_response(
                ToastType::Error,
                "Salary must be positive.".to_string(),
            ));
        }

        let capacity = match params.capacity.parse::<i32>() {
            Ok(capacity) => capacity,
            Err(_) => {
                return Ok(generate_toast_response(
                    ToastType::Error,
                    "Capacity must be a number".to_string(),
                ))
            }
        };
        if capacity <= 0 {
            return Ok(generate_toast_response(
                ToastType::Error,
                "Capacity must be positive.".to_string(),
            ));
        }

        let event_id = match params.event_id {
            Some(event_id) => event_id,
            None => {
                return Ok(generate_toast_response(
                    ToastType::Error,
                    "Event is required.".to_string(),
                ))
            }
        };
        let category_id = match params.category_id {
            Some(category_id) => category_id,
            None => {
                return Ok(generate_toast_response(
                    ToastType::Error,
                    "Job category is required.".to_string(),
                ))
            }
        };

        // Check if current user can update job in event
        let event = app_state.event_repository.get_event_by_id(event_id).await?;
        if current_user.id != event.owner_id && current_user.role != UserRole::Admin {
            let current_user_managed_events = app_state
                .event_manager_relation_repository
                .list_managers_events(current_user.id)
                .await?;
            if !current_user_managed_events
                .into_iter()
                .any(|event_manager_relation| event_manager_relation.event_id == event_id)
            {
                return Ok(generate_unauthorized_response());
            }
        }

        let _updated_job = app_state
            .job_position_repository
            .update_job_position(
                params.job_id,
                PartialJobPosition {
                    name: Some(params.job_name.clone()),
                    description: Some(params.description.clone()),
                    salary: Some(salary),
                    currency: None,
                    capacity: Some(capacity),
                    instructions_html: Some(params.instructions.clone()),
                    is_opened_for_registration: Some(opened_for_registration),
                    event_id: Some(event_id),
                    position_category_id: Some(category_id),
                },
            )
            .await?;

        Ok(generate_htmx_redirect("/manage/jobs"))
    }
}

pub mod delete {
    use super::*;
    use crate::{
        app::AppState,
        error::ApiError,
        models::user::UserRole,
        repositories::{
            event::EventRepository, event_manager_relation::EventManagerRelationRepository,
            job_position::JobPositionRepository,
        },
        utils::response_utils::generate_unauthorized_response,
    };
    use axum::extract::Path;

    pub async fn manage(
        Path(job_id): Path<i32>,
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Response, AppError> {
        let current_user = match auth_session.clone().user {
            Some(user) => user,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };

        // Check if current user can delete job in event
        let job = app_state
            .job_position_repository
            .get_job_position_by_id(job_id)
            .await?;
        let event = app_state
            .event_repository
            .get_event_by_id(job.event_id)
            .await?;
        if current_user.id != event.owner_id && current_user.role != UserRole::Admin {
            let current_user_managed_events = app_state
                .event_manager_relation_repository
                .list_managers_events(current_user.id)
                .await?;
            if !current_user_managed_events
                .into_iter()
                .any(|event_manager_relation| event_manager_relation.event_id == job.event_id)
            {
                return Ok(generate_unauthorized_response());
            }
        }

        app_state
            .job_position_repository
            .delete_job_position(job_id)
            .await?;
        Ok(Response::new("".into()))
    }
}
