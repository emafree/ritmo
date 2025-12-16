//! Command modules for ritmo CLI
//!
//! This module organizes all CLI commands into separate files for better maintainability.
//! Each command group is in its own file with clear responsibilities.

// Command modules
pub mod books;
pub mod contents;
pub mod init;
pub mod libraries;
pub mod presets;

// Re-export command functions for convenience
pub use books::{cmd_add, cmd_list_books};
pub use contents::cmd_list_contents;
pub use init::cmd_init;
pub use libraries::{cmd_info, cmd_list_libraries, cmd_set_library};
pub use presets::{cmd_delete_preset, cmd_list_presets, cmd_save_preset, cmd_set_default_filter};
