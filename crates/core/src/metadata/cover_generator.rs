use std::path::{Path, PathBuf};

use color_eyre::eyre::{Context, bail};

use crate::metadata::image_cache::ImageCache;

/// Generate a cover image for `file_path` based on its extension (PDF or
/// EPUB), caching the result via `cache`. Returns `None` if the file type
/// isn't supported or no cover could be extracted.
/// # Errors
/// Returns an error if cover generation for the underlying file type fails.
pub async fn generate_cover(
    file_path: &Path,
    item_id: i32,
    cache: &ImageCache,
) -> color_eyre::Result<Option<PathBuf>> {
    match file_path.extension().and_then(|e| e.to_str()) {
        Some("pdf") => generate_from_pdf(file_path, item_id, cache).await,
        Some("epub") => generate_from_epub(file_path, item_id, cache).await,
        _ => Ok(None),
    }
}

/// Render the first page of the PDF at `file_path` to a cached cover image
/// using `pdftoppm`.
/// # Errors
/// Returns an error if `pdftoppm` fails to run or exits unsuccessfully.
pub async fn generate_from_pdf(
    file_path: &Path,
    item_id: i32,
    cache: &ImageCache,
) -> color_eyre::Result<Option<PathBuf>> {
    let prefix = cache.generated_prefix(item_id);
    let status = tokio::process::Command::new("pdftoppm")
        .args(["-png", "-f", "1", "-l", "1", "-r", "150", "-singlefile"])
        .arg(file_path)
        .arg(&prefix)
        .status()
        .await
        .context("Running pdftoppm")?;

    if !status.success() {
        bail!("pdftoppm exited with status: {status}");
    }

    let out = prefix.with_extension("png");
    if out.exists() {
        Ok(Some(out))
    } else {
        Ok(None)
    }
}

/// Extract the cover image embedded in the EPUB at `file_path` and store it
/// via `cache`. Returns `None` if the EPUB has no cover.
/// # Errors
/// Returns an error if the EPUB cannot be opened or the extraction task
/// fails to join.
pub async fn generate_from_epub(
    file_path: &Path,
    item_id: i32,
    cache: &ImageCache,
) -> color_eyre::Result<Option<PathBuf>> {
    let file_path = file_path.to_path_buf();
    let cache = cache.clone();
    tokio::task::spawn_blocking(move || -> color_eyre::Result<Option<PathBuf>> {
        let mut doc = epub::doc::EpubDoc::new(&file_path).context("Opening EpubDoc")?;
        let Some((bytes, _mime)) = doc.get_cover() else {
            return Ok(None);
        };
        let path = cache.store_generated(item_id, &bytes, "jpg")?;
        Ok(Some(path))
    })
    .await
    .context("Joining epub cover extraction")?
}
