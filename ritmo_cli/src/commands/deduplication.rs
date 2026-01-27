//! ML deduplication commands for finding and merging duplicate entities

use crate::helpers::get_library_path;
use ritmo_config::AppSettings;
use ritmo_db_core::LibraryConfig;
use ritmo_errors::reporter::SilentReporter;
use ritmo_ml::deduplication::{
    deduplicate_people, deduplicate_publishers, deduplicate_roles, deduplicate_series,
    deduplicate_tags, DeduplicationConfig, DeduplicationResult,
};
use std::path::PathBuf;

/// Print deduplication results in a user-friendly format
fn print_deduplication_results(result: &DeduplicationResult, entity_type: &str, dry_run: bool) {
    println!("📊 Deduplication Results for {}:", entity_type);
    println!("   Total entities processed: {}", result.total_entities);
    println!(
        "   Duplicate groups found: {}",
        result.duplicate_groups.len()
    );

    if result.duplicate_groups.is_empty() {
        println!("✓ No duplicates found! Database is clean.");
        return;
    }

    println!("\n📋 Duplicate Groups:");
    for (i, group) in result.duplicate_groups.iter().enumerate() {
        println!(
            "\n   Group {} (confidence: {:.2}%):",
            i + 1,
            group.confidence * 100.0
        );
        println!("     Primary: {} (ID: {})", group.primary_name, group.primary_id);
        println!("     Duplicates:");
        for (j, (dup_id, dup_name)) in group
            .duplicate_ids
            .iter()
            .zip(group.duplicate_names.iter())
            .enumerate()
        {
            println!("       {}. {} (ID: {})", j + 1, dup_name, dup_id);
        }
    }

    if dry_run {
        println!("\n🔍 Dry-run mode: No changes were made to the database");
        println!(
            "   Run without --dry-run to merge these duplicates (if --auto-merge is set)"
        );
    } else if !result.merged_groups.is_empty() {
        println!("\n✓ Merged {} groups:", result.merged_groups.len());
        for (i, stats) in result.merged_groups.iter().enumerate() {
            println!(
                "   {}. Primary ID {}: merged {} duplicates ({} books, {} contents updated)",
                i + 1,
                stats.primary_id,
                stats.merged_ids.len(),
                stats.books_updated,
                stats.contents_updated
            );
        }
    } else {
        println!("\n⚠️  No auto-merge performed (use --auto-merge to enable)");
    }

    if result.skipped_low_confidence > 0 {
        println!(
            "\n⏭️  Skipped {} groups due to low confidence",
            result.skipped_low_confidence
        );
    }
}

/// Command: deduplicate-people - Find and merge duplicate people (authors, translators, etc.)
pub async fn cmd_deduplicate_people(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    threshold: f64,
    auto_merge: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;

    let config = LibraryConfig::new(&library_path);
    if !config.exists() {
        return Err(format!("Library does not exist: {}", library_path.display()).into());
    }

    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

    println!("🔍 Searching for duplicate people...");

    // Default to dry-run mode for safety (invert the flag logic)
    let actual_dry_run = if auto_merge && !dry_run {
        false  // Only disable dry-run if auto-merge is requested AND --dry-run was NOT passed
    } else {
        true   // Default to dry-run in all other cases
    };

    let dedup_config = DeduplicationConfig {
        min_confidence: threshold,
        min_frequency: 2,
        auto_merge,
        dry_run: actual_dry_run,
    };

    match deduplicate_people(&pool, &dedup_config).await {
        Ok(result) => {
            print_deduplication_results(&result, "People", dry_run);
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Error during deduplication: {}", e);
            Err(e.into())
        }
    }
}

/// Command: deduplicate-publishers - Find and merge duplicate publishers
pub async fn cmd_deduplicate_publishers(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    threshold: f64,
    auto_merge: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;

    let config = LibraryConfig::new(&library_path);
    if !config.exists() {
        return Err(format!("Library does not exist: {}", library_path.display()).into());
    }

    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

    println!("🔍 Searching for duplicate publishers...");

    // Default to dry-run mode for safety (invert the flag logic)
    let actual_dry_run = if auto_merge && !dry_run {
        false  // Only disable dry-run if auto-merge is requested AND --dry-run was NOT passed
    } else {
        true   // Default to dry-run in all other cases
    };

    let dedup_config = DeduplicationConfig {
        min_confidence: threshold,
        min_frequency: 2,
        auto_merge,
        dry_run: actual_dry_run,
    };

    match deduplicate_publishers(&pool, &dedup_config).await {
        Ok(result) => {
            print_deduplication_results(&result, "Publishers", dry_run);
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Error during deduplication: {}", e);
            Err(e.into())
        }
    }
}

/// Command: deduplicate-series - Find and merge duplicate series
pub async fn cmd_deduplicate_series(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    threshold: f64,
    auto_merge: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;

    let config = LibraryConfig::new(&library_path);
    if !config.exists() {
        return Err(format!("Library does not exist: {}", library_path.display()).into());
    }

    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

    println!("🔍 Searching for duplicate series...");

    // Default to dry-run mode for safety (invert the flag logic)
    let actual_dry_run = if auto_merge && !dry_run {
        false  // Only disable dry-run if auto-merge is requested AND --dry-run was NOT passed
    } else {
        true   // Default to dry-run in all other cases
    };

    let dedup_config = DeduplicationConfig {
        min_confidence: threshold,
        min_frequency: 2,
        auto_merge,
        dry_run: actual_dry_run,
    };

    match deduplicate_series(&pool, &dedup_config).await {
        Ok(result) => {
            print_deduplication_results(&result, "Series", dry_run);
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Error during deduplication: {}", e);
            Err(e.into())
        }
    }
}

/// Command: deduplicate-tags - Find and merge duplicate tags
pub async fn cmd_deduplicate_tags(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    threshold: f64,
    auto_merge: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;

    let config = LibraryConfig::new(&library_path);
    if !config.exists() {
        return Err(format!("Library does not exist: {}", library_path.display()).into());
    }

    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

    println!("🔍 Searching for duplicate tags...");

    // Default to dry-run mode for safety (invert the flag logic)
    let actual_dry_run = if auto_merge && !dry_run {
        false  // Only disable dry-run if auto-merge is requested AND --dry-run was NOT passed
    } else {
        true   // Default to dry-run in all other cases
    };

    let dedup_config = DeduplicationConfig {
        min_confidence: threshold,
        min_frequency: 2,
        auto_merge,
        dry_run: actual_dry_run,
    };

    match deduplicate_tags(&pool, &dedup_config).await {
        Ok(result) => {
            print_deduplication_results(&result, "Tags", dry_run);
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Error during deduplication: {}", e);
            Err(e.into())
        }
    }
}

/// Command: deduplicate-roles - Find and merge duplicate roles
pub async fn cmd_deduplicate_roles(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    threshold: f64,
    auto_merge: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;

    let config = LibraryConfig::new(&library_path);
    if !config.exists() {
        return Err(format!("Library does not exist: {}", library_path.display()).into());
    }

    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

    println!("🔍 Searching for duplicate roles...");

    // Default to dry-run mode for safety (invert the flag logic)
    let actual_dry_run = if auto_merge && !dry_run {
        false  // Only disable dry-run if auto-merge is requested AND --dry-run was NOT passed
    } else {
        true   // Default to dry-run in all other cases
    };

    let dedup_config = DeduplicationConfig {
        min_confidence: threshold,
        min_frequency: 2,
        auto_merge,
        dry_run: actual_dry_run,
    };

    match deduplicate_roles(&pool, &dedup_config).await {
        Ok(result) => {
            print_deduplication_results(&result, "Roles", dry_run);
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Error during deduplication: {}", e);
            Err(e.into())
        }
    }
}

/// Command: deduplicate-all - Find and merge duplicates for all entity types
pub async fn cmd_deduplicate_all(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    threshold: f64,
    auto_merge: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;

    let config = LibraryConfig::new(&library_path);
    if !config.exists() {
        return Err(format!("Library does not exist: {}", library_path.display()).into());
    }

    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

    println!("🔍 Searching for duplicates across all entity types...\n");

    // Default to dry-run mode for safety (invert the flag logic)
    let actual_dry_run = if auto_merge && !dry_run {
        false  // Only disable dry-run if auto-merge is requested AND --dry-run was NOT passed
    } else {
        true   // Default to dry-run in all other cases
    };

    let dedup_config = DeduplicationConfig {
        min_confidence: threshold,
        min_frequency: 2,
        auto_merge,
        dry_run: actual_dry_run,
    };

    // Deduplicate people (authors, translators, etc.)
    println!("═══════════════════════════════════════════════════════");
    println!("👥 PEOPLE");
    println!("═══════════════════════════════════════════════════════");
    match deduplicate_people(&pool, &dedup_config).await {
        Ok(result) => print_deduplication_results(&result, "People", dry_run),
        Err(e) => eprintln!("✗ Error deduplicating people: {}", e),
    }

    // Deduplicate publishers
    println!("\n═══════════════════════════════════════════════════════");
    println!("🏢 PUBLISHERS");
    println!("═══════════════════════════════════════════════════════");
    match deduplicate_publishers(&pool, &dedup_config).await {
        Ok(result) => print_deduplication_results(&result, "Publishers", dry_run),
        Err(e) => eprintln!("✗ Error deduplicating publishers: {}", e),
    }

    // Deduplicate series
    println!("\n═══════════════════════════════════════════════════════");
    println!("📚 SERIES");
    println!("═══════════════════════════════════════════════════════");
    match deduplicate_series(&pool, &dedup_config).await {
        Ok(result) => print_deduplication_results(&result, "Series", dry_run),
        Err(e) => eprintln!("✗ Error deduplicating series: {}", e),
    }

    // Deduplicate tags
    println!("\n═══════════════════════════════════════════════════════");
    println!("🏷️  TAGS");
    println!("═══════════════════════════════════════════════════════");
    match deduplicate_tags(&pool, &dedup_config).await {
        Ok(result) => print_deduplication_results(&result, "Tags", dry_run),
        Err(e) => eprintln!("✗ Error deduplicating tags: {}", e),
    }

    // Deduplicate roles
    println!("\n═══════════════════════════════════════════════════════");
    println!("🎭 ROLES");
    println!("═══════════════════════════════════════════════════════");
    match deduplicate_roles(&pool, &dedup_config).await {
        Ok(result) => print_deduplication_results(&result, "Roles", dry_run),
        Err(e) => eprintln!("✗ Error deduplicating roles: {}", e),
    }

    println!("\n✓ Deduplication complete for all entity types!");

    Ok(())
}
