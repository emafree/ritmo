//! Input validation for filters
//!
//! This module provides validation functions for filter inputs to ensure
//! data integrity and prevent potential issues.

use super::types::{BookFilters, ContentFilters};

/// Validation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Offset is negative
    NegativeOffset,
    /// Limit is zero or negative
    InvalidLimit,
    /// Too many values for a single filter (potential performance issue)
    TooManyValues {
        field: String,
        count: usize,
        max: usize,
    },
    /// Invalid date range (after > before)
    InvalidDateRange { after: i64, before: i64 },
    /// Empty filter value
    EmptyValue { field: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::NegativeOffset => write!(f, "Offset cannot be negative"),
            ValidationError::InvalidLimit => write!(f, "Limit must be greater than 0"),
            ValidationError::TooManyValues { field, count, max } => {
                write!(
                    f,
                    "Too many values for '{}': {} (max: {})",
                    field, count, max
                )
            }
            ValidationError::InvalidDateRange { after, before } => {
                write!(
                    f,
                    "Invalid date range: acquired_after ({}) > acquired_before ({})",
                    after, before
                )
            }
            ValidationError::EmptyValue { field } => {
                write!(f, "Empty value provided for filter '{}'", field)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validates BookFilters
pub fn validate_book_filters(filters: &BookFilters) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Validate offset
    if filters.offset < 0 {
        errors.push(ValidationError::NegativeOffset);
    }

    // Validate limit
    if let Some(limit) = filters.limit {
        if limit <= 0 {
            errors.push(ValidationError::InvalidLimit);
        }
    }

    // Validate number of values (prevent performance issues)
    const MAX_VALUES: usize = 50;

    if filters.authors.len() > MAX_VALUES {
        errors.push(ValidationError::TooManyValues {
            field: "authors".to_string(),
            count: filters.authors.len(),
            max: MAX_VALUES,
        });
    }

    if filters.publishers.len() > MAX_VALUES {
        errors.push(ValidationError::TooManyValues {
            field: "publishers".to_string(),
            count: filters.publishers.len(),
            max: MAX_VALUES,
        });
    }

    if filters.formats.len() > MAX_VALUES {
        errors.push(ValidationError::TooManyValues {
            field: "formats".to_string(),
            count: filters.formats.len(),
            max: MAX_VALUES,
        });
    }

    // Validate date range
    if let (Some(after), Some(before)) = (filters.acquired_after, filters.acquired_before) {
        if after > before {
            errors.push(ValidationError::InvalidDateRange { after, before });
        }
    }

    // Validate non-empty values
    for author in &filters.authors {
        if author.trim().is_empty() {
            errors.push(ValidationError::EmptyValue {
                field: "author".to_string(),
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validates ContentFilters
pub fn validate_content_filters(filters: &ContentFilters) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Validate offset
    if filters.offset < 0 {
        errors.push(ValidationError::NegativeOffset);
    }

    // Validate limit
    if let Some(limit) = filters.limit {
        if limit <= 0 {
            errors.push(ValidationError::InvalidLimit);
        }
    }

    // Validate number of values
    const MAX_VALUES: usize = 50;

    if filters.authors.len() > MAX_VALUES {
        errors.push(ValidationError::TooManyValues {
            field: "authors".to_string(),
            count: filters.authors.len(),
            max: MAX_VALUES,
        });
    }

    if filters.content_types.len() > MAX_VALUES {
        errors.push(ValidationError::TooManyValues {
            field: "content_types".to_string(),
            count: filters.content_types.len(),
            max: MAX_VALUES,
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_book_filters_valid() {
        let filters = BookFilters::default().with_author("Test");
        assert!(validate_book_filters(&filters).is_ok());
    }

    #[test]
    fn test_validate_book_filters_negative_offset() {
        let filters = BookFilters {
            offset: -1,
            ..Default::default()
        };
        let result = validate_book_filters(&filters);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err()[0], ValidationError::NegativeOffset);
    }

    #[test]
    fn test_validate_book_filters_invalid_limit() {
        let filters = BookFilters {
            limit: Some(0),
            ..Default::default()
        };
        let result = validate_book_filters(&filters);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err()[0], ValidationError::InvalidLimit);
    }

    #[test]
    fn test_validate_book_filters_too_many_authors() {
        let mut filters = BookFilters::default();
        filters.authors = (0..51).map(|i| format!("Author{}", i)).collect();

        let result = validate_book_filters(&filters);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::TooManyValues { .. })));
    }

    #[test]
    fn test_validate_book_filters_invalid_date_range() {
        let filters = BookFilters {
            acquired_after: Some(100),
            acquired_before: Some(50),
            ..Default::default()
        };
        let result = validate_book_filters(&filters);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidDateRange { .. })));
    }

    #[test]
    fn test_validate_content_filters_valid() {
        let filters = ContentFilters::default().with_author("Test");
        assert!(validate_content_filters(&filters).is_ok());
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError::NegativeOffset;
        assert_eq!(error.to_string(), "Offset cannot be negative");

        let error = ValidationError::TooManyValues {
            field: "authors".to_string(),
            count: 100,
            max: 50,
        };
        assert!(error.to_string().contains("Too many values"));
    }
}
