use std::{borrow::Cow, str::FromStr};

use chrono::{DateTime, Utc};
use color_eyre::eyre::{Context, Result, eyre};
use iroh::PublicKey;
use iroh_blobs::Hash;
use iroh_docs::{
    AuthorId, DocTicket, NamespaceId,
    api::{
        Doc,
        protocol::{AddrInfoOptions, ShareMode},
    },
    store::Query,
};
use n0_future::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::pin;
use uuid::Uuid;

use crate::{
    metadata::common_metadata::{ItemMetadata, ItemType},
    peer2peer::node::ShareNode,
    schema::item::DatabaseItem,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedItemEntry {
    pub item_id: Uuid,
    pub blob_hash: Hash,
    pub blob_size: u64,
    pub file_name: String,
    pub title: String,
    pub description: Option<String>,
    pub r#type: String,
    pub doi: Option<String>,
    pub isbn: Option<String>,
    pub publication_date: Option<String>,
    pub container: Option<String>,
    pub authors: Vec<String>,
    pub tags: Vec<String>,
    pub cover_image_url: Option<String>,
    pub owner: PublicKey,
    pub published_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SharedItemEntry {
    /// Build a [`SharedItemEntry`] describing `item` as published to a
    /// shared library under `item_id`, referencing the blob identified by
    /// `blob_hash`/`blob_size` and owned by `owner`.
    #[must_use]
    pub fn new(
        item: &DatabaseItem,
        item_id: Uuid,
        file_name: String,
        blob_size: u64,
        blob_hash: Hash,
        owner: PublicKey,
    ) -> Self {
        Self {
            item_id,
            blob_hash,
            blob_size,
            file_name,
            title: item.fields.title.clone(),
            description: item.fields.description.clone(),
            r#type: item.fields.r#type.clone(),
            doi: item.fields.doi.clone(),
            isbn: item.fields.isbn.clone(),
            publication_date: item.fields.publication_date.clone(),
            container: item.fields.container.clone(),
            authors: item.authors.iter().map(|a| a.name.clone()).collect(),
            tags: item.tags.iter().map(|t| t.name.clone()).collect(),
            cover_image_url: item.fields.cover_image_url.clone(),
            owner,
            published_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl ShareNode {
    /// Create a new shared-library document namespace and its author
    /// identity.
    /// # Errors
    /// Returns an error if creating the author or the document namespace
    /// fails.
    pub async fn create_library(&self) -> Result<(Doc, AuthorId)> {
        let author = self
            .docs
            .author_create()
            .await
            .map_err(|_| eyre!("Creating author"))?;

        let doc = self
            .docs
            .create()
            .await
            .map_err(|_| eyre!("Creating doc namespace"))?;

        Ok((doc, author))
    }

    /// Publish `entry` into `doc` under `author`, keyed by its item id.
    /// # Errors
    /// Returns an error if serializing `entry` or writing it to the document
    /// fails.
    pub async fn publish_paper(
        &self,
        doc: &Doc,
        author: AuthorId,
        entry: &SharedItemEntry,
    ) -> Result<()> {
        let key = entry.item_id.as_bytes().to_vec();
        let value = serde_json::to_vec(entry).context("Serializing SharedPaperEntry")?;
        doc.set_bytes(author, key, value)
            .await
            .map_err(|e| eyre!("Failed to publish paper: {}", e.to_string()))?;
        Ok(())
    }

    /// Fetch every [`SharedItemEntry`] currently published in `doc`.
    /// # Errors
    /// Returns an error if the document cannot be queried, an entry cannot
    /// be read, its content cannot be fetched, or it cannot be
    /// deserialized.
    pub async fn list_items(&self, doc: &Doc) -> Result<Vec<SharedItemEntry>> {
        let stream = doc
            .get_many(Query::single_latest_per_key())
            .await
            .map_err(|_| eyre!("Querying doc enries"))?;

        pin!(stream);

        let mut out = Vec::new();
        while let Some(entry) = stream.next().await {
            let entry = entry.map_err(|_| eyre!("Reading doc entry"))?;
            let bytes = self
                .blobs_store
                .blobs()
                .get_bytes(entry.content_hash())
                .await
                .context("Fetching entry contenct bytes")?;
            let parsed: SharedItemEntry =
                serde_json::from_slice(&bytes).context("Deserializing SharedPaperEntry")?;
            out.push(parsed);
        }

        Ok(out)
    }

    /// Generate a read-only ticket that lets other peers subscribe to `doc`.
    /// # Errors
    /// Returns an error if generating the ticket fails.
    pub async fn share_library(&self, doc: &Doc) -> Result<DocTicket> {
        doc.share(ShareMode::Read, AddrInfoOptions::Id)
            .await
            .map_err(|_| eyre!("Generating doc ticket"))
    }

    /// Parse `ticket_str` and import the shared document it references.
    /// # Errors
    /// Returns an error if the ticket cannot be parsed or the document
    /// cannot be imported.
    pub async fn subscribe_library(&self, ticket_str: &str) -> Result<(Doc, NamespaceId)> {
        let ticket = DocTicket::from_str(ticket_str).context("Parsing doc ticket")?;
        let namespace_id = ticket.capability.id();
        let doc = self
            .docs
            .import(ticket)
            .await
            .map_err(|_| eyre!("Importing doc"))?;

        Ok((doc, namespace_id))
    }
}

impl ItemMetadata for SharedItemEntry {
    fn title(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.title)
    }

    fn description(&self) -> Option<Cow<'_, str>> {
        self.description.as_deref().map(Cow::Borrowed)
    }

    fn item_type(&self) -> ItemType {
        ItemType::try_from(self.r#type.as_str()).unwrap_or_default()
    }

    fn authors(&self) -> Vec<String> {
        self.authors.clone()
    }

    fn isbn(&self) -> Option<Cow<'_, str>> {
        self.isbn.as_deref().map(Cow::Borrowed)
    }

    fn doi(&self) -> Option<Cow<'_, str>> {
        self.doi.as_deref().map(Cow::Borrowed)
    }

    fn publication_date(&self) -> Option<Cow<'_, str>> {
        self.publication_date.as_deref().map(Cow::Borrowed)
    }

    fn cover_image_url(&self) -> Option<Cow<'_, str>> {
        self.cover_image_url.as_deref().map(Cow::Borrowed)
    }

    fn source(&self) -> Cow<'_, str> {
        Cow::Owned(format!("Peer:{}", self.owner))
    }

    fn tags(&self) -> Vec<String> {
        self.tags.clone()
    }

    fn container(&self) -> Option<Cow<'_, str>> {
        self.container.as_deref().map(Cow::Borrowed)
    }
}
