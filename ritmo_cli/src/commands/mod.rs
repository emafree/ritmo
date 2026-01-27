//! Command modules for ritmo CLI
//!
//! This module organizes all CLI commands into separate files for better maintainability.
//! Each command group is in its own file with clear responsibilities.

// Command modules
pub mod books;
pub mod cleanup;
pub mod contents;
pub mod deduplication;
pub mod init;
pub mod language;
pub mod libraries;
pub mod presets;

// Re-export command functions for convenience
pub use books::{cmd_add, cmd_delete_book, cmd_list_books, cmd_update_book};
pub use cleanup::cmd_cleanup;
pub use contents::{
    cmd_add_content, cmd_delete_content, cmd_link_content, cmd_list_contents,
    cmd_unlink_content, cmd_update_content,
};
pub use deduplication::{
    cmd_deduplicate_all, cmd_deduplicate_people, cmd_deduplicate_publishers,
    cmd_deduplicate_roles, cmd_deduplicate_series, cmd_deduplicate_tags,
};
pub use init::cmd_init;
pub use language::{cmd_get_language, cmd_set_language};
pub use libraries::{cmd_info, cmd_list_libraries, cmd_set_library};
pub use presets::{cmd_delete_preset, cmd_list_presets, cmd_save_preset, cmd_set_default_filter};
