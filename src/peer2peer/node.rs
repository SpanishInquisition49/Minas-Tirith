use std::path::Path;

use color_eyre::eyre::{Context, Result};
use iroh::{Endpoint, PublicKey, endpoint::presets};
use iroh_mdns_address_lookup::MdnsAddressLookup;

use crate::peer2peer::{PrettyDisplay, identity::load_or_create_secret_key};

const SERVICE_NAME: &str = "minastirith-share";

pub struct ShareNode {
    pub endpoint: Endpoint,
    pub mdns: MdnsAddressLookup,
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

        Ok(Self { endpoint, mdns })
    }
}

impl PrettyDisplay for ShareNode {
    fn node_id(&self) -> PublicKey {
        self.endpoint.id()
    }
}
