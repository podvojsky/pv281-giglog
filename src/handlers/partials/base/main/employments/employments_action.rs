pub mod post {
    use crate::app::AppState;
    use crate::error::{ApiError, AppError};
    use crate::handlers::app::auth::AuthSession;
    use crate::handlers::app::employments::filter_and_sort_employments;
    use crate::handlers::app::employments::post::SortColumn;
    use crate::models::employment::{EmploymentState, PartialEmployment, SelectManyFilter};
    use crate::repositories::employment::EmploymentRepository;
    use crate::repositories::job_position::JobPositionRepository;
    use crate::templates::{EmploymentsTableTemplate, ToastTemplate};
    use crate::utils::table_utils::{optional_filter, parse_filter, SortDirection};
    use askama_axum::Template;
    use axum::extract::State;
    use axum::response::{Html, IntoResponse, Response};
    use axum::Form;
    use serde::Deserialize;
    use std::str::FromStr;

    #[derive(Deserialize)]
    pub struct Method {
        method: String,
        employment_id: i32,
        #[serde(default)]
        rating_value: i32,
        state: String,
        event: String,
        job_name: String,
        employee_name: String,
        sort_by: Option<SortColumn>,
        sort_direction: Option<SortDirection>,
    }

    async fn get_position_details(
        app_state: &AppState,
        employment_id: i32,
    ) -> Result<crate::models::job_position::JobPosition, ApiError> {
        let employment = app_state
            .employment_repository
            .get_employment_by_id(employment_id)
            .await
            .map_err(|err| {
                eprintln!("Failed to get employment: {:?}", err);
                ApiError::NotFound
            })?;

        let position = app_state
            .job_position_repository
            .get_job_position_by_id(employment.position_id)
            .await
            .map_err(|err| {
                eprintln!("Failed to get position: {:?}", err);
                ApiError::NotFound
            })?;

        Ok(position)
    }

    async fn get_current_job_capacity(
        app_state: &AppState,
        position_id: i32,
    ) -> Result<usize, ApiError> {
        let employments_filter = SelectManyFilter {
            position_id: Some(position_id),
            user_id: None,
            state: None,
            rating: None,
        };

        let employments = app_state
            .employment_repository
            .list_employment(employments_filter)
            .await
            .map_err(|err| {
                eprintln!("Failed to retrieve employment list: {:?}", err);
                ApiError::NotFound
            })?;

        Ok(employments
            .iter()
            .filter(|e| matches!(e.state, EmploymentState::Accepted | EmploymentState::Done))
            .count())
    }

    async fn update_employment_and_render(
        app_state: &AppState,
        employment_id: i32,
        update_values: PartialEmployment,
        Form(payload): Form<Method>,
        current_user_id: i32,
    ) -> Result<Response, ApiError> {
        app_state
            .employment_repository
            .update_employment(employment_id, update_values)
            .await
            .map_err(|err| {
                eprintln!("Failed to update employment: {:?}", err);
                ApiError::NotFound
            })?;

        let employment_state = parse_filter(
            payload.state.as_str(),
            |state| EmploymentState::from_str(state).map_err(|_| ApiError::NotFound),
            "All states",
        )?;
        let event_filter = optional_filter(payload.event);
        let job_name_filter = optional_filter(payload.job_name);
        let employee_filter = optional_filter(payload.employee_name);
        let sort_by = payload.sort_by.unwrap_or_default();
        let sort_direction = payload.sort_direction.unwrap_or_default();

        let employments_viewmodels =
            crate::handlers::app::employments::generate_employment_viewmodels(
                current_user_id,
                app_state,
            )
            .await
            .map_err(|_| ApiError::InternalServerError)?;

        let sorted_filtered_employments = filter_and_sort_employments(
            employments_viewmodels,
            employment_state,
            event_filter,
            job_name_filter,
            employee_filter,
            sort_by,
            sort_direction,
        );

        let template = EmploymentsTableTemplate {
            employments: sorted_filtered_employments,
        };

        Ok(template
            .render()
            .map(Html)
            .map_err(|_| ApiError::NotFound)
            .into_response())
    }

    pub async fn action(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
        Form(payload): Form<Method>,
    ) -> Result<Response, AppError> {
        let current_user = auth_session
            .user
            .ok_or_else(|| AppError::from(ApiError::InternalServerError))?;

        let position = get_position_details(&app_state, payload.employment_id).await?;
        let current_job_capacity = get_current_job_capacity(&app_state, position.id).await?;

        let update_values = match payload.method.as_str() {
            "Accept" => {
                if current_job_capacity >= position.capacity as usize {
                    let template = ToastTemplate {
                        toast_type: crate::templates::ToastType::Error,
                        message: "The job position has reached its maximum capacity.".to_string(),
                    };
                    let html = template
                        .render()
                        .map_err(|_| AppError::from(ApiError::InternalServerError))?;
                    return Ok(Response::builder()
                        .status(400)
                        .body(html)
                        .unwrap()
                        .into_response());
                }

                PartialEmployment {
                    rating: None,
                    state: Some(EmploymentState::Accepted),
                    user_id: None,
                    position_id: None,
                }
            }
            "Reject" => PartialEmployment {
                rating: None,
                state: Some(EmploymentState::Rejected),
                user_id: None,
                position_id: None,
            },
            "Finish" => PartialEmployment {
                rating: None,
                state: Some(EmploymentState::Done),
                user_id: None,
                position_id: None,
            },
            "Rating" => PartialEmployment {
                rating: Some(payload.rating_value),
                state: None,
                user_id: None,
                position_id: None,
            },
            _ => return Err(AppError::from(ApiError::NotFound)),
        };

        update_employment_and_render(
            &app_state,
            payload.employment_id,
            update_values,
            Form(payload),
            current_user.id,
        )
        .await
        .map_err(AppError::from)
    }
}
