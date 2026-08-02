use core::fmt;
use std::fmt::Write;

use serde::Deserialize;

use crate::traits::Colorable;

#[derive(Clone, sqlx::FromRow, Deserialize, Debug)]
pub struct Author {
    pub name: String,
    pub slug: String,
    pub bio: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
}

impl Colorable for Author {}

impl fmt::Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = format!("Name: {}\n", self.name);
        if let Some(bio) = &self.bio {
            let _ = write!(res, "Bio:\n{bio}");
        }
        write!(f, "{res}")
    }
}
