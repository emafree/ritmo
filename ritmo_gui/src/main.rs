slint::include_modules!();

fn main() -> anyhow::Result<()> {
    let win = MainWindow::new()?;
    win.run()?;
    Ok(())
}