use once_cell::sync::Lazy;
use regex::Regex;

pub static RE_PHONE_NUMBER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b\d{9}\b|\b\d{3} \d{3} \d{3}\b").unwrap());
pub static RE_DATE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:\d{4})-(?:0[1-9]|1[0-2])-(?:0[1-9]|[12]\d|3[01])$").unwrap());
pub static RE_POSTAL_CODE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b\d{5}\b|\b\d{3} \d{2}").unwrap());
pub static RE_STREET_NUMBER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\d{1,4}(/\d{1,4}[a-zA-Z]?)?$").unwrap());
