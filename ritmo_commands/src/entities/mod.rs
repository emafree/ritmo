//! Entity management commands
//!
//! This module contains commands for entity operations (tags, publishers, series, people).

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
