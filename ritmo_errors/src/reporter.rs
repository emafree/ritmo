//! # RitmoReporter - Output Abstraction Layer
//!
//! This module provides a trait-based abstraction for reporting status, progress,
//! and errors without direct console I/O. This allows shared library code to
//! communicate with different frontends (CLI, GUI) without being tied to println!.
//!
//! ## Design Goals
//!
//! - **GUI Compatibility**: Shared modules don't use println!, allowing GUI integration
//! - **Testability**: SilentReporter allows tests to run without console output
//! - **Flexibility**: Different frontends can implement custom reporting behavior
//!
//! ## Usage Example
//!
//! ```rust
//! use ritmo_errors::reporter::RitmoReporter;
//!
//! fn process_data(reporter: &mut impl RitmoReporter) {
//!     reporter.status("Starting data processing...");
//!
//!     for i in 0..100 {
//!         reporter.progress(&format!("Processing item {}/100", i + 1));
//!     }
//!
//!     if let Err(e) = do_work() {
//!         reporter.error(&format!("Failed to process: {}", e));
//!     }
//!
//!     reporter.status("Processing complete");
//! }
//! # fn do_work() -> Result<(), String> { Ok(()) }
//! ```
//!
//! ## Implementations
//!
//! - **SilentReporter**: No-op implementation for library usage and tests
//! - **CliReporter**: (implemented in ritmo_cli) - prints to stdout/stderr
//! - **GuiReporter**: (to be implemented in ritmo_gui) - updates GUI components
//!
//! ## GuiReporter Implementation Guidelines
//!
//! When implementing GuiReporter for the GUI application, follow this pattern:
//!
//! ```rust,ignore
//! use ritmo_errors::reporter::RitmoReporter;
//! use slint::ComponentHandle; // or your GUI framework
//!
//! pub struct GuiReporter {
//!     window: slint::Weak<MainWindow>,
//! }
//!
//! impl GuiReporter {
//!     pub fn new(window: slint::Weak<MainWindow>) -> Self {
//!         Self { window }
//!     }
//! }
//!
//! impl RitmoReporter for GuiReporter {
//!     fn status(&mut self, message: &str) {
//!         if let Some(window) = self.window.upgrade() {
//!             window.set_status_message(message.into());
//!         }
//!     }
//!
//!     fn progress(&mut self, message: &str) {
//!         if let Some(window) = self.window.upgrade() {
//!             window.set_progress_message(message.into());
//!             // Optionally update progress bar
//!         }
//!     }
//!
//!     fn error(&mut self, message: &str) {
//!         if let Some(window) = self.window.upgrade() {
//!             window.set_error_message(message.into());
//!             // Optionally show error dialog
//!         }
//!     }
//! }
//! ```
//!
//! ### Usage in GUI Application
//!
//! When calling library functions from GUI code:
//!
//! ```rust,ignore
//! use ritmo_db_core::LibraryConfig;
//!
//! // In your GUI event handler
//! let config = LibraryConfig::new(&library_path);
//! let mut reporter = GuiReporter::new(window.as_weak());
//! let pool = config.create_pool(&mut reporter).await?;
//! // Now status messages will appear in the GUI instead of being silent
//! ```
//!
//! ### Thread Safety Considerations
//!
//! If using the reporter across threads (e.g., with async tasks), you may need:
//! - A thread-safe message queue (e.g., `Arc<Mutex<VecDeque<String>>>`)
//! - Periodic polling from the GUI thread to update UI
//! - Or use your GUI framework's built-in async/thread communication

/// Trait for reporting progress, status, and errors without direct console I/O.
///
/// This trait provides a uniform interface for shared library code to communicate
/// with different frontends (CLI, GUI) without being tied to specific output mechanisms.
pub trait RitmoReporter {
    /// Report general status information.
    ///
    /// Use this for important informational messages that the user should see,
    /// such as "Database connected", "File loaded", etc.
    ///
    /// # Example
    /// ```
    /// # use ritmo_errors::reporter::{RitmoReporter, SilentReporter};
    /// let mut reporter = SilentReporter;
    /// reporter.status("Database initialized");
    /// ```
    fn status(&mut self, message: &str);

    /// Report progress updates.
    ///
    /// Use this for incremental progress messages during long operations,
    /// such as "Processing file 5/100", "Loading records...", etc.
    ///
    /// # Example
    /// ```
    /// # use ritmo_errors::reporter::{RitmoReporter, SilentReporter};
    /// let mut reporter = SilentReporter;
    /// for i in 0..10 {
    ///     reporter.progress(&format!("Processing {}/10", i + 1));
    /// }
    /// ```
    fn progress(&mut self, message: &str);

    /// Report error messages.
    ///
    /// Use this for non-fatal errors that should be communicated to the user,
    /// such as "Failed to delete file", "Warning: connection timeout", etc.
    ///
    /// Note: This is for user-facing error messages, not for Result<T, E> errors.
    /// Use Result for recoverable errors in your function signatures.
    ///
    /// # Example
    /// ```
    /// # use ritmo_errors::reporter::{RitmoReporter, SilentReporter};
    /// let mut reporter = SilentReporter;
    /// reporter.error("Failed to connect to server");
    /// ```
    fn error(&mut self, message: &str);
}

/// Silent reporter implementation that discards all messages.
///
/// This is the default implementation for library usage and tests.
/// It implements RitmoReporter but does nothing with the messages.
///
/// # Example
/// ```
/// use ritmo_errors::reporter::{RitmoReporter, SilentReporter};
///
/// fn my_function(reporter: &mut impl RitmoReporter) {
///     reporter.status("Starting work");
///     // ... do work ...
/// }
///
/// // In tests or library code
/// let mut reporter = SilentReporter;
/// my_function(&mut reporter);
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct SilentReporter;

impl RitmoReporter for SilentReporter {
    fn status(&mut self, _message: &str) {
        // Silent: do nothing
    }

    fn progress(&mut self, _message: &str) {
        // Silent: do nothing
    }

    fn error(&mut self, _message: &str) {
        // Silent: do nothing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silent_reporter_does_not_panic() {
        let mut reporter = SilentReporter;
        reporter.status("test status");
        reporter.progress("test progress");
        reporter.error("test error");
        // Test passes if no panic occurs
    }

    #[test]
    fn test_reporter_trait_is_object_safe() {
        // This test ensures RitmoReporter can be used as a trait object
        let mut reporter = SilentReporter;
        let reporter_ref: &mut dyn RitmoReporter = &mut reporter;
        reporter_ref.status("test");
    }

    #[test]
    fn test_silent_reporter_default() {
        let mut reporter = SilentReporter::default();
        reporter.status("test");
        // Test passes if default() works
    }
}
