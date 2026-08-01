use std::path::Path;

use color_eyre::eyre::{Context, Result, eyre};
use iroh::{Endpoint, PublicKey, endpoint::presets, protocol::Router};
use iroh_blobs::{BlobsProtocol, store::fs::FsStore};
use iroh_docs::protocol::Docs;
use iroh_gossip::Gossip;
use iroh_mdns_address_lookup::MdnsAddressLookup;

use crate::peer2peer::{PrettyDisplay, identity::load_or_create_secret_key};

const SERVICE_NAME: &str = "minastirith-share";

pub struct ShareNode {
    pub(in crate::peer2peer) endpoint: Endpoint,
    pub(in crate::peer2peer) router: Router,
    pub(in crate::peer2peer) blobs_store: FsStore,
    pub(in crate::peer2peer) gossip: Gossip,
    pub docs: Docs,
}

impl ShareNode {
    /// Bind the `ShareNode` endpoint and add the address lookup
    pub async fn bind(data_dir: &Path) -> Result<Self> {
        let secret_key = load_or_create_secret_key(data_dir)?;
        let endpoint = Endpoint::builder(presets::N0)
            .secret_key(secret_key)
            .bind()
            .await
            .context("Binding iroh endpoint")?;

        let mdns = MdnsAddressLookup::builder()
            .service_name(SERVICE_NAME)
            .build(endpoint.id())
            .context("Building mDNS address lookup")?;

        endpoint
            .address_lookup()
            .context("Getting endpoint address lookup")?
            .add(mdns.clone());

        let store_dir = data_dir.join("store");
        std::fs::create_dir_all(&store_dir).context("Creating blobs directory")?;
        let blobs_store = FsStore::load(&store_dir)
            .await
            .context("Failed to create FsStore")?;

        let gossip = Gossip::builder().spawn(endpoint.clone());
        let docs_path = data_dir.join("docs");
        std::fs::create_dir_all(&docs_path)?;
        let docs = Docs::persistent(docs_path)
            .spawn(endpoint.clone(), (*blobs_store).clone(), gossip.clone())
            .await
            .map_err(|e| eyre!("Spawning docs protocol: {}", e.to_string()))?;

        let blobs_protocol = BlobsProtocol::new(&blobs_store, None);

        let builder = Router::builder(endpoint.clone());

        let router = builder
            .accept(iroh_blobs::ALPN, blobs_protocol)
            .accept(iroh_gossip::ALPN, gossip.clone())
            .accept(iroh_docs::ALPN, docs.clone())
            .spawn();

        Ok(Self {
            endpoint,
            router,
            blobs_store,
            gossip,
            docs,
        })
    }
}

impl PrettyDisplay for ShareNode {
    fn node_id(&self) -> PublicKey {
        self.endpoint.id()
    }
}
