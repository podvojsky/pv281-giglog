use chrono::{DateTime, Datelike, Local, NaiveDate};
use sqlx::types::time::Date;
use tower_sessions::cookie::time::Month;

/// Parses a date string in format "YYYY-MM-DD" into a Date type.
///
/// # Examples
/// ```rust
/// let result = parse_date("2024-02-04");
/// assert!(result.is_ok());
///
/// let date = result.unwrap();
/// assert_eq!(date.year(), 2024);
/// assert_eq!(date.month(), Month::February);
/// assert_eq!(date.day(), 4);
/// ```
pub fn parse_date(date_str: &str) -> Result<Date, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return Err("Invalid date format".into());
    }

    let year: i32 = parts[0].parse()?;
    let month: u8 = parts[1].parse()?;
    let day: u8 = parts[2].parse()?;

    let month = Month::try_from(month)?;

    Date::from_calendar_date(year, month, day).map_err(|e| e.into())
}
/// Converts DateTime<Local> to Date type.
///
/// # Examples
/// ```rust
/// let dt = Local.with_ymd_and_hms(2024, 2, 4, 15, 30, 0).unwrap();
/// let date = convert_date_time_to_date(dt);
///
/// assert_eq!(date.year(), 2024);
/// assert_eq!(date.month(), Month::February);
/// assert_eq!(date.day(), 4);
/// ```
pub fn convert_date_time_to_date(dt: DateTime<Local>) -> Date {
    let naive_date: NaiveDate = dt.date_naive();
    Date::from_calendar_date(
        naive_date.year(),
        match naive_date.month() {
            1 => Month::January,
            2 => Month::February,
            3 => Month::March,
            4 => Month::April,
            5 => Month::May,
            6 => Month::June,
            7 => Month::July,
            8 => Month::August,
            9 => Month::September,
            10 => Month::October,
            11 => Month::November,
            12 => Month::December,
            _ => Month::January,
        },
        naive_date.day().try_into().unwrap(),
    )
    .unwrap()
}

/// Creates a vector of consecutive dates between start and end dates (inclusive).
///
/// # Examples
/// ```rust
/// let start = Date::from_calendar_date(2024, Month::February, 1).unwrap();
/// let end = Date::from_calendar_date(2024, Month::February, 3).unwrap();
///
/// let dates = from_date_range_to_vec(start, end);
/// assert_eq!(dates.len(), 3);
/// assert_eq!(dates[0].day(), 1);
/// assert_eq!(dates[2].day(), 3);
/// ```
pub fn from_date_range_to_vec(dt_start: Date, dt_end: Date) -> Vec<Date> {
    let mut acc: Vec<Date> = Vec::new();
    let mut dt = dt_start;
    while dt <= dt_end {
        acc.push(dt);
        dt = dt.next_day().unwrap();
    }
    acc
}

/// Checks if a given date is in the past compared to current date.
///
/// # Examples
/// let past_date = Date::from_calendar_date(2023, Month::January, 1).unwrap();
/// assert!(is_date_in_past(past_date));
///
/// let future_date = Date::from_calendar_date(2026, Month::December, 31).unwrap();
/// assert!(!is_date_in_past(future_date));
/// ```
pub fn is_date_in_past(dt: Date) -> bool {
    let current_date_time = chrono::Local::now();
    let current_date = convert_date_time_to_date(current_date_time);
    dt < current_date
}
