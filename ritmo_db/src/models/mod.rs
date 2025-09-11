/// I files che descrivono tabelle cross hanno un nome che inizia per x_
///
/// Per tutte le strutture di un FullBook adotto una semplice convenzione: se al momento di memorizzare
/// esiste già un indice valido, ritengo che si tratti di un elemento già esistente.
/// Non faccio la persistence, ma considero l'indice valido per le strutture cross.
/// La parte di ML si deve occupare di salvare nella struttura l'indice valido.
///
/// La sequenza quindi è:
/// User -> DTO data -> ML -> Models data
pub mod aliases;
pub mod books;
pub mod contents;
pub mod formats;
pub mod languages;
pub mod people;
pub mod publishers;
pub mod roles;
pub mod series;
pub mod tags;
pub mod types;
pub mod x_books_contents;
pub mod x_books_people_roles;
pub mod x_books_tags;
pub mod x_contents_languages;
pub mod x_contents_people_roles;
pub mod x_contents_tags;

pub use self::aliases::*;
pub use self::books::*;
pub use self::contents::*;
pub use self::formats::*;
pub use self::languages::*;
pub use self::people::*;
pub use self::publishers::*;
pub use self::roles::*;
pub use self::series::*;
pub use self::tags::*;
pub use self::types::*;
pub use self::x_books_contents::*;
pub use self::x_books_people_roles::*;
pub use self::x_books_tags::*;
pub use self::x_contents_languages::*;
pub use self::x_contents_people_roles::*;
pub use self::x_contents_tags::*;

use ritmo_core::FullDto;

#[derive(Debug, Clone)]
pub struct FullBook {
    pub alias: Alias,
    pub book: Book,
    pub book_content: BookContent,
    pub book_people_roles: BookPersonRole,
    pub books_tags: BookTag,
    pub contents: Content,
    pub contents_languages: ContentLanguage,
    pub contents_people_roles: ContentPersonRole,
    pub contents_tags: ContentTag,
    pub formats: Format,
    pub languages: RunningLanguages,
    pub people: Person,
    pub publishers: Publisher,
    pub roles: Role,
    pub series: Series,
    pub tags: Tag,
    pub types: Type,
}

impl FullBook {
    pub fn from_dto(_dto: &FullDto) {}
}
