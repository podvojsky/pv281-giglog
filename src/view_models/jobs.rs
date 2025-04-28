use serde::{Deserialize, Serialize};
use sqlx::types::time::Date;

use crate::models::{
    employment::Employment,
    event::Event,
    job_position::SalaryCurrency,
    position_category::PositionCategory,
    user::{Gender, UserRole},
    venue::Venue,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastJobEventViewModel {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastJobsViewModel {
    pub job_name: String,
    pub event: PastJobEventViewModel,
    pub date_start: Date,
    pub date_end: Date,
    pub venue: Venue,
    pub hours_worked: i32,
    pub rating: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageJobPositionsViewModel {
    pub id: i32,
    pub name: String,
    pub salary: f32,
    pub current_capacity: usize,
    pub max_capacity: i32,
    pub is_opened_for_registration: bool,
    pub currency: SalaryCurrency,
    pub event: Event,
    pub category: PositionCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageJobPositionViewModel {
    pub id: i32,
    pub name: String,
    pub salary: f32,
    pub capacity: i32,
    pub is_opened_for_registration: bool,
    pub currency: SalaryCurrency,
    pub instructions: String,
    pub description: Option<String>,
    pub event: Event,
    pub category: PositionCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageJobEmployeeViewModel {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub gender: Gender,
    pub birth_date: Date,
    pub email: String,
    pub phone: String,
    pub password_hash: String,
    pub role: UserRole,
    pub tax_rate: f32,
    pub avatar_url: Option<String>,
    pub employment: Employment,
}
