use crate::models::employment::EmploymentState;
use serde::{Deserialize, Serialize};
use sqlx::types::time::Date;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyJobsViewModel {
    pub state: EmploymentState,
    pub job_name: String,
    pub job_id: i32,
    pub event_name: String,
    pub event_id: i32,
    pub date_from: Date,
    pub date_to: Date,
    pub venue_name: String,
    pub venue_address_url: Option<String>,
    pub hours_worked: String,
    pub salary_tax_free: String,
    pub salary_taxed: String,
    pub job_instructions: String,
    pub rating: i32,
}

pub struct JobSummary {
    pub total_jobs: usize,
    pub unique_events: usize,
    pub unique_venues: usize,
    pub total_hours_worked: String,
    pub total_salary_tax_free: String,
    pub total_salary_taxed: String,
    pub earliest_date: Option<Date>,
    pub latest_date: Option<Date>,
    pub average_rating: String,
}

impl JobSummary {
    pub fn formatted_earliest_date(&self) -> String {
        self.earliest_date
            .map(|d| d.to_string())
            .unwrap_or_else(|| "N/A".to_string())
    }

    pub fn formatted_latest_date(&self) -> String {
        self.latest_date
            .map(|d| d.to_string())
            .unwrap_or_else(|| "N/A".to_string())
    }
}
