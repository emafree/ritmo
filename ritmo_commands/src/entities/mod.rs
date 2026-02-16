//! Entity management commands
//!
//! This module contains commands for entity operations (tags, publishers, series, people, genres).

// Tag commands
mod list_tags;
mod create_tag;
mod update_tag;
mod delete_tag;

// Publisher commands
mod list_publishers;
mod create_publisher;
mod update_publisher;
mod delete_publisher;

// Series commands
mod list_series;
mod create_series;
mod update_series;
mod delete_series;

// People commands
mod list_people;
mod create_person;
mod update_person;
mod delete_person;

// Role commands
mod list_roles;
mod create_role;
mod update_role;
mod delete_role;

// Genre commands
mod list_genres;
mod create_genre;
mod update_genre;
mod delete_genre;

// Tag exports
pub use list_tags::{ListTagsCommand, ListTagsInput};
pub use create_tag::{CreateTagCommand, CreateTagInput};
pub use update_tag::{UpdateTagCommand, UpdateTagInput};
pub use delete_tag::{DeleteTagCommand, DeleteTagInput};

// Publisher exports
pub use list_publishers::{ListPublishersCommand, ListPublishersInput};
pub use create_publisher::{CreatePublisherCommand, CreatePublisherInput};
pub use update_publisher::{UpdatePublisherCommand, UpdatePublisherInput};
pub use delete_publisher::{DeletePublisherCommand, DeletePublisherInput};

// Series exports
pub use list_series::{ListSeriesCommand, ListSeriesInput};
pub use create_series::{CreateSeriesCommand, CreateSeriesInput};
pub use update_series::{UpdateSeriesCommand, UpdateSeriesInput};
pub use delete_series::{DeleteSeriesCommand, DeleteSeriesInput};

// People exports
pub use list_people::{ListPeopleCommand, ListPeopleInput};
pub use create_person::{CreatePersonCommand, CreatePersonInput};
pub use update_person::{UpdatePersonCommand, UpdatePersonInput};
pub use delete_person::{DeletePersonCommand, DeletePersonInput};

// Role exports
pub use list_roles::{ListRolesCommand, ListRolesInput};
pub use create_role::{CreateRoleCommand, CreateRoleInput};
pub use update_role::{UpdateRoleCommand, UpdateRoleInput};
pub use delete_role::{DeleteRoleCommand, DeleteRoleInput};

// Genre exports
pub use list_genres::{ListGenresCommand, ListGenresInput};
pub use create_genre::{CreateGenreCommand, CreateGenreInput};
pub use update_genre::{UpdateGenreCommand, UpdateGenreInput};
pub use delete_genre::{DeleteGenreCommand, DeleteGenreInput};
