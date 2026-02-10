//! Date conversion utilities for ritmo
//!
//! This module provides helper functions for common date conversions,
//! eliminating code duplication across services.

use chrono::NaiveDate;

/// Convert a year (i32) to Unix timestamp (first day of year at midnight UTC)
///
/// # Arguments
/// * `year` - The year to convert (e.g., 2024)
///
/// # Returns
/// * Unix timestamp (i64) representing January 1st of that year at 00:00:00 UTC
///
/// # Example
/// ```
/// use ritmo_core::utils::year_to_timestamp;
///
/// let timestamp = year_to_timestamp(2024);
/// // Returns timestamp for 2024-01-01 00:00:00 UTC
/// ```
pub fn year_to_timestamp(year: i32) -> i64 {
    NaiveDate::from_ymd_opt(year, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp()
}

/// Convert an optional year to an optional Unix timestamp
///
/// This is a convenience wrapper around `year_to_timestamp` for working
/// with optional values.
///
/// # Arguments
/// * `year` - Optional year to convert
///
/// # Returns
/// * `Some(timestamp)` if year is Some
/// * `None` if year is None
///
/// # Example
/// ```
/// use ritmo_core::utils::opt_year_to_timestamp;
///
/// let timestamp = opt_year_to_timestamp(Some(2024));
/// assert!(timestamp.is_some());
///
/// let no_timestamp = opt_year_to_timestamp(None);
/// assert!(no_timestamp.is_none());
/// ```
pub fn opt_year_to_timestamp(year: Option<i32>) -> Option<i64> {
    year.map(year_to_timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn test_year_to_timestamp() {
        let timestamp = year_to_timestamp(2024);

        // Verify it's January 1st, 2024
        let datetime = chrono::DateTime::from_timestamp(timestamp, 0).unwrap();
        assert_eq!(datetime.year(), 2024);
        assert_eq!(datetime.month(), 1);
        assert_eq!(datetime.day(), 1);
        assert_eq!(datetime.hour(), 0);
        assert_eq!(datetime.minute(), 0);
        assert_eq!(datetime.second(), 0);
    }

    #[test]
    fn test_opt_year_to_timestamp_with_some() {
        let result = opt_year_to_timestamp(Some(2024));
        assert!(result.is_some());

        let timestamp = result.unwrap();
        let datetime = chrono::DateTime::from_timestamp(timestamp, 0).unwrap();
        assert_eq!(datetime.year(), 2024);
    }

    #[test]
    fn test_opt_year_to_timestamp_with_none() {
        let result = opt_year_to_timestamp(None);
        assert!(result.is_none());
    }

    #[test]
    fn test_different_years() {
        let ts_2020 = year_to_timestamp(2020);
        let ts_2021 = year_to_timestamp(2021);
        let ts_2022 = year_to_timestamp(2022);

        // Later years should have larger timestamps
        assert!(ts_2021 > ts_2020);
        assert!(ts_2022 > ts_2021);

        // Years should differ by roughly 365-366 days (accounting for leap years)
        let min_year = 365 * 24 * 60 * 60;       // 365 days in seconds
        let max_year = 366 * 24 * 60 * 60;       // 366 days in seconds
        let diff_2021_2020 = ts_2021 - ts_2020;
        let diff_2022_2021 = ts_2022 - ts_2021;

        assert!(diff_2021_2020 >= min_year && diff_2021_2020 <= max_year);
        assert!(diff_2022_2021 >= min_year && diff_2022_2021 <= max_year);
    }
}
