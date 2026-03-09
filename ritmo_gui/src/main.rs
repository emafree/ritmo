slint::include_modules!();
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

    win.run()?;
    Ok(())
}
