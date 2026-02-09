//! Command modules for ritmo CLI
//!
//! This module organizes all CLI commands into separate files for better maintainability.
//! Each command group is in its own file with clear responsibilities.

// Command modules
pub mod deduplication;
pub mod init;
pub mod language;
pub mod libraries;
pub mod presets;
pub mod sync;

// Re-export command functions for convenience
pub use deduplication::{
    cmd_deduplicate_all, cmd_deduplicate_people, cmd_deduplicate_publishers,
    cmd_deduplicate_roles, cmd_deduplicate_series, cmd_deduplicate_tags,
};
pub use init::cmd_init;
pub use language::{cmd_get_language, cmd_set_language};
pub use libraries::{cmd_info, cmd_list_libraries, cmd_set_library};
pub use presets::{cmd_delete_preset, cmd_list_presets, cmd_save_preset, cmd_set_default_filter};
pub use sync::{cmd_sync_dry_run, cmd_sync_metadata, cmd_sync_status};
