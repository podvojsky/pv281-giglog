use super::{employment::EmploymentState, position_category::PositionCategory};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use validator::Validate;

#[derive(Serialize, Deserialize)]
pub struct SelectManyFilter {
    pub event_id: Option<i32>,
    pub position_category_id: Option<i32>,
    pub salary: Option<f32>,
    pub currency: Option<SalaryCurrency>,
    pub capacity: Option<i32>,
    pub is_opened_for_registration: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPosition {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub salary: f32,
    pub capacity: i32,
    pub instructions_html: String,
    pub is_opened_for_registration: bool,
    pub currency: SalaryCurrency,
    pub event_id: i32,
    pub position_category_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateJobPosition {
    #[validate(length(min = 3, message = "Job name must be at least 3 characters long."))]
    pub name: String,
    pub description: String,
    #[validate(range(min = 0.0, message = "Salary cannot be negative."))]
    pub salary: f32,
    pub currency: SalaryCurrency,
    #[validate(range(min = 1, message = "Job must have at least one space."))]
    pub capacity: i32,
    pub instructions_html: String,
    pub is_opened_for_registration: bool,
    pub event_id: i32,
    pub position_category_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PartialJobPosition {
    #[validate(length(min = 3, message = "Job name must be at least 3 characters long."))]
    pub name: Option<String>,
    pub description: Option<String>,
    #[validate(range(min = 0.0, message = "Salary cannot be negative."))]
    pub salary: Option<f32>,
    pub currency: Option<SalaryCurrency>,
    #[validate(range(min = 1, message = "Job must have at least one space."))]
    pub capacity: Option<i32>,
    pub instructions_html: Option<String>,
    pub is_opened_for_registration: Option<bool>,
    pub event_id: Option<i32>,
    pub position_category_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPositionViewModel {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub instructions_html: String,
    pub salary: f32,
    pub current_capacity: i32, // Current users that were accepted in the position.
    pub max_capacity: i32,
    pub is_opened_for_registration: bool,
    pub employment_state: Option<EmploymentState>, // In which state the job is according to the current user.
    pub position_category: Option<PositionCategory>,
}

#[allow(warnings)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "salary_currency", rename_all = "UPPERCASE")]
pub enum SalaryCurrency {
    CZK,
    EUR,
}
