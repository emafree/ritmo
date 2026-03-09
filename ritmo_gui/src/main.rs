slint::include_modules!();
mod dialogs;
mod i18n;

fn main() -> anyhow::Result<()> {
    let win = MainWindow::new()?;

    let lang = std::env::var("LANG")
        .unwrap_or_default()
        .split('_')
        .next()
        .unwrap_or("en")
        .to_string();

    i18n::apply_translations(&win, &lang);

    // Open BookDialog when the "+" button is clicked
    win.on_request_add_book(move || {
        let _ = dialogs::open_book_dialog();
    });

    // Open PeopleDialog when the add-person action is triggered
    win.on_request_add_person(move || {
        let _ = dialogs::open_people_dialog();
    });

    // Open FilterDialog when the filter action is triggered
    win.on_request_filter(move || {
        let _ = dialogs::open_filter_dialog();
    });

    win.run()?;
    Ok(())
}
