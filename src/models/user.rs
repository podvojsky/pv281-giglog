use crate::regex::RE_PHONE_NUMBER;
use serde::{Deserialize, Serialize};
use sqlx::types::time::Date;
use sqlx::Type;
use std::fmt;
use std::str::FromStr;
use validator::Validate;

#[derive(Serialize, Deserialize)]
pub struct SelectManyFilter {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub gender: Option<Gender>,
    pub role: Option<UserRole>,
    pub tax_rate: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateUser {
    #[validate(length(min = 3, message = "First Name has to be at least 3 characters long."))]
    pub first_name: String,
    #[validate(length(min = 3, message = "Last Name has to be at least 3 characters long."))]
    pub last_name: String,
    #[validate(length(min = 3, message = "Username has to be at least 3 characters long."))]
    pub username: String,
    pub gender: Gender,
    pub birth_date: Date,
    #[validate(email(message = "Email is not in the correct format."))]
    pub email: String,
    #[validate(regex(path = *RE_PHONE_NUMBER, message = "Phone is not in the correct format."))]
    pub phone: String,
    pub password_hash: String,
    pub role: UserRole,
    #[validate(range(
        min = 0.0,
        max = 100.0,
        message = "Tax rate is out of bounds <0, 100>."
    ))]
    pub tax_rate: f32,
    #[validate(url(message = "Avatar URL is not in the correct format."))]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PartialUser {
    #[validate(length(min = 3, message = "First Name has to be at least 3 characters long."))]
    pub first_name: Option<String>,
    #[validate(length(min = 3, message = "Last Name has to be at least 3 characters long."))]
    pub last_name: Option<String>,
    #[validate(length(min = 3, message = "Username has to be at least 3 characters long."))]
    pub username: Option<String>,
    pub gender: Option<Gender>,
    pub birth_date: Option<Date>,
    #[validate(email(message = "Email is not in the correct format."))]
    pub email: Option<String>,
    #[validate(regex(path = *RE_PHONE_NUMBER, message = "Phone is not in the correct format. The correct format is 666777888 or 666 777 888."))]
    pub phone: Option<String>,
    pub password_hash: Option<String>,
    pub role: Option<UserRole>,
    #[validate(range(
        min = 0.0,
        max = 100.0,
        message = "Tax rate is out of bounds <0, 100>."
    ))]
    pub tax_rate: Option<f32>,
    #[validate(url(message = "Avatar URL is not in the correct format."))]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "gender_type", rename_all = "lowercase")]
pub enum Gender {
    Male,
    Female,
    Other,
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Gender::Male => write!(f, "Male"),
            Gender::Female => write!(f, "Female"),
            Gender::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Employee,
    Organizer,
    Admin,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserRole::Employee => write!(f, "Employee"),
            UserRole::Organizer => write!(f, "Organizer"),
            UserRole::Admin => write!(f, "Admin"),
        }
    }
}

impl FromStr for UserRole {
    type Err = ();
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "Employee" => Ok(UserRole::Employee),
            "Organizer" => Ok(UserRole::Organizer),
            "Admin" => Ok(UserRole::Admin),
            _ => Err(()),
        }
    }
}
