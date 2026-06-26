use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, Result as AnyhowResult};
use directories::ProjectDirs;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

use crate::{
    database::archive::Archive,
    metadata::{
        crosseref::{CrossrefItem, CrossrefManager},
        openlibrary::{OpenLibraryItem, OpenLibraryManager},
        proxy::MetadataFetcher,
    },
};

mod database;
mod metadata;
mod schema;

fn get_db_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("com", "TheSpanishInquisition", "minastirith")
        .expect("Cannot enstablish data directory");

    let data_dir = proj_dirs.data_dir();

    std::fs::create_dir_all(data_dir).expect("Cannot create data directory");

    data_dir.join("minastirith.db")
}

pub async fn init_db() -> AnyhowResult<SqlitePool> {
    let db_path = get_db_path();
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
async fn main() -> AnyhowResult<()> {
    // NOTE: setting up the Database connection
    let pool = init_db().await.context("Connecting to Database")?;
    let archive = Archive::from_pool(pool);
    archive.migrate().await.context("Running Migrations")?;
    openlibrary(&archive).await?;
    crossref(&archive).await?;
    let items = archive
        .get_all_items()
        .await
        .context("Fetching Items from Archive")?;
    if items.is_empty() {
        println!("No items fetched");
    }
    for item in items {
        println!("========================");
        println!("{item}")
    }
    Ok(())
}

async fn crossref(archive: &Archive) -> AnyhowResult<()> {
    let crosseref = CrossrefManager::new();
    let articles = crosseref.fetch("Taming Undefined Behavior in LLVM").await?;
    if !articles.is_empty() {
        let article = articles.first().unwrap();
        archive
            .add_item::<CrossrefItem>(article)
            .await
            .context("Creating Item in Archive")?;
    }
    Ok(())
}

async fn openlibrary(archive: &Archive) -> AnyhowResult<()> {
    let openlibrary = OpenLibraryManager::new();
    let books = openlibrary
        .fetch("compilers principles, technique & tools")
        .await?;
    if !books.is_empty() {
        let book = books.first().unwrap();
        archive
            .add_item::<OpenLibraryItem>(book)
            .await
            .context("Creating Item in Archive")?;
    }
    Ok(())
}
