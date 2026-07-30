use color_eyre::{Result, eyre::Context};
use slug::slugify;

use crate::{database::Archive, schema::collection::Collection};

impl Archive {
    pub async fn get_all_collections(&self) -> Result<Vec<Collection>> {
        sqlx::query_as("SELECT * FROM collections ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .context("Fetching collections")
    }

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

    pub async fn delete_collection(&self, collection_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM collections WHERE id = ?")
            .bind(collection_id)
            .execute(&self.pool)
            .await
            .context("Deleting collection")?;
        Ok(())
    }

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
