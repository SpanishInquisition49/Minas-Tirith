use std::path::{Path, PathBuf};

use color_eyre::eyre::{Context, Result};
use iroh::PublicKey;
use iroh_blobs::{Hash, ticket::BlobTicket};

use crate::peer2peer::node::ShareNode;

impl ShareNode {
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

    pub async fn has_blob(&self, hash: Hash) -> Result<bool> {
        Ok(self.blobs_store.blobs().has(hash).await?)
    }
}
