use color_eyre::eyre::{Context, Result};

use crate::{
    database::archive::Archive,
    metadata::shared_library::{LibraryMode, LibrarySubscription, SharedLibrary},
};

impl Archive {
    // NOTE: Shared Libraries Queries
    const CREATE_SHARED_LIBRARY: &str = "
INSERT INTO shared_libraries (collection_id, namespace_id, name, description, mode)
VALUES (?,?,?,?,?) RETURNING *
";
    const GET_SHARED_LIBRARY_BY_COLLECTION: &str = "
SELECT * FROM shared_libraries WHERE collection_id = ?
";
    const GET_ALL_SHARED_LIBRARIES: &str = "SELECT * FROM shared_libraries ORDER BY name";
    const DELETE_SHARED_LIBRARY: &str = "DELETE FROM shared_libraries WHERE id = ?";
    // NOTE: Library Subscription Queries
    const CREATE_SUBSCRIPTION: &str = "
INSERT INTO library_subscriptions (namespace_id, owner_node_id, nickname)
VALUES (?,?,?)
ON CONFLICT (namespace_id) DO UPDATE SET nickname = excluded.nickname
RETURNING *
";
    const GET_ALL_SUBSCRIPTIONS: &str = "SELECT * FROM library_subscriptions ORDER BY nickname";
    const TOUCH_SUBSCRIPTION_SYNC: &str =
        "UPDATE library_subscriptions SET last_synced_at = datetime('now') WHERE namespace_id = ?";
    const DELETE_SUBSCRIPTION: &str = "DELETE FROM library_subscriptions WHERE namespace_id = ?";

    pub async fn create_shared_library(
        &self,
        collection_id: i32,
        namespace_id: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<SharedLibrary> {
        sqlx::query_as(Archive::CREATE_SHARED_LIBRARY)
            .bind(collection_id)
            .bind(namespace_id)
            .bind(name)
            .bind(description)
            .bind(LibraryMode::Personal.to_string())
            .fetch_one(&self.pool)
            .await
            .context("Creating shared library")
    }

    pub async fn get_shared_library_for_collection(
        &self,
        collection_id: i32,
    ) -> Result<Option<SharedLibrary>> {
        sqlx::query_as(Archive::GET_SHARED_LIBRARY_BY_COLLECTION)
            .bind(collection_id)
            .fetch_optional(&self.pool)
            .await
            .context("Fetchaing shared library for collection")
    }

    pub async fn get_all_shared_libraries(&self) -> Result<Vec<SharedLibrary>> {
        sqlx::query_as(Archive::GET_ALL_SHARED_LIBRARIES)
            .fetch_all(&self.pool)
            .await
            .context("Fetching shared libraries")
    }

    pub async fn delete_shared_library(&self, id: i32) -> Result<()> {
        sqlx::query(Archive::DELETE_SHARED_LIBRARY)
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Dealeting shared library")?;
        Ok(())
    }

    pub async fn create_subscription(
        &self,
        namespace_id: &str,
        owner_node_id: &str,
        nickname: &str,
    ) -> Result<LibrarySubscription> {
        sqlx::query_as(Archive::CREATE_SUBSCRIPTION)
            .bind(namespace_id)
            .bind(owner_node_id)
            .bind(nickname)
            .fetch_one(&self.pool)
            .await
            .context("Creating library subscription")
    }

    pub async fn get_all_subscriptions(&self) -> Result<Vec<LibrarySubscription>> {
        sqlx::query_as(Archive::GET_ALL_SUBSCRIPTIONS)
            .fetch_all(&self.pool)
            .await
            .context("Fetching library subscriptions")
    }

    pub async fn touch_subscription_sync(&self, namespace_id: &str) -> Result<()> {
        sqlx::query(Archive::TOUCH_SUBSCRIPTION_SYNC)
            .bind(namespace_id)
            .execute(&self.pool)
            .await
            .context("Updating subscription last_synced_at");
        Ok(())
    }

    pub async fn delete_subscription(&self, namespace_id: &str) -> Result<()> {
        sqlx::query(Archive::DELETE_SUBSCRIPTION)
            .bind(namespace_id)
            .execute(&self.pool)
            .await
            .context("Deleting library subscription");
        Ok(())
    }
}
