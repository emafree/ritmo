use crate::traits::{FromDto, FromModel};
use ritmo_core::dto::BookDto;
use ritmo_db::models::Book;

impl FromDto<BookDto> for Book {
    fn from_dto(dto: &BookDto) -> Self {
        Book {
            id: None,
            name: dto.name.clone(),
            original_title: dto.original_title.clone(),
            publisher_id: dto.publisher_id,
            format_id: dto.format_id,
            series_id: dto.series_id,
            series_index: dto.series_index,
            publication_date: dto.publication_date,
            last_modified_date: chrono::Utc::now().timestamp(),
            isbn: dto.isbn.clone(),
            pages: None,
            notes: dto.notes.clone(),
            has_cover: if dto.has_cover { 1 } else { 0 },
            has_paper: if dto.has_paper { 1 } else { 0 },
            file_link: dto.file_link.clone(),
            file_size: dto.file_size,
            file_hash: dto.file_hash.clone(),
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}

impl FromModel<Book> for BookDto {
    fn from_model(book: &Book) -> Self {
        BookDto {
            name: book.name.clone(),
            original_title: book.original_title.clone(),
            publisher_name: "".to_string(),
            publisher_id: book.publisher_id,
            publisher_is_new: false,
            format_name: "".to_string(),
            format_id: book.format_id,
            format_is_new: false,
            series_name: "".to_string(),
            series_id: book.series_id,
            series_is_new: false,
            series_index: book.series_index,
            publication_date: book.publication_date,
            acquisition_date: None,
            isbn: book.isbn.clone(),
            notes: book.notes.clone(),
            has_cover: book.has_cover != 0,
            has_paper: book.has_paper != 0,
            file_link: book.file_link.clone(),
            file_size: book.file_size,
            file_hash: book.file_hash.clone(),
            tags: vec![],
            contents: vec![],
            people: vec![],
        }
    }
}
