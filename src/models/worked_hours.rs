use serde::{Deserialize, Serialize};
use sqlx::types::time::Date;
use validator::Validate;

#[derive(Serialize, Deserialize)]
pub struct SelectManyFilter {
    pub hours_worked: Option<f32>,
    pub date: Option<Date>,
    pub employment_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkedHours {
    pub id: i32,
    pub date: Date,
    pub hours_worked: f32,
    pub employment_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateWorkedHours {
    pub date: Date,
    #[validate(range(
        min = 1.0,
        max = 24.0,
        message = "Worked hours are out of bounds <1, 24>"
    ))]
    pub hours_worked: f32,
    pub employment_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PartialWorkedHours {
    pub date: Option<Date>,
    #[validate(range(
        min = 1.0,
        max = 24.0,
        message = "Worked hours are out of bounds <1, 24>"
    ))]
    pub hours_worked: Option<f32>,
    pub employment_id: Option<i32>,
}
