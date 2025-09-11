use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasDto {
    pub name: String,
    pub alias_normalized: String,
}
