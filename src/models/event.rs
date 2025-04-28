use serde::{Deserialize, Serialize};
use sqlx::types::time::Date;
use validator::Validate;

#[derive(Clone, Serialize, Deserialize)]
pub struct SelectManyFilter {
    pub date_from: Option<Date>,
    pub date_to: Option<Date>,
    pub is_draft: Option<bool>,
    pub venue_id: Option<i32>,
    pub owner_id: Option<i32>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub date_start: Date,
    pub date_end: Date,
    pub img_url: String,
    pub description: Option<String>,
    pub is_draft: bool,
    pub venue_id: i32,
    pub owner_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateEvent {
    #[validate(length(min = 3, message = "Event name must be at least 3 characters long."))]
    pub name: String,
    pub date_start: Date,
    pub date_end: Date,
    #[validate(url(message = "Event image URL is not in the correct format."))]
    pub img_url: String,
    pub description: String,
    pub is_draft: bool,
    pub venue_id: i32,
    pub owner_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PartialEvent {
    #[validate(length(min = 3, message = "Event name must be at least 3 characters long."))]
    pub name: Option<String>,
    pub date_start: Option<Date>,
    pub date_end: Option<Date>,
    #[validate(url(message = "Event image URL is not in the correct format."))]
    pub img_url: Option<String>,
    pub description: Option<String>,
    pub is_draft: Option<bool>,
    pub venue_id: Option<i32>,
    pub owner_id: Option<i32>,
}
