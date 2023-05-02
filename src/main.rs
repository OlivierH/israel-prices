mod counter;
mod file_info;
mod models;
mod parallel_download;
mod store;
mod store_data_download;
mod xml_to_standard;
use crate::models::ItemKey;
use crate::{counter::Counter, models::ItemInfo};
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::collections::HashMap;
use store::*;
use tokio;
use tracing::{debug, error, info, span, Level};
use tracing_subscriber::EnvFilter;
mod country_code;
mod sanitization;
mod xml;
fn run(command: &str) -> Result<()> {
    let span = span!(Level::INFO, "Run command", command);
    let _enter = span.enter();
    debug!("Start");
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()?;
    if !output.stdout.is_empty() {
        debug!(output = String::from_utf8(output.stdout)?, "Output");
    }
    if !output.stderr.is_empty() {
        error!(error = String::from_utf8(output.stderr)?, "Error",);
    }
    Ok(())
}

fn curate_data_raw() -> Result<()> {
    let span = span!(Level::INFO, "curate_data_raw");
    let _enter = span.enter();
    // Rami levy has two different stores files, one of them with a single store that is already present in the first stores file.
    info!("Deleting superfluous and incomplete Rami levy store file");
    run("rm data_raw/rami_levy/storesfull* -f")?;

    info!("Deleting empty files");
    run("find data_raw -type f -empty -print -delete")?;

    info!("Deleting x1 files");
    run("find data_raw -type f -name \"*.x1\" -print -delete")?;

    info!("Unzipping all files");
    run("gunzip data_raw/*/*.gz")?;

    Ok(())
}

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(short, long, default_value = "../data")]
    input: String,

    #[arg(short, long, default_value = "")]
    output: String,

    #[arg(long)]
    no_download: bool,

    #[arg(long)]
    no_curate: bool,

    #[arg(long)]
    load_from_json: bool,

    #[arg(long)]
    save_to_json: bool,

    #[arg(long)]
    no_process: bool,

    #[arg(long)]
    clear_files: bool,

    #[arg(long)]
    quick: bool,

    #[arg(long)]
    minimal: bool,

    #[arg(long)]
    debug: bool,

    #[arg(long, default_value = "")]
    processing_filter: String,

    #[arg(long, default_value = "")]
    store: String,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Starting");

    let args = Args::parse();

    let file_limit: Option<usize> = match args.minimal {
        true => Some(5),
        false => None,
    };

    let stores = match args.minimal {
        false => get_store_configs(),
        true => get_minimal_store_configs(),
    };
    let stores = match args.debug {
        false => stores,
        true => get_debug_store_configs(),
    };
    let stores = match args.store.as_str() {
        "" => stores,
        store => {
            vec![get_store_config(store).ok_or(anyhow!("{store} is not an existing store name"))?]
        }
    };

    if args.clear_files {
        info!("Deleting data_raw");
        std::fs::remove_dir_all("./data_raw")?;
    }

    if !args.no_download {
        store_data_download::download_all_stores_data(&stores, args.quick, file_limit).await;
    }
    if !args.no_curate {
        curate_data_raw()?;
    }

    if !args.no_process {
        let mut chains: Vec<models::Chain> = Vec::new();
        let mut prices: Vec<models::Prices> = Vec::new();

        if args.load_from_json {
            let chains_file = std::io::BufReader::new(std::fs::File::open("chains.json")?);
            info!("Reading chains from chains.json");
            chains = serde_json::from_reader(chains_file)?;
            info!("Read {} chains from chains.json", chains.len());
            debug!("test_debug");

            let prices_file = std::io::BufReader::new(std::fs::File::open("prices.json")?);
            info!("Reading prices from prices.json - this may take some time");
            prices = serde_json::from_reader(prices_file)?;
            info!("Read {} prices from prices.json", prices.len());
        } else {
            let paths = walkdir::WalkDir::new(std::path::Path::new("data_raw"))
                .into_iter()
                .filter_map(|e| e.ok())
                .map(|d| d.into_path())
                .filter(|path| path.is_file())
                .filter_map(|path| path.to_str().map(|s| s.to_owned()))
                .filter(|path| !path.ends_with(".gz"))
                .filter(|path| {
                    args.processing_filter == "" || path.contains(&args.processing_filter)
                });

            let (price_paths, stores_paths): (Vec<String>, Vec<String>) = paths.partition(|path| {
                let filename = path.rsplit_once("/").unwrap().1;
                filename.starts_with("Price") || filename.starts_with("price")
            });
            for store_path in stores_paths {
                debug!("Reading file: {store_path}");
                let chain = xml_to_standard::handle_stores_file(&store_path)?;
                chains.push(chain);
            }
            if args.save_to_json {
                std::fs::write("chains.json", serde_json::to_string(&chains).unwrap())?;
            }
            for price_path in price_paths {
                debug!("Reading file: {price_path}");
                let price = xml_to_standard::hande_price_file(&price_path)?;
                prices.push(price);
            }
            if args.save_to_json {
                std::fs::write("prices.json", serde_json::to_string(&prices).unwrap())?;
            }
        }

        #[derive(Default, serde::Serialize)]
        struct ItemPrice {
            chain_id: i64,
            store_id: i32,
            price: String,
        }
        #[derive(Default)]
        struct AggregatedData {
            prices: Vec<ItemPrice>,
            names: Counter<String>,
            manufacturer_names: Counter<String>,
            manufacture_country: Counter<String>,
            manufacturer_item_description: Counter<String>,
            chains: Counter<models::ChainId>,
            unit_qty: Counter<String>,
            quantity: Counter<String>,
            unit_of_measure: Counter<String>,
            b_is_weighted: Counter<bool>,
            qty_in_package: Counter<String>,
        }

        let mut items_aggregated_data: HashMap<ItemKey, AggregatedData> = HashMap::new();

        for price in prices {
            for item in price.items {
                let item_key = ItemKey::from_item_and_chain(&item, price.chain_id);

                let add_to_data = |data: &mut AggregatedData| {
                    data.prices.push(ItemPrice {
                        chain_id: price.chain_id,
                        store_id: price.store_id,
                        price: item.item_price,
                    });
                    data.names.inc(sanitization::sanitize_name(&item.item_name));
                    data.manufacturer_names.inc(item.manufacturer_name);
                    data.manufacture_country.inc(item.manufacture_country);
                    data.manufacturer_item_description
                        .inc(item.manufacturer_item_description);
                    data.chains.inc(price.chain_id);
                    data.unit_qty.inc(item.unit_qty);
                    data.quantity.inc(item.quantity);
                    data.unit_of_measure.inc(item.unit_of_measure);
                    data.b_is_weighted.inc(item.b_is_weighted);
                    data.qty_in_package.inc(item.qty_in_package);
                };
                items_aggregated_data
                    .entry(item_key)
                    .and_modify(add_to_data)
                    .or_insert_with(|| {
                        let data = AggregatedData::default();
                        data
                    });
            }
        }

        let mut item_infos: HashMap<ItemKey, ItemInfo> = HashMap::new();

        for (key, data) in items_aggregated_data.into_iter() {
            item_infos.insert(
                key,
                ItemInfo {
                    item_name: counter::longest(&data.names).context(key)?.to_string(),
                    manufacturer_name: counter::longest(&data.manufacturer_names)
                        .context(key)?
                        .to_string(),
                    manufacturer_item_description: counter::longest(
                        &data.manufacturer_item_description,
                    )
                    .context(key)?
                    .to_string(),
                    manufacture_country: counter::longest(&data.manufacture_country)
                        .context(key)?
                        .to_string(),
                    unit_qty: counter::longest(&data.unit_qty).context(key)?.to_string(),
                    quantity: counter::longest(&data.quantity).context(key)?.to_string(),
                    unit_of_measure: counter::longest(&data.unit_of_measure)
                        .context(key)?
                        .to_string(),
                    b_is_weighted: data.b_is_weighted.most_common().context(key)?.clone(),
                    qty_in_package: counter::longest(&data.qty_in_package)
                        .context(key)?
                        .to_string(),
                },
            );
        }
    }
    Ok(())
}
