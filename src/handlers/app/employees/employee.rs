use crate::handlers::app::auth::AuthSession;
use askama::Template;
use axum::{extract::State, response::Html};

use crate::{
    error::AppError,
    repositories::user::UserRepository,
};

pub mod get {
    use super::*;
    use crate::{
        app::AppState,
        models::{
            employment::{EmploymentState, SelectManyFilter},
            user::UserRole,
            worked_hours,
        },
        repositories::{
            employment::EmploymentRepository, event::EventRepository,
            job_position::JobPositionRepository, venue::VenueRepository,
            worked_hours::WorkedHoursRepository,
        },
        templates::EmployeeTemplate,
        utils::{
            date_utils::convert_date_time_to_date, response_utils::generate_unauthorized_response,
        },
        view_models::{
            jobs::{PastJobEventViewModel, PastJobsViewModel},
            user::UserViewModel,
        },
    };
    use askama_axum::IntoResponse;
    use axum::extract::Path;
    use axum::response::Response;
    
    
    
    

    pub async fn employee(
        Path(employee_id): Path<i32>,
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Response, AppError> {
        let current_date_time = chrono::Local::now();
        let current_date = convert_date_time_to_date(current_date_time);
        let user = app_state
            .user_repository
            .get_user_by_id(employee_id)
            .await?;
        match user.role {
            UserRole::Employee => (),
            _ => return Ok(generate_unauthorized_response()),
        }
        let age = (current_date - user.birth_date).as_seconds_f32() / (60.0 * 60.0 * 24.0 * 365.25);
        let age = age.floor() as i32;

        let mut past_jobs: Vec<PastJobsViewModel> = Vec::new();
        let employments = app_state
            .employment_repository
            .list_employment(SelectManyFilter {
                position_id: None,
                user_id: Some(employee_id),
                state: Some(EmploymentState::Done),
                rating: None,
            })
            .await?;

        for employment in employments {
            let job = app_state
                .job_position_repository
                .get_job_position_by_id(employment.position_id)
                .await?;
            let event = app_state
                .event_repository
                .get_event_by_id(job.event_id)
                .await?;
            let venue = app_state
                .venue_repository
                .get_venue_by_id(event.venue_id)
                .await?;
            let worked_hours = app_state
                .worked_hours_repository
                .list_worked_hours(worked_hours::SelectManyFilter {
                    hours_worked: None,
                    date: None,
                    employment_id: Some(employment.id),
                })
                .await?;
            let mut total_worked_hours = 0.0;

            for hours in worked_hours {
                total_worked_hours += hours.hours_worked;
            }

            past_jobs.push(PastJobsViewModel {
                job_name: job.name,
                event: PastJobEventViewModel {
                    id: event.id,
                    name: event.name,
                },
                date_start: event.date_start,
                date_end: event.date_end,
                venue,
                hours_worked: total_worked_hours as i32,
                rating: employment.rating,
            });
        }

        let template = EmployeeTemplate {
            session: auth_session,
            active_route: Some(crate::templates::ActiveRoute::Employees),
            employee: UserViewModel {
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
            },
            past_jobs,
        };
        let html = template.render().unwrap();
        Ok(Html(html).into_response())
    }
}
