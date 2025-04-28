use serde::{Deserialize, Serialize};

use crate::models::user::{Gender, UserRole};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserViewModel {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub gender: Gender,
    pub age: i32,
    pub email: String,
    pub phone: String,
    pub role: UserRole,
    pub tax_rate: f32,
    pub avatar_url: Option<String>,
}
