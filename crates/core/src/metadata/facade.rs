use std::{sync::Arc, time::Duration};

use reqwest::Client;
use tokio::time::timeout;

use crate::{
    app_config::AppConfig,
    metadata::{
        dedup::MergedCandidate,
        providers::{
            core::CoreManager, crossref::CrossrefManager, google_books::GoogleBooksManager,
            openalex::OpenAlexManager, openlibrary::OpenLibraryManager,
            semantic_scholar::SemanticScholarManager,
        },
        proxy::MetadataFetcher,
    },
};

/// Facade that hides all the metadata providers, aggregate their results
/// with a generic simple API
pub struct MetadataProvider {
    client: Arc<Client>,
    providers: Vec<Arc<dyn MetadataFetcher>>,
}

impl Default for MetadataProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MetadataProvider {
    /// Build a [`MetadataProvider`] with every built-in provider, plus any
    /// optional providers (Google Books, CORE) whose API key is configured.
    #[must_use]
    pub fn new() -> Self {
        let cfg = AppConfig::get();
        let mut providers: Vec<Arc<dyn MetadataFetcher>> = vec![
            Arc::new(OpenLibraryManager::new()),
            Arc::new(CrossrefManager::new()),
            Arc::new(OpenAlexManager::new()),
            Arc::new(SemanticScholarManager::new()),
        ];
        if let Some(key) = cfg.api_key("google_books") {
            providers.push(Arc::new(GoogleBooksManager::new(key.to_string())));
        }
        if let Some(key) = cfg.api_key("core") {
            providers.push(Arc::new(CoreManager::new(key.to_string())));
        }

        Self {
            client: Arc::new(Client::new()),
            providers,
        }
    }

    /// Kicks off the metadata fetching for all providers, and waits for their results
    /// then aggregate them into a common vector
    pub async fn fetch(&self, title: &str) -> Vec<MergedCandidate> {
        let mut res = Vec::new();
        let mut tasks = tokio::task::JoinSet::new();
        for provider in &self.providers {
            let client = self.client.clone();
            let title = title.to_string();
            let p = provider.clone();
            tasks.spawn(async move {
                if let Ok(result) = timeout(Duration::from_secs(30), p.fetch(client, title)).await {
                    result.unwrap_or_default()
                } else {
                    let provider_name = p.name();
                    tracing::warn!(
                        provider = provider_name,
                        "Timeout fired while fetching metadata candidates"
                    );
                    vec![]
                }
            });
        }

        for metadata in tasks.join_all().await.into_iter().flatten() {
            res.push(metadata);
        }
        MergedCandidate::merge_candidates(res)
    }

    /// Kicks off a best effort abstract search for each provider
    /// Returning `String` whenever a provider find a valid abstract, `None` otherwise
    pub async fn fetch_abstract(
        &self,
        title: &str,
        doi: Option<String>,
        isbn: Option<String>,
    ) -> Option<String> {
        for provider in &self.providers {
            let client = self.client.clone();

            if let Ok(result) = timeout(
                Duration::from_secs(30),
                provider.fetch_abstract(client, title.to_string(), doi.clone(), isbn.clone()),
            )
            .await
            {
                match result {
                    Ok(maybe_abstract) => {
                        if let Some(text) = maybe_abstract
                            && !text.trim().is_empty()
                        {
                            return Some(text);
                        }
                    }
                    Err(e) => {
                        tracing::error!(error = %e, provider = provider.name(), "Failed to fetch abstract");
                    }
                }
            } else {
                tracing::warn!(
                    provider = provider.name(),
                    "Timeout fired while fetching abstract"
                );
            }
        }
        None
    }
}
