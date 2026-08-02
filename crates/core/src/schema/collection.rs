use serde::Deserialize;

use crate::traits::Colorable;

#[derive(Clone, sqlx::FromRow, Deserialize, Debug)]
pub struct Collection {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

impl Colorable for Collection {}

impl Collection {
    /// Get the All collection
    #[must_use]
    pub fn trivial_collection() -> Self {
        Self {
            id: -1,
            name: "All".to_string(),
            slug: "all".to_string(),
        }
    }

    /// Check if the given id is from the trivial collection
    #[must_use]
    pub fn is_trivial_collection(id: i32) -> bool {
        id == -1
    }
}
