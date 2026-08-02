use std::path::Path;

use color_eyre::eyre::{Context, Result, eyre};
use iroh::SecretKey;

use crate::app_config::AppConfig;

const KEY_FILENAME: &str = "share_identity.key";

/// Load the persisted share identity key from `data_dir`, generating and
/// persisting a new one if none exists yet. Returns an ephemeral,
/// non-persisted key if `AppConfig::ephemeral_identity` is set.
/// # Errors
/// Returns an error if the existing key file is corrupted, or if a newly
/// generated key cannot be written to disk.
pub fn load_or_create_secret_key(data_dir: &Path) -> Result<SecretKey> {
    let path = data_dir.join(KEY_FILENAME);

    if AppConfig::get().ephemeral_identity {
        return Ok(SecretKey::generate());
    }

    if let Ok(bytes) = std::fs::read(&path) {
        let array: [u8; 32] = bytes
            .try_into()
            .map_err(|_| eyre!("Corrupted share identity key at {}", path.display()))?;
        return Ok(SecretKey::from_bytes(&array));
    }

    let secret_key = SecretKey::generate();

    std::fs::write(&path, secret_key.to_bytes())
        .with_context(|| format!("Writing share identity key to {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .context("Restricting permission on share identity key")?;
    }

    Ok(secret_key)
}
