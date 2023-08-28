use std::{fs::File, sync::Mutex};

use anyhow::Result;
use clap::Parser;
use futures::future;
use israel_prices::{
    online_store::{self},
    online_store_data::scrap_store_and_save_to_sqlite,
};
use tokio;
use tracing::info;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(long)]
    save_to_sqlite: bool,

    #[arg(long, default_value = "")]
    save_to_sqlite_only: String,

    #[arg(long)]
    delete_sqlite: bool,

    #[arg(long, default_value = "")]
    store: String,

    #[arg(long, default_value = "0")]
    metadata_fetch_limit: usize,

    #[arg(long, default_value = "shufersal_codes.txt")]
    shufersal_codes_filename: String,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    let log_file = File::create("log.txt")?;
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            "scrap_online_stores=debug,israel_prices=debug",
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .with_line_number(true)
                .with_file(true),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(Mutex::new(log_file)));

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting");

    let args = Args::parse();
    if args.delete_sqlite {
        info!("Deleting data.sqlite");
        let path = std::path::Path::new("data.sqlite");
        if !path.exists() {
            info!("data.sqlite doesn't exist already");
        } else {
            std::fs::remove_file("data.sqlite")?;
        }
    }
    let mut tasks = Vec::new();
    for online_store in online_store::get_online_stores() {
        if !args.store.is_empty() && !args.store.contains(online_store.name) {
            continue;
        }
        tasks.push(tokio::spawn(scrap_store_and_save_to_sqlite(
            online_store,
            args.metadata_fetch_limit,
            args.shufersal_codes_filename.clone(),
        )));
    }
    for result in future::join_all(tasks).await {
        let _outcome = result??;
    }
    Ok(())
}
