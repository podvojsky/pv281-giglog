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
        app::AppState,
        error::ApiError,
        models::event::{Event, SelectManyFilter},
        repositories::{
            event::EventRepository, event_manager_relation::EventManagerRelationRepository,
            position_category::PositionCategoryRepository,
        },
        templates::CreateJobTemplate,
    };

    pub async fn create(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let current_user = match auth_session.clone().user {
            Some(user) => user,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };
        let mut events = app_state
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
        events.append(&mut managed_events);

        let job_categories = app_state
            .position_category_repository
            .list_position_categories()
            .await?;

        let template = CreateJobTemplate {
            session: auth_session,
            active_route: None,
            events,
            job_categories,
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
        models::{
            job_position::{CreateJobPosition, SalaryCurrency},
            user::UserRole,
        },
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

    pub async fn create(
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

        // Check if current user can create job in event
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

        let _new_job = app_state
            .job_position_repository
            .create_job_position(CreateJobPosition {
                name: params.job_name.clone(),
                description: params.description.clone(),
                salary,
                currency: SalaryCurrency::CZK,
                capacity,
                instructions_html: params.instructions.clone(),
                is_opened_for_registration: opened_for_registration,
                event_id,
                position_category_id: category_id,
            })
            .await?;

        Ok(generate_htmx_redirect("/manage/jobs"))
    }
}
