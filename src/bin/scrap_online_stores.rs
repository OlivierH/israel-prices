use std::{
    fs::File,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use clap::Parser;
use israel_prices::{models::Barcode, online_store_data::scrap_stores_and_save_to_sqlite};
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

    let shufersal_codes = Arc::new(
        std::fs::read_to_string(&args.shufersal_codes_filename)
            .unwrap()
            .lines()
            .filter_map(|s| s.parse::<Barcode>().ok())
            .collect::<Vec<Barcode>>(),
    );

    scrap_stores_and_save_to_sqlite(
        args.metadata_fetch_limit,
        match args.store.is_empty() {
            true => None,
            false => Some(&args.store),
        },
        shufersal_codes,
    )
    .await?;
    Ok(())
}
