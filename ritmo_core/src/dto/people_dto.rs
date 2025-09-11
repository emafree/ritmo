use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonDto {
    pub person_id: Option<i64>,
    pub person_name: String,
    pub person_role: String,
}
