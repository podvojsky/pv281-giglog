use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageVenuesViewModel {
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
