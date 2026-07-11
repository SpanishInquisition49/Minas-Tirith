use std::path::Path;

use color_eyre::eyre::{Context, eyre};
use slug::slugify;
use sqlx::Row;
use sqlx::migrate::Migrator;
use sqlx::{SqlitePool, sqlite::SqliteRow};

use crate::{metadata::common_metadata::ItemMetadata, schema::item::DatabaseItem};

static MIGRATOR: Migrator = sqlx::migrate!();

#[derive(Clone, Debug)]
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
    const GET_AUTHORS_FOR_ITEM: &str = "SELECT a.* FROM authors AS a INNER JOIN item_authors AS ia ON a.id = ia.author_id WHERE item_id = ? ORDER BY ia.author_order";
    const GET_TAGS_FOR_ITEM: &str = "SELECT t.* FROM tags AS t INNER JOIN item_tags AS it ON t.id = it.tag_id WHERE item_id = ? ORDER BY t.slug";
    const ADD_ITEM: &str = "
INSERT INTO items (title, description, type, doi, isbn, publication_date, slug, cover_image_url, path, container)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT (slug) DO UPDATE SET slug = excluded.slug, container = COALESCE(excluded.container, container) RETURNING id
";
    const SET_ITEM_DESCRIPTION: &str = "UPDATE items SET description = ? WHERE id = ?";
    const ADD_AUTHOR: &str = "INSERT INTO authors (name, slug, given_name, family_name) VALUES (?,?,?,?) ON CONFLICT DO UPDATE SET slug = excluded.slug, given_name = excluded.given_name, family_name = excluded.given_name RETURNING id";
    const ADD_ITEM_AUTHOR: &str = "INSERT INTO item_authors (item_id, author_id, author_order) VALUES (?, ?, ?) ON CONFLICT DO NOTHING";
    const SET_COVER_IMAGE_URL: &str = "UPDATE items SET cover_image_url = ? WHERE id = ?;";
    const ADD_TAG: &str = "INSERT INTO tags (name, slug) VALUES (?,?) ON CONFLICT DO UPDATE SET slug = excluded.slug RETURNING id";
    const ADD_ITEM_TAG: &str =
        "INSERT INTO item_tags (item_id, tag_id) VALUES (?, ?) ON CONFLICT DO NOTHING";
    const CLEAR_ITEM_TAGS: &str = "DELETE FROM item_tags WHERE item_id = ?";
    const UPDATE_ITEM_METADATA: &str = "
UPDATE items
SET title = ?, description = ?, type = ?, doi = ?, isbn = ?, publication_date = ?, slug = ?, cover_image_url = ?, container = ?
WHERE id = ?
";

    pub async fn get_all_items(&self) -> color_eyre::Result<Vec<DatabaseItem>> {
        // NOTE: STEP 1: fetch all items
        let mut items: Vec<DatabaseItem> = sqlx::query_as(Archive::GET_ALL_ITEMS)
            .fetch_all(&self.pool)
            .await
            .context("Fetching Items")?;

        // NOTE: STEP 2: fetch all authors and tags for each item
        for item in &mut items {
            let authors = sqlx::query_as(Archive::GET_AUTHORS_FOR_ITEM)
                .bind(item.id)
                .fetch_all(&self.pool)
                .await
                .context("Fetching authors for item")?;

            let tags = sqlx::query_as(Archive::GET_TAGS_FOR_ITEM)
                .bind(item.id)
                .fetch_all(&self.pool)
                .await
                .context("Fetching tags for item")?;
            item.authors = authors;
            item.tags = tags
        }
        Ok(items)
    }

    pub async fn set_cover_image_url(
        &self,
        item_id: i32,
        cover_url: &str,
    ) -> color_eyre::Result<()> {
        sqlx::query(Archive::SET_COVER_IMAGE_URL)
            .bind(cover_url)
            .bind(item_id)
            .execute(&self.pool)
            .await
            .context(format!("Update cover url for item {item_id}"))?;
        Ok(())
    }

    async fn sync_tags(
        txn: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        item_id: i32,
        tags: &[String],
    ) -> color_eyre::Result<()> {
        for tag in tags {
            let row: SqliteRow = sqlx::query(Archive::ADD_TAG)
                .bind(tag)
                .bind(slugify(tag))
                .fetch_one(&mut **txn)
                .await
                .context("Upserting tag")?;

            let tag_id: i32 = row.get("id");
            sqlx::query(Archive::ADD_ITEM_TAG)
                .bind(item_id)
                .bind(tag_id)
                .execute(&mut **txn)
                .await
                .context("Linking item-tag")?;
        }
        Ok(())
    }

    pub async fn save_item_from_form<T: ItemMetadata + ?Sized>(
        &self,
        form: &T,
        item_path: &Path,
    ) -> color_eyre::Result<()> {
        let mut txn = self.pool.begin().await.context("Begin item insertion")?;

        let result: Result<SqliteRow, sqlx::Error> = sqlx::query(Archive::ADD_ITEM)
            .bind(form.title())
            .bind(form.description())
            .bind(form.item_type().to_string())
            .bind(form.doi())
            .bind(form.isbn())
            .bind(form.publication_date())
            .bind(form.slug())
            .bind(form.cover_image_url())
            .bind(item_path.to_string_lossy())
            .bind(form.container())
            .fetch_one(&mut *txn)
            .await;

        let item_id: i32 = match result {
            Ok(row) => row.get("id"),
            Err(e) => {
                txn.rollback().await.ok();
                return Err(eyre!("Failed to create Item: {e}"));
            }
        };

        for (index, author) in form.authors_structured().iter().enumerate() {
            let row: SqliteRow = sqlx::query(Archive::ADD_AUTHOR)
                .bind(&author.full_name)
                .bind(slugify(&author.full_name))
                .bind(&author.given_name)
                .bind(&author.family_name)
                .fetch_one(&mut *txn)
                .await
                .context("Upserting author")?;
            let author_id: i32 = row.get("id");
            sqlx::query(Archive::ADD_ITEM_AUTHOR)
                .bind(item_id)
                .bind(author_id)
                .bind(index as i32)
                .execute(&mut *txn)
                .await
                .context("Linking item-author")?;
        }

        Self::sync_tags(&mut txn, item_id, &form.tags()).await?;

        txn.commit().await.context("Commit item insertion")?;

        Ok(())
    }

    pub async fn update_item_from_form<T: ItemMetadata + ?Sized>(
        &self,
        item_id: i32,
        form: &T,
    ) -> color_eyre::Result<()> {
        let mut txn = self.pool.begin().await.context("Begin item update")?;

        let result = sqlx::query(Archive::UPDATE_ITEM_METADATA)
            .bind(form.title())
            .bind(form.description())
            .bind(form.item_type().to_string())
            .bind(form.doi())
            .bind(form.isbn())
            .bind(form.publication_date())
            .bind(form.slug())
            .bind(form.cover_image_url())
            .bind(form.container())
            .bind(item_id)
            .execute(&mut *txn)
            .await
            .context("Updating item metadata")?;

        if result.rows_affected() == 0 {
            txn.rollback().await.ok();
            return Err(eyre!("No item with id {item_id} found to update"));
        }

        sqlx::query(Archive::CLEAR_ITEM_TAGS)
            .bind(item_id)
            .execute(&mut *txn)
            .await
            .context("Clearing item tags")?;

        Self::sync_tags(&mut txn, item_id, &form.tags()).await?;

        txn.commit().await.context("Commit item update")?;
        Ok(())
    }

    pub async fn set_item_description(
        &self,
        item_id: i32,
        description: &str,
    ) -> color_eyre::Result<()> {
        sqlx::query(Self::SET_ITEM_DESCRIPTION)
            .bind(description)
            .bind(item_id)
            .execute(&self.pool)
            .await
            .with_context(|| format!("Update description for item: {item_id}"))?;
        Ok(())
    }
}
