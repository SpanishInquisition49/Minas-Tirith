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
    /// Construct an [`Archive`] backed by an existing [`SqlitePool`].
    #[must_use]
    pub fn from_pool(pool: SqlitePool) -> Self {
        Archive { pool }
    }

    /// Run the migrations to align the database to the current standard used by the core.
    /// # Errors
    /// the function fail there is a failure while running migrations.
    pub async fn migrate(&self) -> Result<()> {
        MIGRATOR.run(&self.pool).await.context("Running Migrations")
    }
}
