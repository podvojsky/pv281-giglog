use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventManagerRelation {
    pub user_id: i32,
    pub event_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventManagerRelation {
    pub user_id: i32,
    pub event_id: i32,
}
