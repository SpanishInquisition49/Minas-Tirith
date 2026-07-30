use core::fmt;
use serde::Deserialize;

#[derive(Clone, sqlx::FromRow, Deserialize, Debug)]
pub struct SharedLibrary {
    pub id: i32,
    pub collection_id: i32,
    pub namespace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub mode: String,
}

#[derive(Clone, sqlx::FromRow, Deserialize, Debug)]
pub struct LibrarySubscription {
    pub id: i32,
    pub namespace_id: String,
    pub owner_node_id: String,
    pub nickname: String,
    pub last_synced_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryMode {
    Personal,
    Group,
}

impl fmt::Display for LibraryMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibraryMode::Personal => write!(f, "personal"),
            LibraryMode::Group => write!(f, "group"),
        }
    }
}
