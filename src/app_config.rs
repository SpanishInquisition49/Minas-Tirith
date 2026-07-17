use std::{collections::HashMap, sync::OnceLock};

use color_eyre::eyre::{Context, Result, eyre};
use directories::ProjectDirs;
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::Deserialize;

static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();

#[derive(Clone, Debug, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub api_keys: HashMap<String, String>,
}

impl AppConfig {
    pub fn init(proj_dirs: &ProjectDirs) -> Result<()> {
        let cfg = AppConfig::load(proj_dirs)?;
        APP_CONFIG
            .set(cfg)
            .map_err(|_| eyre!("AppConfig already initialized"))
    }

    pub fn app_config() -> &'static AppConfig {
        APP_CONFIG
            .get()
            .expect("AppConfig::init() must be called before app_config()")
    }

    fn load(proj_dirs: &ProjectDirs) -> Result<Self> {
        let config_path = proj_dirs.config_dir().join("config.toml");
        Figment::new()
            .merge(Toml::file(&config_path))
            .merge(Env::prefixed("MINASTIRITH_"))
            .extract()
            .with_context(|| format!("Loading App configuration from {}", config_path.display()))
    }

    pub fn api_key(&self, provider: &str) -> Option<&str> {
        self.api_keys.get(provider).map(|key| key.as_str())
    }
}
