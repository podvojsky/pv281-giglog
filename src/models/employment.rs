use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::fmt::Display;
use std::str::FromStr;
use validator::Validate;

#[derive(Serialize, Deserialize)]
pub struct SelectManyFilter {
    pub position_id: Option<i32>,
    pub user_id: Option<i32>,
    pub state: Option<EmploymentState>,
    pub rating: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employment {
    pub id: i32,
    pub rating: i32,
    pub state: EmploymentState,
    pub user_id: i32,
    pub position_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateEmployment {
    #[validate(range(min = 0, max = 5, message = "Rating is out of bounds <0, 5>"))]
    pub rating: i32,
    pub state: EmploymentState,
    pub user_id: i32,
    pub position_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PartialEmployment {
    #[validate(range(min = 0, max = 5, message = "Rating is out of bounds <0, 5>"))]
    pub rating: Option<i32>,
    pub state: Option<EmploymentState>,
    pub user_id: Option<i32>,
    pub position_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, PartialOrd, Ord)]
#[sqlx(type_name = "employment_state", rename_all = "lowercase")]
pub enum EmploymentState {
    Accepted,
    Done,
    Pending,
    Rejected,
}

pub fn all_employment_states() -> Vec<EmploymentState> {
    vec![
        EmploymentState::Accepted,
        EmploymentState::Done,
        EmploymentState::Pending,
        EmploymentState::Rejected,
    ]
}

impl Display for EmploymentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            EmploymentState::Pending => "Pending".to_string(),
            EmploymentState::Accepted => "Accepted".to_string(),
            EmploymentState::Rejected => "Rejected".to_string(),
            EmploymentState::Done => "Done".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl FromStr for EmploymentState {
    type Err = ();
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "Pending" => Ok(EmploymentState::Pending),
            "Accepted" => Ok(EmploymentState::Accepted),
            "Rejected" => Ok(EmploymentState::Rejected),
            "Done" => Ok(EmploymentState::Done),
            _ => Err(()),
        }
    }
}
