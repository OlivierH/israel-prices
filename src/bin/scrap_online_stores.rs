use std::{fs::File, sync::Mutex};

use anyhow::Result;
use clap::Parser;
use israel_prices::{online_store, online_store_data, sqlite_utils};
use metrics_exporter_prometheus::PrometheusBuilder;
use tokio;
use tracing::{debug, error, info, span, Level};
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

    #[arg(long)]
    fetch_shufersal_metadata: bool,

    #[arg(long)]
    fetch_rami_levy_metadata: bool,

    #[arg(long)]
    fetch_victory_metadata: bool,

    #[arg(long)]
    fetch_yenot_bitan_metadata: bool,

    #[arg(long)]
    fetch_mega_metadata: bool,

    #[arg(long)]
    fetch_maayan_2000_metadata: bool,

    #[arg(long)]
    fetch_am_pm_metadata: bool,

    #[arg(long)]
    fetch_tiv_taam_metadata: bool,

    #[arg(long)]
    fetch_keshet_metadata: bool,

    #[arg(long)]
    fetch_shukcity_metadata: bool,

    #[arg(long)]
    fetch_yochananof_metadata: bool,

    #[arg(long, default_value = "0")]
    metadata_fetch_limit: usize,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    let log_file = File::create("log.txt")?;
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            "scrap_online_stores=debug,israel_prices=debug",
        ))
        .with(tracing_subscriber::fmt::layer())
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

    for online_store in online_store::get_online_stores() {
        let scraped_data = match online_store.website {
            online_store::Website::Excalibur(url) => {
                online_store_data::scrap_excalibur_data(
                    online_store.name,
                    url,
                    args.metadata_fetch_limit,
                )
                .await?
            }
            _ => panic!("SSSS"),
        };
        info!(
            "From store {}, got {} elements",
            online_store.name,
            scraped_data.len()
        );
        sqlite_utils::save_scraped_data_to_sqlite(&scraped_data)?;
    }

    // for d in victory_metadata {
    //     println!("{:?}", d);
    // }
    // sqlite_utils::save_victory_metadata_to_sqlite("Victory", &victory_metadata)?;
    Ok(())
}
