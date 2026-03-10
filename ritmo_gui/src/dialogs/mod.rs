mod book_dialog;
mod content_dialog;
mod filter_dialog;
mod people_row; // ← aggiunto

pub use book_dialog::open_book_dialog;
pub use content_dialog::open_content_dialog;
pub use content_dialog::open_content_dialog_for_book;
pub use filter_dialog::open_filter_dialog;
// people_dialog e role_dialog eliminati
