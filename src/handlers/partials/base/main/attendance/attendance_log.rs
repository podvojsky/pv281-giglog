use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::error::AppError;

use askama_axum::IntoResponse;

pub mod get {
    use std::collections::BTreeMap;

    use axum::extract::Query;

    use serde::Deserialize;
    use sqlx::types::time::Date;

    use crate::{
        app::AppState,
        error::ApiError,
        models::{
            employment::{self, EmploymentState},
            worked_hours::{self, WorkedHours},
        },
        repositories::{
            employment::EmploymentRepository, event::EventRepository,
            job_position::JobPositionRepository, worked_hours::WorkedHoursRepository,
        },
        templates::AttendanceLogTemplate,
        utils::date_utils::from_date_range_to_vec,
    };

    use super::*;

    #[derive(Deserialize)]
    pub struct Params {
        job_position_id: i32,
    }

    pub async fn attendance_log(
        params: Query<Params>,
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let current_user = match auth_session.clone().user {
            Some(user) => user,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };

        let job = app_state
            .job_position_repository
            .get_job_position_by_id(params.job_position_id)
            .await?;
        let event = app_state
            .event_repository
            .get_event_by_id(job.event_id)
            .await?;

        let employment = app_state
            .employment_repository
            .list_employment(employment::SelectManyFilter {
                position_id: Some(params.job_position_id),
                user_id: Some(current_user.id),
                state: Some(EmploymentState::Accepted),
                rating: None,
            })
            .await?;
        let employment = employment.first();
        let employment = match employment {
            Some(employment) => employment,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };

        let mut attendance_log: BTreeMap<Date, Option<WorkedHours>> = BTreeMap::new();
        let date_range = from_date_range_to_vec(event.date_start, event.date_end);
        let worked_hours = app_state
            .worked_hours_repository
            .list_worked_hours(worked_hours::SelectManyFilter {
                hours_worked: None,
                date: None,
                employment_id: Some(employment.id),
            })
            .await?;

        for date in date_range {
            attendance_log.entry(date).or_insert(None);
        }
        for hours in worked_hours {
            *attendance_log.entry(hours.date).or_insert(None) = Some(hours.clone());
        }

        let template = AttendanceLogTemplate {
            session: auth_session,
            attendance_log,
            employment: employment.clone(),
        };
        let html = template.render().unwrap();
        Ok(Html(html))
    }
}

pub mod patch {

    use axum::{response::Response, Form};

    use serde::Deserialize;
    use sqlx::types::time::Date;

    use validator::Validate;

    use crate::{
        app::AppState,
        error::ApiError,
        models::{
            employment::{self},
            worked_hours::{CreateWorkedHours, PartialWorkedHours},
        },
        repositories::{employment::EmploymentRepository, worked_hours::WorkedHoursRepository},
        templates::{HoursWorkedInputTemplate, ToastType},
        utils::response_utils::{
            generate_form_errors_response, generate_toast_response, generate_unauthorized_response,
        },
    };

    use super::*;

    #[derive(Deserialize, Validate)]
    pub struct Params {
        #[validate(length(min = 1, message = "Worked hours are required."))]
        worked_hours: String,
        employment_id: i32,
        worked_hours_id: Option<i32>,
        date: Date,
    }

    pub async fn attendance_log(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
        params: Form<Params>,
    ) -> Result<Response, AppError> {
        let current_user = match auth_session.clone().user {
            Some(user) => user,
            None => return Err(AppError::from(ApiError::InternalServerError)),
        };

        let worked_hours = params.worked_hours.parse::<f32>();
        let worked_hours = match worked_hours {
            Ok(worked_hours) => worked_hours,
            Err(_error) => {
                return Ok(generate_toast_response(
                    ToastType::Error,
                    "Hours worked value must be a number".to_string(),
                ));
            }
        };

        // Check if user has employment.
        let user_employments = app_state
            .employment_repository
            .list_employment(employment::SelectManyFilter {
                position_id: None,
                user_id: Some(current_user.id),
                state: None,
                rating: None,
            })
            .await?;
        if !user_employments
            .into_iter()
            .any(|employment| employment.id == params.employment_id)
        {
            return Ok(generate_unauthorized_response());
        }

        match params.validate() {
            Ok(_) => (),
            Err(errors) => return Ok(generate_form_errors_response(errors)),
        }
        let updated_worked_hours = match params.worked_hours_id {
            Some(worked_hours_id) => {
                app_state
                    .worked_hours_repository
                    .update_worked_hours(
                        worked_hours_id,
                        PartialWorkedHours {
                            date: None,
                            hours_worked: Some(worked_hours),
                            employment_id: None,
                        },
                    )
                    .await?
            }
            None => {
                app_state
                    .worked_hours_repository
                    .create_worked_hours(CreateWorkedHours {
                        date: params.date,
                        hours_worked: worked_hours,
                        employment_id: params.employment_id,
                    })
                    .await?
            }
        };

        let template = HoursWorkedInputTemplate {
            session: auth_session,
            worked_hours: updated_worked_hours,
        };
        let html = template.render().unwrap();
        Ok(Html(html).into_response())
    }
}
