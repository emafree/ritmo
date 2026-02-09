//! Command handlers for the ritmo CLI
//!
//! This module contains the implementation of all command handlers,
//! organized by command group (books, contents, libraries, presets, etc.).
//!
//! Each handler module implements the business logic for a specific set of commands.
//! Handlers are called from main.rs after command parsing.

// Common helper functions
pub mod common;

// Phase 2: Books handlers
pub mod books;

// Phase 3: Contents handlers
pub mod contents;
// pub mod deduplication;
// pub mod language;
// pub mod libraries;
// pub mod presets;
// pub mod sync;
