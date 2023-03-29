mod file_info;
mod parallel_download;
mod store;
mod store_data_download;
use slog::{self, info, o, Drain};
use slog_async;
use slog_term;
use std::env;
use store::*;
use tokio;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!("P" => "raw_downloader"));
    info!(log, "Start");

    let args: Vec<String> = env::args().collect();
    let quick = args.contains(&String::from("q"));
    let minimal = args.contains(&String::from("m"));
    let debug = args.contains(&String::from("d"));

    let file_limit = match minimal {
        true => Some(5),
        false => None,
    };

    let stores = match minimal {
        false => get_store_configs(),
        true => get_minimal_store_configs(),
    };
    let stores = match debug {
        false => stores,
        true => get_debug_store_configs(),
    };

    store_data_download::download_all_stores_data(&stores, quick, file_limit, &log).await;
}
