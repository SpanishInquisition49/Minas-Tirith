use color_eyre::{Result, eyre::Context};
use slug::slugify;
use sqlx::{Row, sqlite::SqliteRow};

use crate::database::Archive;

impl Archive {
    pub(in crate::database) async fn sync_tags(
        txn: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        item_id: i32,
        tags: &[String],
    ) -> Result<()> {
        for tag in tags {
            let row: SqliteRow = sqlx::query("INSERT INTO tags (name, slug) VALUES (?,?) ON CONFLICT (slug) DO UPDATE SET slug = excluded.slug RETURNING id")
                .bind(tag)
                .bind(slugify(tag))
                .fetch_one(&mut **txn)
                .await
                .context("Upserting tag")?;

            let tag_id: i32 = row.get("id");
            sqlx::query(
                "INSERT INTO item_tags (item_id, tag_id) VALUES (?, ?) ON CONFLICT DO NOTHING",
            )
            .bind(item_id)
            .bind(tag_id)
            .execute(&mut **txn)
            .await
            .context("Linking item-tag")?;
        }
        Ok(())
    }
}
