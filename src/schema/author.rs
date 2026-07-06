use core::fmt;

use chrono::{DateTime, Utc};

#[derive(Clone, sqlx::FromRow, Debug)]
pub struct Author {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl fmt::Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = format!("Name: {}\n", self.name);
        if let Some(bio) = &self.bio {
            res.push_str(&format!("Bio:\n{}", bio));
        }
        write!(f, "{res}")
    }
}
