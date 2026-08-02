use std::path::{Path, PathBuf};

use color_eyre::eyre::{Context, Result};
use iroh::PublicKey;
use iroh_blobs::{Hash, ticket::BlobTicket};

use crate::peer2peer::node::ShareNode;

impl ShareNode {
    /// Add `file_path` to the local blob store and produce a ticket other
    /// peers can use to download it.
    /// # Errors
    /// Returns an error if the file cannot be added to the blob store.
    pub async fn share_file(&self, file_path: &Path) -> Result<BlobTicket> {
        let tag = self
            .blobs_store
            .blobs()
            .add_path(file_path)
            .await
            .with_context(|| format!("Adding {} to blob store", file_path.display()))?;

        let addr = self.endpoint.addr();
        Ok(BlobTicket::new(addr, tag.hash, tag.format))
    }

    /// Download the blob referenced by `ticket` from its owning peer and
    /// export it to `dest_path`.
    /// # Errors
    /// Returns an error if the download or the export to `dest_path` fails.
    pub async fn download_blob(&self, ticket: &BlobTicket, dest_path: &Path) -> Result<PathBuf> {
        let downloader = self.blobs_store.downloader(&self.endpoint);

        downloader
            .download(ticket.hash(), Some(ticket.addr().id))
            .await
            .context("Downloading blob from peer")?;

        self.blobs_store
            .blobs()
            .export(ticket.hash(), dest_path)
            .await
            .context("Exporting downloaded blob to destination path")?;

        Ok(dest_path.to_path_buf())
    }

    /// Download the blob identified by `hash` from `owner` and export it to
    /// `dest_path`.
    /// # Errors
    /// Returns an error if the download or the export to `dest_path` fails.
    pub async fn download_blob_by_hash(
        &self,
        hash: Hash,
        owner: PublicKey,
        dest_path: &Path,
    ) -> Result<PathBuf> {
        let downloader = self.blobs_store.downloader(&self.endpoint);
        downloader
            .download(hash, Some(owner))
            .await
            .context("Downloading blob by hash")?;

        self.blobs_store
            .blobs()
            .export(hash, dest_path)
            .await
            .context("Exporting downloaded blob to destination path")?;

        Ok(dest_path.to_path_buf())
    }

    /// Check whether the blob identified by `hash` is present in the local
    /// blob store.
    /// # Errors
    /// Returns an error if the local blob store cannot be queried.
    pub async fn has_blob(&self, hash: Hash) -> Result<bool> {
        Ok(self.blobs_store.blobs().has(hash).await?)
    }
}
