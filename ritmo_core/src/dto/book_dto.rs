use crate::dto::{ContentDto, PersonDto, TagDto};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookDto {
    pub name: String,
    pub original_title: Option<String>,

    // Publisher con campi aggiuntivi
    pub publisher_name: String,
    pub publisher_id: Option<i64>,
    pub publisher_is_new: bool,

    // Format con campi aggiuntivi
    pub format_name: String,
    pub format_id: Option<i64>,
    pub format_is_new: bool,

    // Serie con campi aggiuntivi
    pub series_name: String,
    pub series_id: Option<i64>,
    pub series_is_new: bool,

    pub series_index: Option<i64>,

    pub publication_date: Option<i64>,
    pub acquisition_date: Option<i64>,
    pub isbn: Option<String>,

    pub notes: Option<String>,
    pub has_cover: bool,
    pub has_paper: bool,
    pub file_link: Option<String>,
    pub file_size: Option<i64>,
    pub file_hash: Option<String>,
    pub tags: Vec<TagDto>,
    pub contents: Vec<ContentDto>,
    pub people: Vec<PersonDto>,
}
