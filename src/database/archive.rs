use anyhow::{Context, Ok, Result as AnyhowResult};
use sqlx::SqlitePool;
use sqlx::migrate::Migrator;

use crate::schema::item::{DatabaseItem, Item};

static MIGRATOR: Migrator = sqlx::migrate!();

pub struct Archive {
    pool: SqlitePool,
}

impl Archive {
    pub fn from_pool(pool: SqlitePool) -> Self {
        Archive { pool }
    }

    pub async fn migrate(&self) -> AnyhowResult<()> {
        MIGRATOR.run(&self.pool).await.context("Running Migrations")
    }

    const GET_ALL_ITEMS: &str = "SELECT * FROM items";
    pub async fn get_all_items(&self) -> AnyhowResult<Vec<DatabaseItem>> {
        sqlx::query_as(Archive::GET_ALL_ITEMS)
            .fetch_all(&self.pool)
            .await
            .context("Fetching Items")
    }

    const ADD_ITEM: &str = "
INSERT INTO items (title, description, type, doi, isbn, publication_date, slug, cover_image_url)
VALUES (?, ?, ?, ?, ?, ?, ?, ?)
";
    pub async fn add_item(&self, item: &Item) -> AnyhowResult<()> {
        sqlx::query(Archive::ADD_ITEM)
            .bind(&item.title)
            .bind(&item.description)
            .bind(&item.r#type)
            .bind(&item.doi)
            .bind(&item.isbn)
            .bind(&item.publication_date)
            .bind(&item.slug)
            .bind(&item.cover_image_url)
            .execute(&self.pool)
            .await
            .context("Creating Item in archive")?;
        Ok(())
    }
}
