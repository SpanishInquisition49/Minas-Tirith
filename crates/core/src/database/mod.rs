use color_eyre::{Result, eyre::Context};
use sqlx::SqlitePool;
use sqlx::migrate::Migrator;

static MIGRATOR: Migrator = sqlx::migrate!();

pub mod collections;
pub mod items;
pub mod query;
pub mod shared;
pub mod tags;

#[derive(Clone, Debug)]
pub struct Archive {
    pub(in crate::database) pool: SqlitePool,
}

impl Archive {
    pub fn from_pool(pool: SqlitePool) -> Self {
        Archive { pool }
    }

    pub async fn migrate(&self) -> Result<()> {
        MIGRATOR.run(&self.pool).await.context("Running Migrations")
    }
}
