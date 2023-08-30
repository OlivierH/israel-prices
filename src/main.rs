use anyhow::Result;
use clap::Parser;
use israel_prices::process_raw_files::process_raw_files;
use israel_prices::sqlite_utils::maybe_delete_database;
use israel_prices::store::get_store_configs;
use israel_prices::{
    constants, curate_data_raw, log_utils, online_store_data, sqlite_utils, store_data_download,
};
use itertools::Itertools;
use metrics_exporter_prometheus::PrometheusBuilder;
use std::sync::Arc;
use tokio;
use tracing::info;

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(short, long, default_value = "./data_raw")]
    dir: String,

    #[arg(long)]
    save_to_sqlite: bool,

    #[arg(long)]
    delete_sqlite: bool,

    #[arg(long)]
    clear_files: bool,

    #[arg(long)]
    quick: bool,

    #[arg(long, default_value = "")]
    store: String,

    #[arg(long, default_value = "")]
    scrap_only: String,

    #[arg(long)]
    fetch_yochananof_metadata: bool,

    #[arg(long, default_value = "0")]
    metadata_fetch_limit: usize,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    let prometheus = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install prometheus exporter");
    log_utils::init()?;

    info!("Starting");

    let args = Args::parse();

    let stores = get_store_configs()
        .into_iter()
        .filter(|s| args.store.is_empty() || args.store.contains(s.name))
        .collect_vec();

    if args.clear_files {
        info!("Deleting {} directory", args.dir);
        let path = std::path::Path::new(&args.dir);
        if !path.exists() {
            info!("data_raw doesn't exist already");
        } else {
            std::fs::remove_dir_all(&args.dir)?;
        }
    }

    maybe_delete_database(args.delete_sqlite)?;

    store_data_download::download_all_stores_data(&stores, args.quick, None, &args.dir).await;
    curate_data_raw::curate_data_raw()?;

    let processed_data = process_raw_files(&args.dir, &args.store)?;

    if args.save_to_sqlite {
        sqlite_utils::save_to_sqlite(&processed_data.chains, &processed_data.item_infos)?;
    }

    let shufersal_codes = Arc::new(sqlite_utils::get_codes_from_chain_id(constants::SHUFERSAL)?);
    info!("Found {} barcodes for shufersal", shufersal_codes.len());
    online_store_data::scrap_stores_and_save_to_sqlite(
        args.metadata_fetch_limit,
        match args.scrap_only.is_empty() {
            true => None,
            false => Some(&args.scrap_only),
        },
        shufersal_codes,
    )
    .await?;

    if args.fetch_yochananof_metadata {
        let yochananof_metadata = online_store_data::fetch_yochananof_metadata().await?;
        sqlite_utils::save_yochananof_metadata_to_sqlite(&yochananof_metadata)?;
    }
    info!("{}", prometheus.render());
    Ok(())
}
