use crate::app::App;
use crate::config::ViewMode;
use crate::events::Message;

/// Thumbnail display size in grid view (width × height in logical pixels)
const THUMB_W: f32 = 120.0;
const THUMB_H: f32 = 160.0;
/// Minimum card width including padding
const CARD_W: f32 = THUMB_W + 16.0;

/// Render books tab – switches between list and grid view based on `app.settings.view_mode`.
pub fn render_books_tab(app: &mut App, ui: &mut egui::Ui) {
    ui.heading("Books");
    ui.separator();

    let books = app.library_state.get_books().to_vec();
    let selected_book_id = app.selected_book_id;

    if books.is_empty() {
        ui.label("No books found. Try adjusting your filters or add books to your library.");
        return;
    }

    match app.settings.view_mode {
        ViewMode::List => render_list_view(app, ui, &books, selected_book_id),
        ViewMode::Grid => render_grid_view(app, ui, &books),
    }
}

/// List view – original scrollable list with detail expansion on selection.
fn render_list_view(
    app: &mut App,
    ui: &mut egui::Ui,
    books: &[ritmo_commands::BookSummary],
    selected_book_id: Option<i64>,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        for book in books {
            let is_selected = selected_book_id == Some(book.id);

            let response = ui.selectable_label(
                is_selected,
                format!("📚 {} {}", book.title, book.authors.join(", ")),
            );

            if response.clicked() {
                app.handle_message(Message::BookSelected(book.id));
            }

            if response.double_clicked() {
                app.handle_message(Message::BookDoubleClicked(book.id));
            }

            // Show additional info when selected
            if is_selected {
                ui.indent("book_details", |ui| {
                    if let Some(ref publisher) = book.publisher {
                        ui.label(format!("Publisher: {}", publisher));
                    }
                    if let Some(year) = book.year {
                        ui.label(format!("Year: {}", year));
                    }
                    if let Some(ref format) = book.format {
                        ui.label(format!("Format: {}", format));
                    }
                    if let Some(size) = book.file_size {
                        ui.label(format!("Size: {} bytes", size));
                    }
                });
            }

            ui.separator();
        }
    });
}

/// Grid view – scrollable grid of thumbnail cards.
/// Each card shows a thumbnail (or a placeholder) plus a caption with title/author.
fn render_grid_view(
    app: &mut App,
    ui: &mut egui::Ui,
    books: &[ritmo_commands::BookSummary],
) {
    let available_width = ui.available_width();
    let columns = ((available_width / CARD_W).floor() as usize).max(1);

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Collect book ids so we can avoid holding an immutable borrow on `app`
        // while also mutably borrowing it for texture loading.
        let book_ids: Vec<i64> = books.iter().map(|b| b.id).collect();

        let ctx = ui.ctx().clone();

        // Pre-load textures (mutable borrow of app, outside the UI grid loop)
        for &id in &book_ids {
            app.get_thumbnail(&ctx, id);
        }

        egui::Grid::new("books_grid")
            .num_columns(columns)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                for (i, book) in books.iter().enumerate() {
                    render_book_card(app, ui, book);
                    if (i + 1) % columns == 0 {
                        ui.end_row();
                    }
                }
                // Close final incomplete row
                if !books.is_empty() && books.len() % columns != 0 {
                    ui.end_row();
                }
            });
    });
}

/// Render a single book card with thumbnail + caption.
fn render_book_card(app: &mut App, ui: &mut egui::Ui, book: &ritmo_commands::BookSummary) {
    let texture_opt = app.thumbnail_cache.get(&book.id).cloned();

    // Build card frame
    let frame = egui::Frame::none()
        .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color))
        .inner_margin(egui::Margin::same(4.0));

    frame.show(ui, |ui| {
        ui.set_max_width(CARD_W);

        // Thumbnail or placeholder
        if let Some(texture) = texture_opt {
            let size = fit_size(texture.size(), THUMB_W, THUMB_H);
            let img = egui::Image::from_texture(egui::load::SizedTexture::new(
                texture.id(),
                egui::vec2(size.0, size.1),
            ));
            let response = ui.add(img.sense(egui::Sense::click()));
            if response.clicked() {
                app.handle_message(Message::BookSelected(book.id));
            }
            if response.double_clicked() {
                app.handle_message(Message::BookDoubleClicked(book.id));
            }
        } else {
            // Placeholder rectangle
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(THUMB_W, THUMB_H),
                egui::Sense::click(),
            );
            let painter = ui.painter();
            painter.rect_filled(rect, 4.0, ui.visuals().extreme_bg_color);
            painter.rect_stroke(
                rect,
                4.0,
                egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "No thumbnail",
                egui::FontId::default(),
                ui.visuals().weak_text_color(),
            );
            if response.clicked() {
                app.handle_message(Message::BookSelected(book.id));
            }
            if response.double_clicked() {
                app.handle_message(Message::BookDoubleClicked(book.id));
            }
        }

        // Caption: title (and author if available)
        ui.separator();
        ui.add(
            egui::Label::new(
                egui::RichText::new(&book.title).small().strong(),
            )
            .wrap(true),
        );
        if !book.authors.is_empty() {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(book.authors.join(", ")).small(),
                )
                .wrap(true),
            );
        }
    });
}

/// Scale (w, h) to fit inside max_w × max_h while preserving aspect ratio.
fn fit_size(original: [usize; 2], max_w: f32, max_h: f32) -> (f32, f32) {
    let (ow, oh) = (original[0] as f32, original[1] as f32);
    if ow == 0.0 || oh == 0.0 {
        return (max_w, max_h);
    }
    let scale = (max_w / ow).min(max_h / oh);
    (ow * scale, oh * scale)
}
