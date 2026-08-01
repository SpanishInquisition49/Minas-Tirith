use serde::Deserialize;

use crate::traits::Colorable;

#[derive(Clone, Debug, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub slug: String,
}

impl Colorable for Tag {}
