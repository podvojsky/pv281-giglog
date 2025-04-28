use serde::{Deserialize, Serialize};
use crate::models::employment::EmploymentState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentViewModel {
    pub state: EmploymentState,
    pub job_name: String,
    pub event_name: String,
    pub employee_name: String,
    pub employee_id: i32,
    pub event_id: i32,
    pub employment_id: i32,
    pub max_capacity: i32,
    pub current_capacity: i32,
    pub rating: i32,
}