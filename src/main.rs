use std::{path::PathBuf, str::FromStr};

use color_eyre::eyre::Context;
use directories::ProjectDirs;
use ratatui_image::picker::Picker;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

use crate::{
    app_config::AppConfig,
    database::archive::Archive,
    metadata::image_cache::ImageCache,
    tui::{app::App, event::run},
};

pub mod app_config;
mod database;
mod metadata;
mod schema;
mod tui;

fn get_db_path(proj_dirs: &ProjectDirs) -> PathBuf {
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir).expect("Cannot create data directory");
    data_dir.join("minastirith.db")
}

fn get_image_cache_path(proj_dirs: &ProjectDirs) -> PathBuf {
    let cache_dir = proj_dirs.cache_dir().join("covers");
    std::fs::create_dir_all(&cache_dir).expect("Cannot crate cache dir");
    cache_dir
}

pub async fn init_db(proj_dirs: &ProjectDirs) -> color_eyre::Result<SqlitePool> {
    let db_path = get_db_path(proj_dirs);

    let options = SqliteConnectOptions::from_str(
        db_path
            .to_str()
            .expect("Database path should be UTF-8 encoded"),
    )?
    .create_if_missing(true)
    .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    Ok(pool)
}

pub fn init(proj_dirs: &ProjectDirs) -> color_eyre::Result<WorkerGuard> {
    let log_dir = proj_dirs.data_dir().join("logs");
    std::fs::create_dir_all(&log_dir).context("Creating log directory")?;

    let file_appender = tracing_appender::rolling::daily(&log_dir, "minastirith.log");

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_env_filter(filter)
        .with_target(true)
        .init();
    Ok(guard)
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let proj_dirs = ProjectDirs::from("com", "TheSpanishInquisition", "minastirith")
        .expect("Cannot Enstablish Data directories");
    // NOTE: loafing app configuration
    AppConfig::init(&proj_dirs).context("Loading App configuration")?;
    // NOTE: setting up the logger
    let _log_guard = init(&proj_dirs).context("Initializing logging")?;
    // NOTE: setting up the Database connection
    let pool = init_db(&proj_dirs)
        .await
        .context("Connecting to Database")?;
    let archive = Archive::from_pool(pool);
    // NOTE: applying database migrations
    archive.migrate().await.context("Running Migrations")?;

    let mut terminal = ratatui::init();
    let picker =
        Picker::from_query_stdio().context("Querying terminal for image graphics protocol")?;

    let image_cache = ImageCache::new(get_image_cache_path(&proj_dirs));
    let mut app = App::new(archive, picker, image_cache).await?;
    run(&mut terminal, &mut app).await?;
    ratatui::restore();
    Ok(())
}
