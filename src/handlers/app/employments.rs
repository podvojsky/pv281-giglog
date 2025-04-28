use crate::handlers::app::employments::post::SortColumn;
use crate::{
    app::AppState,
    error::{ApiError, AppError},
    handlers::app::auth::AuthSession,
    models::{
        employment::{all_employment_states, EmploymentState},
        job_position::SelectManyFilter,
    },
    repositories::{
        employment::EmploymentRepository, event::EventRepository,
        event_manager_relation::EventManagerRelationRepository,
        job_position::JobPositionRepository, user::UserRepository,
    },
    templates::{ActiveRoute, EmploymentsTableTemplate, EmploymentsTemplate},
    utils::table_utils::{optional_filter, parse_filter, SortDirection},
    view_models::employments::EmploymentViewModel,
};
use askama::Template;
use axum::{extract::State, response::Html, Form};
use serde::{Deserialize, Serialize};

pub(crate) async fn generate_employment_viewmodels(
    current_user_id: i32,
    app_state: &AppState,
) -> Result<Vec<EmploymentViewModel>, AppError> {
    let mut employments_viewmodels = Vec::new();

    let managed_events = app_state
        .event_manager_relation_repository
        .list_managers_events(current_user_id)
        .await
        .map_err(|err| {
            eprintln!("Failed to retrieve manager events list: {:?}", err);
            ApiError::NotFound
        })?;

    let event_filter = crate::models::event::SelectManyFilter {
        date_from: None,
        date_to: None,
        is_draft: None,
        venue_id: None,
        owner_id: Some(current_user_id),
        city: None,
        state: None,
        name: None,
    };

    let owned_events = app_state
        .event_repository
        .list_events(event_filter)
        .await
        .map_err(|err| {
            eprintln!("Failed to retrieve event: {:?}", err);
            ApiError::NotFound
        })?;

    for event in owned_events {
        employments_viewmodels.extend(process_event_employments(event, app_state).await?);
    }

    for managed_event in managed_events {
        let event = app_state
            .event_repository
            .get_event_by_id(managed_event.event_id)
            .await
            .map_err(|err| {
                eprintln!("Failed to retrieve event: {:?}", err);
                ApiError::NotFound
            })?;
        employments_viewmodels.extend(process_event_employments(event, app_state).await?);
    }

    Ok(employments_viewmodels)
}

async fn process_event_employments(
    event: crate::models::event::Event,
    app_state: &AppState,
) -> Result<Vec<EmploymentViewModel>, AppError> {
    let mut viewmodels = Vec::new();

    let job_position_filter = SelectManyFilter {
        event_id: Some(event.id),
        position_category_id: None,
        salary: None,
        currency: None,
        capacity: None,
        is_opened_for_registration: None,
    };

    let positions = app_state
        .job_position_repository
        .list_job_positions(job_position_filter)
        .await
        .map_err(|err| {
            eprintln!("Failed to retrieve job positions: {:?}", err);
            ApiError::NotFound
        })?;

    for position in positions {
        let employment_filter = crate::models::employment::SelectManyFilter {
            position_id: Some(position.id),
            user_id: None,
            state: None,
            rating: None,
        };

        let employments = app_state
            .employment_repository
            .list_employment(employment_filter)
            .await
            .map_err(|err| {
                eprintln!("Failed to retrieve employment list: {:?}", err);
                ApiError::NotFound
            })?;

        let current_capacity = employments
            .iter()
            .filter(|e| e.state == EmploymentState::Accepted || e.state == EmploymentState::Done)
            .count() as i32;

        for employment in employments {
            let employee = app_state
                .user_repository
                .get_user_by_id(employment.user_id)
                .await
                .map_err(|err| {
                    eprintln!("Failed to retrieve employee: {:?}", err);
                    ApiError::NotFound
                })?;

            viewmodels.push(EmploymentViewModel {
                state: employment.state,
                job_name: position.name.clone(),
                event_name: event.name.clone(),
                employee_name: employee.first_name + " " + employee.last_name.as_str(),
                employee_id: employee.id,
                event_id: event.id,
                employment_id: employment.id,
                max_capacity: position.capacity,
                current_capacity,
                rating: employment.rating,
            });
        }
    }
    Ok(viewmodels)
}

pub fn filter_and_sort_employments(
    employments: Vec<EmploymentViewModel>,
    employment_state: Option<EmploymentState>,
    event_filter: Option<String>,
    job_name_filter: Option<String>,
    employee_filter: Option<String>,
    sort_by: SortColumn,
    sort_direction: SortDirection,
) -> Vec<EmploymentViewModel> {
    let filtered_employments = employments
        .into_iter()
        .filter(|employment| match &employment_state {
            Some(state) => employment.state.eq(state),
            None => true,
        })
        .filter(|employment| match &event_filter {
            Some(event) => employment
                .event_name
                .to_lowercase()
                .contains(&event.to_lowercase()),
            None => true,
        })
        .filter(|employment| match &job_name_filter {
            Some(job_name) => employment
                .job_name
                .to_lowercase()
                .contains(&job_name.to_lowercase()),
            None => true,
        })
        .filter(|employment| match &employee_filter {
            Some(employee_name) => employment
                .employee_name
                .to_lowercase()
                .contains(&employee_name.to_lowercase()),
            None => true,
        })
        .collect::<Vec<EmploymentViewModel>>();

    let mut sorted_employments = filtered_employments;
    sorted_employments.sort_by(|a, b| {
        let cmp = match sort_by {
            SortColumn::State => a.state.cmp(&b.state),
            SortColumn::JobName => a.job_name.cmp(&b.job_name),
            SortColumn::Event => a.event_name.cmp(&b.event_name),
            SortColumn::EmployeeName => a.employee_name.cmp(&b.employee_name),
            SortColumn::Capacity => a.current_capacity.cmp(&b.current_capacity),
            SortColumn::Id => a.employment_id.cmp(&b.employment_id),
        };

        match sort_direction {
            SortDirection::Asc => cmp,
            SortDirection::Desc => cmp.reverse(),
        }
    });

    sorted_employments
}

pub mod get {
    use super::*;

    pub async fn employments(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let current_user = match auth_session.clone().user {
            Some(user) => user,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };

        let employments_viewmodels =
            generate_employment_viewmodels(current_user.id, &app_state).await?;

        let sorted_employments = filter_and_sort_employments(
            employments_viewmodels,
            None,
            None,
            None,
            None,
            SortColumn::Id,
            SortDirection::Asc,
        );

        let template = EmploymentsTemplate {
            session: auth_session,
            active_route: Some(ActiveRoute::Employments),
            employments: sorted_employments,
            employment_states: all_employment_states(),
        };
        Ok(Html(template.render().unwrap()))
    }
}

pub mod post {
    use super::*;
    use std::str::FromStr;

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum SortColumn {
        State,
        JobName,
        EmployeeName,
        Event,
        Capacity,
        Id,
    }

    #[allow(warnings)]
    impl Default for SortColumn {
        fn default() -> Self {
            SortColumn::Id
        }
    }

    #[derive(Deserialize)]
    pub struct FilterSortData {
        state: String,
        event: String,
        job_name: String,
        employee_name: String,
        sort_by: Option<SortColumn>,
        sort_direction: Option<SortDirection>,
    }

    pub async fn employments(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
        Form(payload): Form<FilterSortData>,
    ) -> Result<Html<String>, AppError> {
        let current_user = match auth_session.clone().user {
            Some(user) => user,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };

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
            generate_employment_viewmodels(current_user.id, &app_state).await?;

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

        Ok(Html(template.render().unwrap()))
    }
}
