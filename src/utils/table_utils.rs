use crate::error::ApiError;
use crate::utils::date_utils::parse_date;
use serde::{Deserialize, Serialize};
use sqlx::types::time::Date;

pub fn optional_filter(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

pub fn parse_optional_date(value: &str) -> Result<Option<Date>, ApiError> {
    if value.is_empty() {
        Ok(None)
    } else {
        parse_date(value).map(Some).map_err(|_| ApiError::NotFound)
    }
}

pub fn parse_filter<T, F>(
    value: &str,
    parse_fn: F,
    default_value: &str,
) -> Result<Option<T>, ApiError>
where
    F: FnOnce(&str) -> Result<T, ApiError>,
{
    if value == default_value {
        Ok(None)
    } else {
        parse_fn(value).map(Some)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Asc,
    Desc,
}

#[allow(warnings)]
impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Desc
    }
}
