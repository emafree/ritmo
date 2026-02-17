//! ML deduplication commands for finding and merging duplicate entities

use crate::helpers::get_library_path;
use ritmo_config::AppSettings;
use ritmo_db::mark_books_for_sync;
use ritmo_db_core::LibraryConfig;
use ritmo_errors::reporter::SilentReporter;
use ritmo_ml::deduplication::{
    deduplicate_genres, deduplicate_people, deduplicate_publishers, deduplicate_roles, deduplicate_series,
    deduplicate_tags, filter_duplicate_groups_by_entity, DeduplicationConfig, DeduplicationResult,
    DuplicateGroup,
};
use std::io::{self, Write};
use std::path::PathBuf;

/// Show interactive menu for selecting canonical entity from a duplicate group
///
/// Returns the ID of the selected entity, or None if cancelled
fn show_interactive_menu(group: &DuplicateGroup) -> Option<i64> {
    println!("\n┌─────────────────────────────────────────────────────────");
    println!("│ 🔍 Duplicate entities detected!");
    println!("│ Please select which entity to keep as canonical:");
    println!("├─────────────────────────────────────────────────────────");
    
    // Show all options with indices
    let mut all_options = vec![(group.primary_id, group.primary_name.clone())];
    all_options.extend(
        group.duplicate_ids.iter()
            .zip(group.duplicate_names.iter())
            .map(|(id, name)| (*id, name.clone()))
    );
    
    for (i, (id, name)) in all_options.iter().enumerate() {
        println!("│ {}. {} (ID: {})", i + 1, name, id);
    }
    
    println!("│ 0. Cancel merge");
    println!("└─────────────────────────────────────────────────────────");
    
    // Get user input
    loop {
        print!("Select option (0-{}): ", all_options.len());
        io::stdout().flush().ok()?;
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return None;
        }
        
        let input = input.trim();
        if let Ok(choice) = input.parse::<usize>() {
            if choice == 0 {
                println!("❌ Merge cancelled");
                return None;
            }
            
            if choice > 0 && choice <= all_options.len() {
                let (selected_id, selected_name) = &all_options[choice - 1];
                println!("✓ Selected: {} (ID: {})", selected_name, selected_id);
                return Some(*selected_id);
            }
        }
        
        println!("❌ Invalid choice. Please enter a number between 0 and {}", all_options.len());
    }
}

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

/// Process duplicate groups interactively, allowing user to select canonical entity
///
/// Returns the processed groups with user-selected primary entities, or an empty vector
/// if all merges were cancelled.
fn process_groups_interactively(groups: &[DuplicateGroup]) -> Vec<DuplicateGroup> {
    println!("\n🎯 Interactive mode enabled");
    
    let mut processed_groups = Vec::new();
    
    for group in groups {
        if let Some(selected_id) = show_interactive_menu(group) {
            // Create a new group with the selected entity as primary
            let mut new_group = group.clone();
            
            // If selected_id is not already primary, swap it
            if selected_id != group.primary_id {
                // Find the selected entity in duplicates
                if let Some(pos) = group.duplicate_ids.iter().position(|&id| id == selected_id) {
                    // Swap primary with selected duplicate
                    new_group.primary_id = selected_id;
                    new_group.primary_name = group.duplicate_names[pos].clone();
                    
                    // Add old primary to duplicates
                    new_group.duplicate_ids = vec![group.primary_id];
                    new_group.duplicate_names = vec![group.primary_name.clone()];
                    
                    // Add remaining duplicates
                    for (i, &dup_id) in group.duplicate_ids.iter().enumerate() {
                        if i != pos {
                            new_group.duplicate_ids.push(dup_id);
                            new_group.duplicate_names.push(group.duplicate_names[i].clone());
                        }
                    }
                }
            }
            
            processed_groups.push(new_group);
        }
    }
    
    processed_groups
}

/// Command: deduplicate-people - Find and merge duplicate people (authors, translators, etc.)
pub async fn cmd_deduplicate_people(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    entity_name: Option<String>,
    threshold: f64,
    auto_merge: bool,
    dry_run: bool,
    interactive: bool,
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
        Ok(mut result) => {
            // Filter by entity name if provided
            if let Some(ref name) = entity_name {
                let original_count = result.duplicate_groups.len();
                result.duplicate_groups = filter_duplicate_groups_by_entity(&result.duplicate_groups, name);
                
                if result.duplicate_groups.is_empty() {
                    println!("✓ No duplicates found for entity: {}", name);
                    return Ok(());
                }
                
                println!("📋 Filtered {} -> {} groups matching '{}'", 
                    original_count, result.duplicate_groups.len(), name);
            }

            // Interactive mode: let user choose canonical entity
            if interactive && !result.duplicate_groups.is_empty() {
                let processed_groups = process_groups_interactively(&result.duplicate_groups);
                
                if processed_groups.is_empty() {
                    println!("\n⚠️  All merges were cancelled");
                    return Ok(());
                }
                
                // Replace duplicate_groups with processed ones
                result.duplicate_groups = processed_groups;
                
                // Now perform the merge if not in dry-run
                if !actual_dry_run {
                    use ritmo_ml::merge::merge_people;
                    let mut merged_groups = Vec::new();
                    
                    for group in &result.duplicate_groups {
                        match merge_people(&pool, group.primary_id, &group.duplicate_ids).await {
                            Ok(stats) => {
                                merged_groups.push(stats);
                                println!("✓ Merged group with primary ID: {}", group.primary_id);
                            }
                            Err(e) => {
                                eprintln!("✗ Error merging group (primary={}): {}", group.primary_id, e);
                            }
                        }
                    }
                    
                    result.merged_groups = merged_groups;
                }
            }

            print_deduplication_results(&result, "People", dry_run);

            // Mark affected books for sync if not dry-run
            if !actual_dry_run && !result.merged_groups.is_empty() {
                let mut all_affected_books = Vec::new();
                for stats in &result.merged_groups {
                    all_affected_books.extend(&stats.affected_book_ids);
                }
                all_affected_books.sort();
                all_affected_books.dedup();

                if !all_affected_books.is_empty() {
                    mark_books_for_sync(&pool, &all_affected_books, "author_deduplicate").await?;
                    println!("\n📝 Marked {} books for metadata sync", all_affected_books.len());
                    println!("   Run 'ritmo sync-metadata' to update EPUB files with new metadata");
                }
            }

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
    entity_name: Option<String>,
    threshold: f64,
    auto_merge: bool,
    dry_run: bool,
    interactive: bool,
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
        Ok(mut result) => {
            // Filter by entity name if provided
            if let Some(ref name) = entity_name {
                let original_count = result.duplicate_groups.len();
                result.duplicate_groups = filter_duplicate_groups_by_entity(&result.duplicate_groups, name);
                
                if result.duplicate_groups.is_empty() {
                    println!("✓ No duplicates found for entity: {}", name);
                    return Ok(());
                }
                
                println!("📋 Filtered {} -> {} groups matching '{}'", 
                    original_count, result.duplicate_groups.len(), name);
            }

            // Interactive mode: let user choose canonical entity
            if interactive && !result.duplicate_groups.is_empty() {
                let processed_groups = process_groups_interactively(&result.duplicate_groups);
                
                if processed_groups.is_empty() {
                    println!("\n⚠️  All merges were cancelled");
                    return Ok(());
                }
                
                // Replace duplicate_groups with processed ones
                result.duplicate_groups = processed_groups;
                
                // Now perform the merge if not in dry-run
                if !actual_dry_run {
                    use ritmo_ml::merge::merge_publishers;
                    let mut merged_groups = Vec::new();
                    
                    for group in &result.duplicate_groups {
                        match merge_publishers(&pool, group.primary_id, &group.duplicate_ids).await {
                            Ok(stats) => {
                                merged_groups.push(stats);
                                println!("✓ Merged group with primary ID: {}", group.primary_id);
                            }
                            Err(e) => {
                                eprintln!("✗ Error merging group (primary={}): {}", group.primary_id, e);
                            }
                        }
                    }
                    
                    result.merged_groups = merged_groups;
                }
            }

            print_deduplication_results(&result, "Publishers", dry_run);

            // Mark affected books for sync if not dry-run
            if !actual_dry_run && !result.merged_groups.is_empty() {
                let mut all_affected_books = Vec::new();
                for stats in &result.merged_groups {
                    all_affected_books.extend(&stats.affected_book_ids);
                }
                all_affected_books.sort();
                all_affected_books.dedup();

                if !all_affected_books.is_empty() {
                    mark_books_for_sync(&pool, &all_affected_books, "publisher_deduplicate").await?;
                    println!("\n📝 Marked {} books for metadata sync", all_affected_books.len());
                    println!("   Run 'ritmo sync-metadata' to update EPUB files with new metadata");
                }
            }

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
    entity_name: Option<String>,
    threshold: f64,
    auto_merge: bool,
    dry_run: bool,
    interactive: bool,
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
        Ok(mut result) => {
            // Filter by entity name if provided
            if let Some(ref name) = entity_name {
                let original_count = result.duplicate_groups.len();
                result.duplicate_groups = filter_duplicate_groups_by_entity(&result.duplicate_groups, name);
                
                if result.duplicate_groups.is_empty() {
                    println!("✓ No duplicates found for entity: {}", name);
                    return Ok(());
                }
                
                println!("📋 Filtered {} -> {} groups matching '{}'", 
                    original_count, result.duplicate_groups.len(), name);
            }

            // Interactive mode: let user choose canonical entity
            if interactive && !result.duplicate_groups.is_empty() {
                let processed_groups = process_groups_interactively(&result.duplicate_groups);
                
                if processed_groups.is_empty() {
                    println!("\n⚠️  All merges were cancelled");
                    return Ok(());
                }
                
                // Replace duplicate_groups with processed ones
                result.duplicate_groups = processed_groups;
                
                // Now perform the merge if not in dry-run
                if !actual_dry_run {
                    use ritmo_ml::merge::merge_series;
                    let mut merged_groups = Vec::new();
                    
                    for group in &result.duplicate_groups {
                        match merge_series(&pool, group.primary_id, &group.duplicate_ids).await {
                            Ok(stats) => {
                                merged_groups.push(stats);
                                println!("✓ Merged group with primary ID: {}", group.primary_id);
                            }
                            Err(e) => {
                                eprintln!("✗ Error merging group (primary={}): {}", group.primary_id, e);
                            }
                        }
                    }
                    
                    result.merged_groups = merged_groups;
                }
            }

            print_deduplication_results(&result, "Series", dry_run);

            // Mark affected books for sync if not dry-run
            if !actual_dry_run && !result.merged_groups.is_empty() {
                let mut all_affected_books = Vec::new();
                for stats in &result.merged_groups {
                    all_affected_books.extend(&stats.affected_book_ids);
                }
                all_affected_books.sort();
                all_affected_books.dedup();

                if !all_affected_books.is_empty() {
                    mark_books_for_sync(&pool, &all_affected_books, "series_deduplicate").await?;
                    println!("\n📝 Marked {} books for metadata sync", all_affected_books.len());
                    println!("   Run 'ritmo sync-metadata' to update EPUB files with new metadata");
                }
            }

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
    entity_name: Option<String>,
    threshold: f64,
    auto_merge: bool,
    dry_run: bool,
    interactive: bool,
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
        Ok(mut result) => {
            // Filter by entity name if provided
            if let Some(ref name) = entity_name {
                let original_count = result.duplicate_groups.len();
                result.duplicate_groups = filter_duplicate_groups_by_entity(&result.duplicate_groups, name);
                
                if result.duplicate_groups.is_empty() {
                    println!("✓ No duplicates found for entity: {}", name);
                    return Ok(());
                }
                
                println!("📋 Filtered {} -> {} groups matching '{}'", 
                    original_count, result.duplicate_groups.len(), name);
            }

            // Interactive mode: let user choose canonical entity
            if interactive && !result.duplicate_groups.is_empty() {
                let processed_groups = process_groups_interactively(&result.duplicate_groups);
                
                if processed_groups.is_empty() {
                    println!("\n⚠️  All merges were cancelled");
                    return Ok(());
                }
                
                // Replace duplicate_groups with processed ones
                result.duplicate_groups = processed_groups;
                
                // Now perform the merge if not in dry-run
                if !actual_dry_run {
                    use ritmo_ml::merge::merge_tags;
                    let mut merged_groups = Vec::new();
                    
                    for group in &result.duplicate_groups {
                        match merge_tags(&pool, group.primary_id, &group.duplicate_ids).await {
                            Ok(stats) => {
                                merged_groups.push(stats);
                                println!("✓ Merged group with primary ID: {}", group.primary_id);
                            }
                            Err(e) => {
                                eprintln!("✗ Error merging group (primary={}): {}", group.primary_id, e);
                            }
                        }
                    }
                    
                    result.merged_groups = merged_groups;
                }
            }

            print_deduplication_results(&result, "Tags", dry_run);

            // Mark affected books for sync if not dry-run
            if !actual_dry_run && !result.merged_groups.is_empty() {
                let mut all_affected_books = Vec::new();
                for stats in &result.merged_groups {
                    all_affected_books.extend(&stats.affected_book_ids);
                }
                all_affected_books.sort();
                all_affected_books.dedup();

                if !all_affected_books.is_empty() {
                    mark_books_for_sync(&pool, &all_affected_books, "tag_deduplicate").await?;
                    println!("\n📝 Marked {} books for metadata sync", all_affected_books.len());
                    println!("   Run 'ritmo sync-metadata' to update EPUB files with new metadata");
                }
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Error during deduplication: {}", e);
            Err(e.into())
        }
    }
}

/// Command: deduplicate-genres - Find and merge duplicate genres
pub async fn cmd_deduplicate_genres(
    cli_library: &Option<PathBuf>,
    app_settings: &AppSettings,
    entity_name: Option<String>,
    threshold: f64,
    auto_merge: bool,
    dry_run: bool,
    interactive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path(cli_library, app_settings)?;

    let config = LibraryConfig::new(&library_path);
    if !config.exists() {
        return Err(format!("Library does not exist: {}", library_path.display()).into());
    }

    let mut reporter = SilentReporter;
    let pool = config.create_pool(&mut reporter).await?;

    println!("🔍 Searching for duplicate genres...");

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

    match deduplicate_genres(&pool, &dedup_config).await {
        Ok(mut result) => {
            // Filter by entity name if provided
            if let Some(ref name) = entity_name {
                let original_count = result.duplicate_groups.len();
                result.duplicate_groups = filter_duplicate_groups_by_entity(&result.duplicate_groups, name);
                
                if result.duplicate_groups.is_empty() {
                    println!("✓ No duplicates found for entity: {}", name);
                    return Ok(());
                }
                
                println!("📋 Filtered {} -> {} groups matching '{}'", 
                    original_count, result.duplicate_groups.len(), name);
            }

            // Interactive mode: let user choose canonical entity
            if interactive && !result.duplicate_groups.is_empty() {
                let processed_groups = process_groups_interactively(&result.duplicate_groups);
                
                if processed_groups.is_empty() {
                    println!("\n⚠️  All merges were cancelled");
                    return Ok(());
                }
                
                // Replace duplicate_groups with processed ones
                result.duplicate_groups = processed_groups;
                
                // Now perform the merge if not in dry-run
                if !actual_dry_run {
                    use ritmo_ml::merge::merge_genres;
                    let mut merged_groups = Vec::new();
                    
                    for group in &result.duplicate_groups {
                        match merge_genres(&pool, group.primary_id, &group.duplicate_ids).await {
                            Ok(stats) => {
                                merged_groups.push(stats);
                                println!("✓ Merged group with primary ID: {}", group.primary_id);
                            }
                            Err(e) => {
                                eprintln!("✗ Error merging group (primary={}): {}", group.primary_id, e);
                            }
                        }
                    }
                    
                    result.merged_groups = merged_groups;
                }
            }

            print_deduplication_results(&result, "Genres", dry_run);

            // Note: Genres don't affect book metadata, only content metadata
            // We don't need to mark books for sync

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
    entity_name: Option<String>,
    threshold: f64,
    auto_merge: bool,
    dry_run: bool,
    interactive: bool,
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
        Ok(mut result) => {
            // Filter by entity name if provided
            if let Some(ref name) = entity_name {
                let original_count = result.duplicate_groups.len();
                result.duplicate_groups = filter_duplicate_groups_by_entity(&result.duplicate_groups, name);
                
                if result.duplicate_groups.is_empty() {
                    println!("✓ No duplicates found for entity: {}", name);
                    return Ok(());
                }
                
                println!("📋 Filtered {} -> {} groups matching '{}'", 
                    original_count, result.duplicate_groups.len(), name);
            }

            // Interactive mode: let user choose canonical entity
            if interactive && !result.duplicate_groups.is_empty() {
                let processed_groups = process_groups_interactively(&result.duplicate_groups);
                
                if processed_groups.is_empty() {
                    println!("\n⚠️  All merges were cancelled");
                    return Ok(());
                }
                
                // Replace duplicate_groups with processed ones
                result.duplicate_groups = processed_groups;
                
                // Now perform the merge if not in dry-run
                if !actual_dry_run {
                    use ritmo_ml::merge::merge_roles;
                    let mut merged_groups = Vec::new();
                    
                    for group in &result.duplicate_groups {
                        match merge_roles(&pool, group.primary_id, &group.duplicate_ids).await {
                            Ok(stats) => {
                                merged_groups.push(stats);
                                println!("✓ Merged group with primary ID: {}", group.primary_id);
                            }
                            Err(e) => {
                                eprintln!("✗ Error merging group (primary={}): {}", group.primary_id, e);
                            }
                        }
                    }
                    
                    result.merged_groups = merged_groups;
                }
            }

            print_deduplication_results(&result, "Roles", dry_run);

            // Mark affected books for sync if not dry-run
            if !actual_dry_run && !result.merged_groups.is_empty() {
                let mut all_affected_books = Vec::new();
                for stats in &result.merged_groups {
                    all_affected_books.extend(&stats.affected_book_ids);
                }
                all_affected_books.sort();
                all_affected_books.dedup();

                if !all_affected_books.is_empty() {
                    mark_books_for_sync(&pool, &all_affected_books, "role_deduplicate").await?;
                    println!("\n📝 Marked {} books for metadata sync", all_affected_books.len());
                    println!("   Run 'ritmo sync-metadata' to update EPUB files with new metadata");
                }
            }

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

    // Deduplicate genres
    println!("\n═══════════════════════════════════════════════════════");
    println!("🎭 GENRES");
    println!("═══════════════════════════════════════════════════════");
    match deduplicate_genres(&pool, &dedup_config).await {
        Ok(result) => print_deduplication_results(&result, "Genres", dry_run),
        Err(e) => eprintln!("✗ Error deduplicating genres: {}", e),
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
