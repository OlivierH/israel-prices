use anyhow::Result;
use clap::Parser;
use israel_prices::{log_utils, process_raw_files, sqlite_utils};
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

    let processed_data = process_raw_files::process_raw_files(&args.dir, &args.store)?;

    sqlite_utils::save_to_sqlite(&processed_data.chains, &processed_data.item_infos)?;

    info!("{}", prometheus.render());
    Ok(())
}
