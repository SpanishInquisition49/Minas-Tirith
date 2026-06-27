use color_eyre::eyre::{Context, eyre};
use sqlx::Row;
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::{SqlitePool, sqlite::SqliteRow};

use crate::{metadata::common_metadata::ItemMetadata, schema::item::DatabaseItem};

static MIGRATOR: Migrator = sqlx::migrate!();

pub struct Archive {
    pool: SqlitePool,
}

impl Archive {
    pub fn from_pool(pool: SqlitePool) -> Self {
        Archive { pool }
    }

    pub async fn migrate(&self) -> color_eyre::Result<()> {
        MIGRATOR.run(&self.pool).await.context("Running Migrations")
    }

    const GET_ALL_ITEMS: &str = "SELECT * FROM items";
    pub async fn get_all_items(&self) -> color_eyre::Result<Vec<DatabaseItem>> {
        sqlx::query_as(Archive::GET_ALL_ITEMS)
            .fetch_all(&self.pool)
            .await
            .context("Fetching Items")
    }

    const ADD_ITEM: &str = "
INSERT INTO items (title, description, type, doi, isbn, publication_date, slug, cover_image_url)
VALUES (?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT (slug) DO UPDATE SET slug = excluded.slug RETURNING id
";
    const ADD_AUTHOR: &str =
        "INSERT INTO authors (name, slug) VALUES (?,?) ON CONFLICT DO NOTHING RETURNING id";
    const ADD_ITEM_AUTHOR: &str = "INSERT INTO item_authors (item_id, author_id, author_order) VALUES (?, ?, ?) ON CONFLICT DO NOTHING";
    pub async fn add_item<T: ItemMetadata + Sized>(&self, item: &T) -> color_eyre::Result<()> {
        let mut txn = self
            .pool
            .begin()
            .await
            .context("Begin Item insertion Transaction")?;

        // NOTE: STEP 1: Create the item
        let result: Result<SqliteRow, sqlx::Error> = sqlx::query(Archive::ADD_ITEM)
            .bind(item.title())
            .bind(item.description())
            .bind(item.item_type().to_string())
            .bind(item.doi())
            .bind(item.isbn())
            .bind(item.publication_date())
            .bind(item.slug())
            .bind(item.cover_image_url())
            .fetch_one(&mut *txn)
            .await;
        if let Err(e) = result {
            txn.rollback()
                .await
                .context("Rollback Item insertion Transaction")?;
            return Err(eyre!("Failed to create Item: {e}"));
        }
        let item_id: i32 = result.unwrap().get("id");

        // NOTE: STEP 2: Create the authors
        let iter = item.authors().into_iter();
        for (index, author) in iter.enumerate() {
            let result: Result<SqliteRow, sqlx::Error> = sqlx::query(Archive::ADD_AUTHOR)
                .bind(&author)
                .bind(slug::slugify(&author))
                .fetch_one(&mut *txn)
                .await;
            if let Err(e) = result {
                txn.rollback()
                    .await
                    .context("Rollback Author insertion Transaction")?;
                return Err(eyre!("Failed to create Author: {e}"));
            }
            let author_id: i32 = result.unwrap().get("id");
            // NOTE: STEP 3: Create the n2n binding
            let result: Result<SqliteQueryResult, sqlx::Error> =
                sqlx::query(Archive::ADD_ITEM_AUTHOR)
                    .bind(item_id)
                    .bind(author_id)
                    .bind(index as i32)
                    .execute(&mut *txn)
                    .await;
            if let Err(e) = result {
                txn.rollback()
                    .await
                    .context("Rollback Item-Author insertion Transaction")?;
                return Err(eyre!("Failed to create Item-Author: {e}"));
            }
        }

        if let Err(e) = txn.commit().await {
            return Err(eyre!("Failed to commit Transaction: {e}"));
        }
        Ok(())
    }
}
