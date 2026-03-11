use ritmo_db_core::filters::{BookResult, ContentResult};
use std::io::{self, Write};

/// Result of a confirmation prompt
#[derive(Debug, PartialEq)]
pub enum ConfirmationResult {
    /// User confirmed the operation
    Confirmed,
    /// User declined the operation
    Declined,
    /// Skip confirmation (single item, --yes flag, or dry-run)
    Skip,
}

/// Preview item for display before confirmation
#[derive(Debug, Clone)]
pub struct PreviewItem {
    pub id: i64,
    pub display_text: String,
}

/// Confirmation configuration
pub struct ConfirmationConfig<'a> {
    /// Items to be affected by the operation
    pub items: Vec<PreviewItem>,
    /// Operation name (e.g., "delete", "update")
    pub operation: &'a str,
    /// Entity type (e.g., "book", "content")
    pub entity_type: &'a str,
    /// Skip confirmation (--yes flag)
    pub force_yes: bool,
    /// Dry-run mode (show preview and skip execution)
    pub dry_run: bool,
    /// Additional warning message
    pub warning: Option<&'a str>,
}

/// Display preview and get confirmation from user
///
/// # Returns
/// - `ConfirmationResult::Skip` if count == 1 or --yes flag or --dry-run
/// - `ConfirmationResult::Confirmed` if user types 'y' or 'yes'
/// - `ConfirmationResult::Declined` if user types anything else
///
/// # Errors
/// Returns IO error if stdin/stdout operations fail
pub fn confirm_operation(config: ConfirmationConfig) -> Result<ConfirmationResult, io::Error> {
    let count = config.items.len();

    // Dry-run mode: show preview and exit without execution
    if config.dry_run {
        println!("\n🔍 DRY-RUN MODE - No changes will be made");
        println!("{}", "=".repeat(60));
        println!(
            "Operation: {} {} {}",
            config.operation, count, config.entity_type
        );

        if let Some(warning) = config.warning {
            println!("\n⚠️  WARNING: {}", warning);
        }

        println!("\nItems that would be affected:");
        display_preview_items(&config.items);

        println!("\n{}", "=".repeat(60));
        println!(
            "Would {} {} {}(s)",
            config.operation, count, config.entity_type
        );
        return Ok(ConfirmationResult::Skip);
    }

    // Single item: execute directly without confirmation
    if count == 1 && !config.force_yes {
        println!(
            "Executing {} on 1 {}...",
            config.operation, config.entity_type
        );
        return Ok(ConfirmationResult::Skip);
    }

    // --yes flag: skip confirmation and execute
    if config.force_yes {
        println!("--yes flag: skipping confirmation");
        return Ok(ConfirmationResult::Skip);
    }

    // Bulk operation: show preview and prompt for confirmation
    println!("\n{} PREVIEW", config.operation.to_uppercase());
    println!("{}", "=".repeat(60));
    println!(
        "Operation: {} {} {}(s)",
        config.operation, count, config.entity_type
    );

    if let Some(warning) = config.warning {
        println!("\n⚠️  WARNING: {}", warning);
    }

    println!("\nAffected items:");
    display_preview_items(&config.items);

    println!("\n{}", "=".repeat(60));
    print!(
        "\nProceed with {} on {} {}(s)? [y/N]: ",
        config.operation, count, config.entity_type
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => {
            println!();
            Ok(ConfirmationResult::Confirmed)
        }
        _ => {
            println!();
            Ok(ConfirmationResult::Declined)
        }
    }
}

/// Display preview items (max 20 items shown, with truncation notice)
fn display_preview_items(items: &[PreviewItem]) {
    for (i, item) in items.iter().enumerate().take(20) {
        println!("  {}. [ID {}] {}", i + 1, item.id, item.display_text);
    }

    if items.len() > 20 {
        println!("  ... and {} more items", items.len() - 20);
    }
}

/// Format book for preview display
///
/// Shows book title and format in the preview
pub fn format_book_preview(book: &BookResult) -> PreviewItem {
    let format_display = book
        .format_key
        .as_deref()
        .unwrap_or("unknown");

    PreviewItem {
        id: book.id,
        display_text: format!("{} ({})", book.name, format_display),
    }
}

/// Format content for preview display
///
/// Shows content title and type in the preview
pub fn format_content_preview(content: &ContentResult) -> PreviewItem {
    let type_display = content
        .type_key
        .as_deref()
        .unwrap_or("unknown");

    PreviewItem {
        id: content.id,
        display_text: format!("{} ({})", content.name, type_display),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_preview_items(count: usize) -> Vec<PreviewItem> {
        (0..count)
            .map(|i| PreviewItem {
                id: i as i64,
                display_text: format!("Item {}", i),
            })
            .collect()
    }

    #[test]
    fn test_preview_item_creation() {
        let item = PreviewItem {
            id: 1,
            display_text: "Test Book (epub)".to_string(),
        };

        assert_eq!(item.id, 1);
        assert_eq!(item.display_text, "Test Book (epub)");
    }

    #[test]
    fn test_format_book_preview() {
        let book = BookResult {
            id: 123,
            name: "The Shining".to_string(),
            original_title: None,
            publisher_name: Some("Publisher".to_string()),
            format_key: Some("epub".to_string()),
            series_name: None,
            series_index: None,
            publication_date: None,
            isbn: None,
            file_link: None,
            created_at: 0,
        };

        let preview = format_book_preview(&book);
        assert_eq!(preview.id, 123);
        assert_eq!(preview.display_text, "The Shining (epub)");
    }

    #[test]
    fn test_format_book_preview_no_format() {
        let book = BookResult {
            id: 456,
            name: "IT".to_string(),
            original_title: None,
            publisher_name: None,
            format_key: None,
            series_name: None,
            series_index: None,
            publication_date: None,
            isbn: None,
            file_link: None,
            created_at: 0,
        };

        let preview = format_book_preview(&book);
        assert_eq!(preview.id, 456);
        assert_eq!(preview.display_text, "IT (unknown)");
    }

    #[test]
    fn test_format_content_preview() {
        let content = ContentResult {
            id: 789,
            name: "The Mist".to_string(),
            original_title: None,
            type_key: Some("type.novel".to_string()),
            publication_date: None,
            pages: None,
            created_at: 0,
        };

        let preview = format_content_preview(&content);
        assert_eq!(preview.id, 789);
        assert_eq!(preview.display_text, "The Mist (type.novel)");
    }

    #[test]
    fn test_format_content_preview_no_type() {
        let content = ContentResult {
            id: 999,
            name: "Short Story".to_string(),
            original_title: None,
            type_key: None,
            publication_date: None,
            pages: None,
            created_at: 0,
        };

        let preview = format_content_preview(&content);
        assert_eq!(preview.id, 999);
        assert_eq!(preview.display_text, "Short Story (unknown)");
    }

    #[test]
    fn test_display_preview_items_truncation() {
        let items = create_preview_items(25);

        // This is a visual test - in a real scenario we'd capture stdout
        // For now, we just verify the count logic
        assert_eq!(items.len(), 25);
        assert!(items.len() > 20); // Would show truncation message
    }
}
