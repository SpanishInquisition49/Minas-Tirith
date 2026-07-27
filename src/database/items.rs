use std::path::Path;

use color_eyre::{
    Result,
    eyre::{Context, bail},
};
use slug::slugify;
use sqlx::{Row, sqlite::SqliteRow};

use crate::{
    database::Archive,
    metadata::common_metadata::ItemMetadata,
    schema::item::{DatabaseItem, RawItemRow},
};

impl Archive {
    // NOTE: Items Queries

    pub async fn get_all_items(&self) -> Result<Vec<DatabaseItem>> {
        let items: Vec<RawItemRow> = sqlx::query_as(
            "
SELECT i.*, c.collections, a.authors, t.tags
FROM items AS i
LEFT JOIN view_collections_aggregated AS c ON c.item_id = i.id
LEFT JOIN view_authors_aggregated AS a ON a.item_id = i.id
LEFT JOIN view_tags_aggregated AS t ON t.item_id = i.id
ORDER BY i.title",
        )
        .fetch_all(&self.pool)
        .await
        .context("Fetching Items")?;
        Ok(items.into_iter().map(DatabaseItem::from).collect())
    }

    pub async fn set_cover_image_url(&self, item_id: i32, cover_url: &str) -> Result<()> {
        sqlx::query("UPDATE items SET cover_image_url = ? WHERE id = ?;")
            .bind(cover_url)
            .bind(item_id)
            .execute(&self.pool)
            .await
            .context(format!("Update cover url for item {item_id}"))?;
        Ok(())
    }

    pub async fn save_item_from_form<T: ItemMetadata + ?Sized>(
        &self,
        form: &T,
        item_path: &Path,
    ) -> Result<()> {
        let mut txn = self.pool.begin().await.context("Begin item insertion")?;

        let result: Result<SqliteRow, sqlx::Error> = sqlx::query(
            "
INSERT INTO items (
    title,
    description,
    type,
    doi,
    isbn,
    publication_date,
    slug,
    cover_image_url,
    path,
    container
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) 
RETURNING id
",
        )
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
                bail!("Failed to create Item: {e}")
            }
        };

        for (index, author) in form.authors_structured().iter().enumerate() {
            let row: SqliteRow = sqlx::query(
                "
INSERT INTO authors (name, slug, given_name, family_name)
VALUES (?,?,?,?)
ON CONFLICT (slug) DO UPDATE SET
    slug = excluded.slug,
    given_name = excluded.given_name,
    family_name = excluded.family_name
RETURNING id",
            )
            .bind(&author.full_name)
            .bind(slugify(&author.full_name))
            .bind(&author.given_name)
            .bind(&author.family_name)
            .fetch_one(&mut *txn)
            .await
            .context("Upserting author")?;
            let author_id: i32 = row.get("id");
            sqlx::query(
                "
INSERT INTO item_authors (item_id, author_id, author_order)
VALUES (?, ?, ?) ON CONFLICT DO NOTHING",
            )
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
    ) -> Result<()> {
        let mut txn = self.pool.begin().await.context("Begin item update")?;

        let result = sqlx::query(
            "
UPDATE items
SET
    title = ?,
    description = ?,
    type = ?,
    doi = ?,
    isbn = ?,
    publication_date = ?,
    slug = ?,
    cover_image_url = ?,
    container = ?
WHERE id = ?
",
        )
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
            bail!("No item with id {item_id} found to update");
        }

        sqlx::query("DELETE FROM item_tags WHERE item_id = ?")
            .bind(item_id)
            .execute(&mut *txn)
            .await
            .context("Clearing item tags")?;

        Self::sync_tags(&mut txn, item_id, &form.tags()).await?;

        txn.commit().await.context("Commit item update")?;
        Ok(())
    }

    pub async fn set_item_description(&self, item_id: i32, description: &str) -> Result<()> {
        sqlx::query("UPDATE items SET description = ? WHERE id = ?")
            .bind(description)
            .bind(item_id)
            .execute(&self.pool)
            .await
            .with_context(|| format!("Update description for item: {item_id}"))?;
        Ok(())
    }
}
