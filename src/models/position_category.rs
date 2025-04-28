use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionCategory {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePositionCategory {
    #[validate(length(
        min = 3,
        message = "Position category name must be at least 3 characters long."
    ))]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PartialPositionCategory {
    #[validate(length(
        min = 3,
        message = "Position category name must be at least 3 characters long."
    ))]
    pub name: Option<String>,
}
