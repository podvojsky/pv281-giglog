use crate::app::AppState;
use crate::error::{ApiError, AppError};
use crate::handlers::app::auth::AuthSession;
use crate::models::employment::{all_employment_states, EmploymentState, SelectManyFilter};
use crate::repositories::employment::EmploymentRepository;
use crate::repositories::event::EventRepository;
use crate::repositories::job_position::JobPositionRepository;
use crate::repositories::venue::VenueRepository;
use crate::repositories::worked_hours::WorkedHoursRepository;
use crate::templates::{JobsTableTemplate, JobsTemplate};
use crate::view_models::my_jobs::{JobSummary, MyJobsViewModel};
use askama_axum::Template;
use axum::extract::State;
use axum::response::Html;
use axum::Form;
use serde::Deserialize;
use std::collections::HashSet;
use std::str::FromStr;

pub mod create;
pub mod job;
pub mod manage;

async fn generate_jobs_viewmodels(
    auth_user_id: i32,
    tax_rate: f32,
    app_state: &AppState,
    employment_state: Option<EmploymentState>,
) -> Result<Vec<MyJobsViewModel>, AppError> {
    let employment_filter = SelectManyFilter {
        position_id: None,
        user_id: Some(auth_user_id),
        state: employment_state,
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

    let mut my_jobs_viewmodels = Vec::new();

    for employment in employments {
        let position = app_state
            .job_position_repository
            .get_job_position_by_id(employment.position_id)
            .await
            .map_err(|err| {
                eprintln!("Failed to retrieve job position: {:?}", err);
                ApiError::NotFound
            })?;

        let event = app_state
            .event_repository
            .get_event_by_id(position.event_id)
            .await
            .map_err(|err| {
                eprintln!("Failed to retrieve event: {:?}", err);
                ApiError::NotFound
            })?;

        let venue = app_state
            .venue_repository
            .get_venue_by_id(event.venue_id)
            .await
            .map_err(|err| {
                eprintln!("Failed to retrieve venue: {:?}", err);
                ApiError::NotFound
            })?;

        let worked_hours_filter = crate::models::worked_hours::SelectManyFilter {
            hours_worked: None,
            date: None,
            employment_id: Some(employment.id),
        };

        let worked_hours_list = app_state
            .worked_hours_repository
            .list_worked_hours(worked_hours_filter)
            .await
            .map_err(|err| {
                eprintln!("Failed to retrieve worked hours: {:?}", err);
                ApiError::NotFound
            })?;

        let total_hours: f32 = worked_hours_list
            .iter()
            .map(|worked_hours| worked_hours.hours_worked)
            .sum();
        let salary_tax_free = total_hours * position.salary;
        let salary_taxed = salary_tax_free * (1.0 - tax_rate);

        my_jobs_viewmodels.push(MyJobsViewModel {
            state: employment.state,
            job_name: position.name,
            job_id: position.id,
            event_name: event.name,
            event_id: event.id,
            date_from: event.date_start,
            date_to: event.date_end,
            venue_name: venue.name,
            venue_address_url: venue.address_url,
            hours_worked: format!("{:.2}", total_hours.abs()),
            salary_tax_free: format!("{:.2}", salary_tax_free.abs()),
            salary_taxed: format!("{:.2}", salary_taxed.abs()),
            job_instructions: position.instructions_html,
            rating: employment.rating,
        });
    }

    Ok(my_jobs_viewmodels)
}

pub mod get {
    use super::*;

    pub async fn jobs(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
    ) -> Result<Html<String>, AppError> {
        let auth_user = auth_session
            .user
            .clone()
            .expect("User should be logged in.");

        let my_jobs_viewmodels =
            generate_jobs_viewmodels(auth_user.id, auth_user.tax_rate, &app_state, None).await?;

        let summary = generate_job_summary(&my_jobs_viewmodels);

        let template = JobsTemplate {
            session: auth_session,
            active_route: Some(crate::templates::ActiveRoute::MyJobs),
            my_jobs: my_jobs_viewmodels,
            employment_states: all_employment_states(),
            job_summary: summary,
        };

        let html = template.render().map_err(|_| ApiError::NotFound)?;
        Ok(Html(html))
    }
}
fn generate_job_summary(jobs: &[MyJobsViewModel]) -> JobSummary {
    JobSummary {
        total_jobs: jobs.len(),
        unique_events: jobs
            .iter()
            .map(|j| &j.event_name)
            .collect::<HashSet<_>>()
            .len(),
        unique_venues: jobs
            .iter()
            .map(|j| &j.venue_name)
            .collect::<HashSet<_>>()
            .len(),
        total_hours_worked: format!(
            "{:.2}",
            jobs.iter()
                .map(|j| j.hours_worked.parse::<f32>().unwrap_or(0.0))
                .sum::<f32>()
                .abs()
        ),
        total_salary_tax_free: format!(
            "{:.2}",
            jobs.iter()
                .map(|j| j.salary_tax_free.parse::<f32>().unwrap_or(0.0))
                .sum::<f32>()
                .abs()
        ),
        total_salary_taxed: format!(
            "{:.2}",
            jobs.iter()
                .map(|j| j.salary_taxed.parse::<f32>().unwrap_or(0.0))
                .sum::<f32>()
                .abs()
        ),
        earliest_date: jobs.iter().map(|j| j.date_from).min(),
        latest_date: jobs.iter().map(|j| j.date_to).max(),
        average_rating: {
            let valid_ratings: Vec<_> = jobs.iter()
                .filter(|j| j.rating != 0)
                .map(|j| j.rating)
                .collect();

            if valid_ratings.is_empty() {
                "0.00".to_string()
            } else {
                format!("{:.2}",
                        valid_ratings.iter().sum::<i32>() as f32 / valid_ratings.len() as f32
                )
            }
        },
    }
}

pub mod post {
    use super::*;
    use crate::utils::table_utils::{
        optional_filter, parse_filter, parse_optional_date, SortDirection,
    };
    use serde::Serialize;

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum SortColumn {
        State,
        JobName,
        Event,
        DateStart,
        Venue,
        HoursWorked,
        SalaryTaxFree,
        SalaryTaxed,
    }

    #[allow(warnings)]
    impl Default for SortColumn {
        fn default() -> Self {
            SortColumn::DateStart
        }
    }

    #[derive(Deserialize)]
    pub struct FilterSortData {
        state: String,
        date_start: String,
        date_end: String,
        event: String,
        venue: String,
        job_name: String,
        sort_by: Option<SortColumn>,
        sort_direction: Option<SortDirection>,
    }

    pub async fn jobs(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
        Form(payload): Form<FilterSortData>,
    ) -> Result<Html<String>, AppError> {
        let auth_user = auth_session
            .user
            .clone()
            .expect("User should be logged in.");

        let employment_state = parse_filter(
            payload.state.as_str(),
            |state| EmploymentState::from_str(state).map_err(|_| ApiError::NotFound),
            "All states",
        )?;

        let date_start = parse_optional_date(&payload.date_start)?;
        let date_end = parse_optional_date(&payload.date_end)?;

        let venue_filter = optional_filter(payload.venue);
        let event_filter = optional_filter(payload.event);
        let job_name_filter = optional_filter(payload.job_name);
        let sort_by = payload.sort_by.unwrap_or_default();
        let sort_direction = payload.sort_direction.unwrap_or_default();

        let my_jobs_viewmodels = generate_jobs_viewmodels(
            auth_user.id,
            auth_user.tax_rate,
            &app_state,
            employment_state,
        )
        .await?;

        let filtered_jobs = my_jobs_viewmodels
            .into_iter()
            .filter(|job| match date_start {
                Some(start) => job.date_from >= start,
                None => true,
            })
            .filter(|job| match date_end {
                Some(end) => job.date_to <= end,
                None => true,
            })
            .filter(|job| match &venue_filter {
                Some(venue) => job
                    .venue_name
                    .to_lowercase()
                    .contains(&venue.to_lowercase()),
                None => true,
            })
            .filter(|job| match &event_filter {
                Some(event) => job
                    .event_name
                    .to_lowercase()
                    .contains(&event.to_lowercase()),
                None => true,
            })
            .filter(|job| match &job_name_filter {
                Some(job_name) => job
                    .job_name
                    .to_lowercase()
                    .contains(&job_name.to_lowercase()),
                None => true,
            })
            .collect::<Vec<_>>();

        let summary = generate_job_summary(&filtered_jobs);

        let mut sorted_jobs = filtered_jobs;
        sorted_jobs.sort_by(|a, b| {
            let cmp = match sort_by {
                SortColumn::State => a.state.cmp(&b.state),
                SortColumn::JobName => a.job_name.cmp(&b.job_name),
                SortColumn::Event => a.event_name.cmp(&b.event_name),
                SortColumn::DateStart => a.date_from.cmp(&b.date_from),
                SortColumn::Venue => a.venue_name.cmp(&b.venue_name),
                SortColumn::HoursWorked => a
                    .hours_worked
                    .parse::<f32>()
                    .unwrap_or_default()
                    .partial_cmp(&b.hours_worked.parse::<f32>().unwrap_or_default())
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::SalaryTaxFree => a
                    .salary_tax_free
                    .parse::<f32>()
                    .unwrap_or_default()
                    .partial_cmp(&b.salary_tax_free.parse::<f32>().unwrap_or_default())
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::SalaryTaxed => a
                    .salary_taxed
                    .parse::<f32>()
                    .unwrap_or_default()
                    .partial_cmp(&b.salary_taxed.parse::<f32>().unwrap_or_default())
                    .unwrap_or(std::cmp::Ordering::Equal),
            };

            match sort_direction {
                SortDirection::Asc => cmp,
                SortDirection::Desc => cmp.reverse(),
            }
        });

        let template = JobsTableTemplate {
            my_jobs: sorted_jobs,
            job_summary: summary,
        };

        let html = template.render().map_err(|_| ApiError::NotFound)?;
        Ok(Html(html))
    }
}
