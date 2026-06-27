use std::{path::PathBuf, str::FromStr};

use color_eyre::eyre::Context;
use directories::ProjectDirs;
use ratatui_image::picker::Picker;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

use crate::{
    database::archive::Archive,
    metadata::{
        crosseref::{CrossrefItem, CrossrefManager},
        image_cache::ImageCache,
        openlibrary::{OpenLibraryItem, OpenLibraryManager},
        proxy::MetadataFetcher,
    },
    tui::{app::App, event::run},
};

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
    println!("DB path: {:?}", db_path);

    let options = SqliteConnectOptions::from_str(db_path.to_str().unwrap())?
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    Ok(pool)
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    // NOTE: setting up the Database connection
    let proj_dirs = ProjectDirs::from("com", "TheSpanishInquisition", "minastirith")
        .expect("Cannot Enstablish Data directories");
    let pool = init_db(&proj_dirs)
        .await
        .context("Connecting to Database")?;
    let archive = Archive::from_pool(pool);
    archive.migrate().await.context("Running Migrations")?;
    //openlibrary(&archive).await?;

    let mut terminal = ratatui::init();
    let picker =
        Picker::from_query_stdio().context("Querying terminal for image graphics protocol")?;

    let image_cache = ImageCache::new(get_image_cache_path(&proj_dirs));
    let mut app = App::new(archive, picker, image_cache).await?;
    run(&mut terminal, &mut app).await?;
    ratatui::restore();
    Ok(())
}
