mod file_info;
mod parallel_download;
mod store;
mod store_data_download;
use anyhow::Result;
use slog::{self, debug, error, info, o, Drain, Logger};
use slog_async;
use slog_term;
use std::env;
use store::*;
use tokio;

fn run(command: &str, log: &Logger) -> Result<()> {
    debug!(log, "Running command {}", command);
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()?;
    if !output.stdout.is_empty() {
        debug!(log, "Output: {}", String::from_utf8(output.stdout)?);
    }
    if !output.stderr.is_empty() {
        debug!(log, "Error: {}", String::from_utf8(output.stderr)?);
    }
    Ok(())
}

fn curate_data_raw(log: &Logger) -> Result<()> {
    // Rami levy has two different stores files, one of them with a single store that is already present in the first stores file.
    info!(
        log,
        "Deleting superfluous and incomplete Rami levy store file"
    );
    run("rm data_raw/rami_levy/storesfull* -f", &log)?;

    info!(log, "Deleting empty files");
    run("find data_raw -type f -empty -print -delete", &log)?;

    info!(log, "Deleting x1 files");
    run("find data_raw -type f -name \"*.x1\" -print -delete", &log)?;

    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
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

    curate_data_raw(&log)?;
    Ok(())
}
