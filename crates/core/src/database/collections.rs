use color_eyre::{Result, eyre::Context};
use slug::slugify;

use crate::{database::Archive, schema::collection::Collection};

impl Archive {
    /// Fetch every collection in the archive, ordered alphabetically by name.
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn get_all_collections(&self) -> Result<Vec<Collection>> {
        sqlx::query_as("SELECT * FROM collections ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .context("Fetching collections")
    }

    /// Create a collection with the given `name`, or update its slug if a
    /// collection with that name already exists.
    /// # Errors
    /// Returns an error if the insert/update query fails.
    pub async fn create_collection(&self, name: &str) -> Result<Collection> {
        sqlx::query_as(
            "
INSERT INTO collections (name, slug)
VALUES (?, ?)
ON CONFLICT (name)
DO UPDATE SET
    slug = excluded.slug,
    name = excluded.name
RETURNING *
",
        )
        .bind(name)
        .bind(slugify(name))
        .fetch_one(&self.pool)
        .await
        .context("Creating collection")
    }

    /// Delete the collection identified by `collection_id`.
    /// # Errors
    /// Returns an error if the delete query fails.
    pub async fn delete_collection(&self, collection_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM collections WHERE id = ?")
            .bind(collection_id)
            .execute(&self.pool)
            .await
            .context("Deleting collection")?;
        Ok(())
    }

    /// Add the item identified by `item_id` to the collection identified by
    /// `collection_id`. No-op if the association already exists.
    /// # Errors
    /// Returns an error if the insert query fails.
    pub async fn add_item_to_collection(&self, item_id: i32, collection_id: i32) -> Result<()> {
        sqlx::query(
            "
INSERT INTO item_collections (item_id, collection_id)
VALUES (?,?) ON CONFLICT DO NOTHING
",
        )
        .bind(item_id)
        .bind(collection_id)
        .execute(&self.pool)
        .await
        .context("Adding item to collection")?;
        Ok(())
    }

    /// Remove the item identified by `item_id` from the collection identified
    /// by `collection_id`.
    /// # Errors
    /// Returns an error if the delete query fails.
    pub async fn remove_item_from_collection(
        &self,
        item_id: i32,
        collection_id: i32,
    ) -> Result<()> {
        sqlx::query(
            "
DELETE FROM item_collections WHERE item_id = ? AND collection_id = ?
",
        )
        .bind(item_id)
        .bind(collection_id)
        .execute(&self.pool)
        .await
        .context("Adding item to collection")?;
        Ok(())
    }
}
