use color_eyre::eyre::{Context, Result};

use crate::{
    database::Archive,
    metadata::shared_library::{LibraryMode, LibrarySubscription, SharedLibrary},
};

impl Archive {
    /// Create a shared library for `collection_id`, publishing it under
    /// `namespace_id` with the given `name` and optional `description`.
    /// # Errors
    /// Returns an error if the insert query fails.
    pub async fn create_shared_library(
        &self,
        collection_id: i32,
        namespace_id: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<SharedLibrary> {
        sqlx::query_as(
            "
INSERT INTO shared_libraries (collection_id, namespace_id, name, description, mode)
VALUES (?,?,?,?,?) RETURNING *
",
        )
        .bind(collection_id)
        .bind(namespace_id)
        .bind(name)
        .bind(description)
        .bind(LibraryMode::Personal.to_string())
        .fetch_one(&self.pool)
        .await
        .context("Creating shared library")
    }

    /// Fetch the shared library associated with `collection_id`, if any.
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn get_shared_library_for_collection(
        &self,
        collection_id: i32,
    ) -> Result<Option<SharedLibrary>> {
        sqlx::query_as(
            "
SELECT * FROM shared_libraries WHERE collection_id = ?
",
        )
        .bind(collection_id)
        .fetch_optional(&self.pool)
        .await
        .context("Fetchaing shared library for collection")
    }

    /// Fetch every shared library, ordered alphabetically by name.
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn get_all_shared_libraries(&self) -> Result<Vec<SharedLibrary>> {
        sqlx::query_as("SELECT * FROM shared_libraries ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .context("Fetching shared libraries")
    }

    /// Delete the shared library identified by `id`.
    /// # Errors
    /// Returns an error if the delete query fails.
    pub async fn delete_shared_library(&self, id: i32) -> Result<()> {
        sqlx::query("DELETE FROM shared_libraries WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Dealeting shared library")?;
        Ok(())
    }

    /// Subscribe to the shared library published under `namespace_id` by
    /// `owner_node_id`, storing it locally under `nickname`. Updates the
    /// nickname if a subscription for that namespace already exists.
    /// # Errors
    /// Returns an error if the insert/update query fails.
    pub async fn create_subscription(
        &self,
        namespace_id: &str,
        owner_node_id: &str,
        nickname: &str,
    ) -> Result<LibrarySubscription> {
        sqlx::query_as(
            "
INSERT INTO library_subscriptions (namespace_id, owner_node_id, nickname)
VALUES (?,?,?)
ON CONFLICT (namespace_id) DO UPDATE SET nickname = excluded.nickname
RETURNING *
",
        )
        .bind(namespace_id)
        .bind(owner_node_id)
        .bind(nickname)
        .fetch_one(&self.pool)
        .await
        .context("Creating library subscription")
    }

    /// Fetch every library subscription, ordered alphabetically by nickname.
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn get_all_subscriptions(&self) -> Result<Vec<LibrarySubscription>> {
        sqlx::query_as("SELECT * FROM library_subscriptions ORDER BY nickname")
            .fetch_all(&self.pool)
            .await
            .context("Fetching library subscriptions")
    }

    /// Update the `last_synced_at` timestamp for the subscription identified
    /// by `namespace_id` to now.
    /// # Errors
    /// Returns an error if the update query fails.
    pub async fn touch_subscription_sync(&self, namespace_id: &str) -> Result<()> {
        sqlx::query("UPDATE library_subscriptions SET last_synced_at = datetime('now') WHERE namespace_id = ?")
            .bind(namespace_id)
            .execute(&self.pool)
            .await
            .context("Updating subscription last_synced_at")?;
        Ok(())
    }

    /// Delete the subscription identified by `namespace_id`.
    /// # Errors
    /// Returns an error if the delete query fails.
    pub async fn delete_subscription(&self, namespace_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM library_subscriptions WHERE namespace_id = ?")
            .bind(namespace_id)
            .execute(&self.pool)
            .await
            .context("Deleting library subscription")?;
        Ok(())
    }
}
