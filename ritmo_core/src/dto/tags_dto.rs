use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagDto {
    pub name: String,
    pub is_book_tag: bool,
    pub is_content_tag: bool,
}
