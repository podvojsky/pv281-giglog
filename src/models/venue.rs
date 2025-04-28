use crate::regex::{RE_POSTAL_CODE, RE_STREET_NUMBER};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize)]
pub struct SelectManyFilter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub town: Option<String>,
    pub street_name: Option<String>,
    pub street_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Venue {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub state: String,
    pub postal_code: String,
    pub town: String,
    pub street_name: String,
    pub street_number: String,
    pub address_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateVenue {
    #[validate(length(min = 3, message = "Venue name must be at least 3 characters long."))]
    pub name: String,
    pub description: Option<String>,
    #[validate(length(min = 3, message = "State must be at least 3 characters long."))]
    pub state: String,
    #[validate(regex(path = *RE_POSTAL_CODE, message = "Postal code is not in the correct format."))]
    pub postal_code: String,
    #[validate(length(min = 2, message = "Town must be at least 2 characters long."))]
    pub town: String,
    #[validate(length(min = 1, message = "Street name must be at least 1 character long."))]
    pub street_name: String,
    #[validate(regex(path = *RE_STREET_NUMBER, message = "Street number is not in the correct format."))]
    pub street_number: String,
    #[validate(url(message = "Address URL is not in the correct format."))]
    pub address_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PartialVenue {
    #[validate(length(min = 3, message = "Venue name must be at least 3 characters long."))]
    pub name: Option<String>,
    pub description: Option<String>,
    #[validate(length(min = 3, message = "State must be at least 3 characters long."))]
    pub state: Option<String>,
    #[validate(regex(path = *RE_POSTAL_CODE, message = "Postal code is not in the correct format."))]
    pub postal_code: Option<String>,
    #[validate(length(min = 2, message = "Town must be at least 2 characters long."))]
    pub town: Option<String>,
    #[validate(length(min = 1, message = "Street name must be at least 1 character long."))]
    pub street_name: Option<String>,
    #[validate(regex(path = *RE_STREET_NUMBER, message = "Street number is not in the correct format."))]
    pub street_number: Option<String>,
    #[validate(url(message = "Address URL is not in the correct format."))]
    pub address_url: Option<String>,
}
