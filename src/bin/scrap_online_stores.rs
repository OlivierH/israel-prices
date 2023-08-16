use std::{fs::File, sync::Mutex};

use anyhow::Result;
use clap::Parser;
use israel_prices::online_store_data;
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

    let victory_metadata = online_store_data::scrap_excalibur_data(
        "https://www.victoryonline.co.il/v2/retailers/1470",
        args.metadata_fetch_limit,
    )
    .await?;   
    // sqlite_utils::save_victory_metadata_to_sqlite("Victory", &victory_metadata)?;
    Ok(())
}
