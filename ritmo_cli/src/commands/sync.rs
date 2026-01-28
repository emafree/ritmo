use crate::helpers::get_library_path;
use ritmo_config::AppSettings;
use ritmo_core::service::metadata_sync_service::sync_book_metadata;
use ritmo_db::pending_sync::{count_pending_sync, get_pending_sync_books};
use ritmo_db::Book;
use ritmo_db_core::LibraryConfig;
use ritmo_errors::reporter::SilentReporter;
use std::path::PathBuf;

/// Command: sync-metadata --status
pub async fn cmd_sync_status(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;
    let config = LibraryConfig::new(&library_path);
    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

    let count = count_pending_sync(&pool).await?;

    if count == 0 {
        println!("✓ No books pending metadata sync");
    } else {
        println!("📊 Books pending metadata sync: {}", count);
        println!("\nRun 'ritmo sync-metadata' to sync EPUB files with database metadata");
        println!("Run 'ritmo sync-metadata --dry-run' to preview changes");
    }

    Ok(())
}

/// Command: sync-metadata --dry-run
pub async fn cmd_sync_dry_run(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;
    let config = LibraryConfig::new(&library_path);
    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

    let book_ids = get_pending_sync_books(&pool).await?;

    if book_ids.is_empty() {
        println!("✓ No books pending metadata sync");
        return Ok(());
    }

    println!(
        "🔍 Dry-run: {} books would be synchronized",
        book_ids.len()
    );
    println!("\nBooks that would be updated:");
    for book_id in &book_ids {
        // Get book name for display
        if let Some(book) = Book::get(&pool, *book_id).await? {
            println!("  • [{}] {}", book_id, book.name);
        }
    }

    println!("\n⚠️  Dry-run mode: No changes were made");
    println!("Run 'ritmo sync-metadata' without --dry-run to perform sync");

    Ok(())
}

/// Command: sync-metadata (default - actually sync)
pub async fn cmd_sync_metadata(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;
    let config = LibraryConfig::new(&library_path);
    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

    let book_ids = get_pending_sync_books(&pool).await?;

    if book_ids.is_empty() {
        println!("✓ No books pending metadata sync");
        return Ok(());
    }

    println!(
        "🔄 Synchronizing metadata for {} books...\n",
        book_ids.len()
    );

    let mut success_count = 0;
    let mut error_count = 0;

    for (i, book_id) in book_ids.iter().enumerate() {
        print!(
            "[{}/{}] Syncing book ID {}... ",
            i + 1,
            book_ids.len(),
            book_id
        );

        match sync_book_metadata(&config, &pool, *book_id).await {
            Ok(result) => {
                println!("✓");
                println!("  Old hash: {}", &result.old_hash[..16]);
                println!("  New hash: {}", &result.new_hash[..16]);
                if result.old_hash != result.new_hash {
                    println!(
                        "  Moved: {} → {}",
                        result
                            .old_path
                            .file_name()
                            .unwrap()
                            .to_string_lossy(),
                        result.new_path.file_name().unwrap().to_string_lossy()
                    );
                }
                success_count += 1;
            }
            Err(e) => {
                println!("✗");
                eprintln!("  Error: {:?}", e);
                error_count += 1;
            }
        }
    }

    println!("\n📊 Sync Summary:");
    println!("  ✓ Successful: {}", success_count);
    if error_count > 0 {
        println!("  ✗ Failed: {}", error_count);
    }

    Ok(())
}
