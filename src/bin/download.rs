use anyhow::Result;
use clap::Parser;
use israel_prices::store::get_store_configs;
use israel_prices::{curate_data_raw, log_utils, store_data_download};
use itertools::Itertools;
use metrics_exporter_prometheus::PrometheusBuilder;
use tokio;
use tracing::info;

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(short, long, default_value = "./data_raw")]
    dir: String,

    #[arg(long)]
    clear_files: bool,

    #[arg(long)]
    quick: bool,

    #[arg(long, default_value = "")]
    store: String,
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

    store_data_download::download_all_stores_data(&stores, args.quick, None, args.dir).await;
    curate_data_raw::curate_data_raw()?;
    info!("{}", prometheus.render());
    Ok(())
}
