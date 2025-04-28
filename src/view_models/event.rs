use serde::{Deserialize, Serialize};
use sqlx::types::time::Date;

use crate::models::{event::Event, job_position::JobPositionViewModel, user::User, venue::Venue};

pub trait IEventViewModel {
    fn new(event: Event, venue: Venue, owner: User) -> Self;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventViewModel {
    pub id: i32,
    pub name: String,
    pub date_start: Date,
    pub date_end: Date,
    pub img_url: String,
    pub is_draft: bool,
    pub venue: Venue,
    pub owner: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageEventViewModel {
    pub id: i32,
    pub name: String,
    pub date_start: Date,
    pub date_end: Date,
    pub img_url: String,
    pub is_draft: bool,
    pub venue: Venue,
    pub owner: User,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDetailViewModel {
    pub id: i32,
    pub name: String,
    pub date_start: Date,
    pub date_end: Date,
    pub img_url: String,
    pub description: Option<String>,
    pub is_draft: bool,
    pub venue: Venue,
    pub owner: User,
    pub job_positions: Vec<JobPositionViewModel>,
}

impl IEventViewModel for EventViewModel {
    fn new(event: Event, venue: Venue, owner: User) -> Self {
        Self {
            id: event.id,
            name: event.name,
            date_start: event.date_start,
            date_end: event.date_end,
            img_url: event.img_url,
            is_draft: event.is_draft,
            venue,
            owner,
        }
    }
}
