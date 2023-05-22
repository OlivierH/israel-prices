mod counter;
mod file_info;
mod models;
mod parallel_download;
mod store;
mod store_data_download;
mod xml_to_standard;
use crate::models::{ItemKey, ItemPrice};
use crate::{counter::DataCounter, models::ItemInfo};
use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use metrics_exporter_prometheus::PrometheusBuilder;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use std::collections::HashMap;
use std::fs::File;
use std::io::ErrorKind;
use std::sync::Mutex;
use store::*;
use tokio;
use tracing::{debug, error, info, span, Level};
use tracing_subscriber::prelude::*;
mod country_code;
mod nutrition;
mod online_store_data;
mod sanitization;
mod sqlite_utils;
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

    // Note: we need a loop here in case a gz file is malformed and causes an error
    info!("Unzipping all files");
    run("for file in data_raw/*/*.gz; do gunzip $file; done")?;
    // run("gunzip data_raw/*/*.gz")?;

    Ok(())
}

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(short, long, default_value = "./data_raw")]
    dir: String,

    #[arg(short, long, default_value = "")]
    output: String, // unused

    #[arg(long)]
    no_download: bool,

    #[arg(long)]
    no_curate: bool,

    #[arg(long)]
    load_from_json: bool,

    #[arg(long)]
    save_to_json: bool,

    #[arg(long)]
    save_item_infos_to_json: bool,

    #[arg(long)]
    load_item_infos_to_json: bool,

    #[arg(long)]
    save_to_sqlite: bool,

    #[arg(long, default_value = "")]
    save_to_sqlite_only: String,

    #[arg(long)]
    delete_sqlite: bool,

    #[arg(long)]
    no_process: bool,

    #[arg(long)]
    no_build_item_infos: bool,

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

    #[arg(long)]
    fetch_shufersal_metadata: bool,

    #[arg(long)]
    fetch_rami_levy_metadata: bool,

    #[arg(long, default_value = "0")]
    metadata_fetch_limit: usize,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    let prometheus = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install prometheus exporter");
    let log_file = File::create("log.txt")?;
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().with_writer(Mutex::new(log_file)));

    tracing::subscriber::set_global_default(subscriber)?;

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
        let res = std::fs::remove_dir_all("./data_raw");
        if let Err(e) = res {
            if e.kind() == ErrorKind::NotFound {
                info!("data_raw doesn't exist already");
            } else {
                bail!(e);
            }
        }
    }

    if args.delete_sqlite {
        info!("Deleting data.sqlite");
        let path = std::path::Path::new("data.sqlite");
        if !path.exists() {
            info!("data.sqlite doesn't exist already");
        } else {
            std::fs::remove_file("data.sqlite")?;
        }
    }

    if !args.no_download {
        store_data_download::download_all_stores_data(&stores, args.quick, file_limit, args.dir)
            .await;
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

            let prices_file = std::io::BufReader::new(std::fs::File::open("prices.json")?);
            info!("Reading prices from prices.json - this may take some time");
            prices = serde_json::from_reader(prices_file)?;
            info!("Read {} prices from prices.json", prices.len());
        } else {
            info!("Starting processing of files");
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
            info!(
                "There are {} stores files and {} prices files",
                stores_paths.len(),
                price_paths.len()
            );
            info!("Starting to handle stores");
            for store_path in stores_paths {
                debug!("Reading file: {store_path}");
                let chain = xml_to_standard::handle_stores_file(&store_path)?;
                chains.push(chain);
            }
            if args.save_to_json {
                info!("Writing chains.json");
                std::fs::write("chains.json", serde_json::to_string(&chains).unwrap())?;
            }
            info!("Finished to handle stores, starting to handle prices");
            for price_path in price_paths {
                debug!("Reading file: {price_path}");
                let price = xml_to_standard::hande_price_file(&price_path)?;
                prices.push(price);
            }
            if args.save_to_json {
                info!("Writing prices.json");
                std::fs::write("prices.json", serde_json::to_string(&prices).unwrap())?;
            }
            info!("Finished to handle prices");
        }

        #[serde_as]
        #[derive(Default, Serialize, Deserialize, Debug)]
        struct ItemInfos {
            #[serde_as(as = "Vec<(_, _)>")]
            data: HashMap<ItemKey, ItemInfo>,
        }

        let mut item_infos: ItemInfos = ItemInfos::default();

        fn get_item_codes_for_chain(
            chain_id: models::ChainId,
            prices: &Vec<models::Prices>,
        ) -> Vec<models::Barcode> {
            let mut codes = prices
                .iter()
                .filter(|price| price.chain_id == chain_id)
                .flat_map(|price| price.items.iter().map(|item| item.item_code))
                .collect::<Vec<_>>();
            codes.sort();
            codes.dedup();
            codes
        }

        let shufersal_item_codes = get_item_codes_for_chain(7290027600007, &prices);
        info!(
            "Found {} barcodes for shufersal",
            shufersal_item_codes.len()
        );
        let rami_levy_item_codes = get_item_codes_for_chain(7290058140886, &prices);
        info!(
            "Found {} barcodes for rami_levy",
            rami_levy_item_codes.len()
        );

        if args.load_item_infos_to_json {
            let item_infos_file = std::io::BufReader::new(std::fs::File::open("item_infos.json")?);
            info!("Reading item_infos from item_infos.json");
            item_infos = serde_json::from_reader(item_infos_file)?;
            info!(
                "Read {} item_infos from item_infos.json",
                item_infos.data.len()
            );
        } else if !args.no_build_item_infos {
            #[derive(Default, Debug)]
            struct AggregatedData {
                prices: Vec<ItemPrice>,
                names: DataCounter<String>,
                manufacturer_names: DataCounter<String>,
                manufacture_country: DataCounter<String>,
                manufacturer_item_description: DataCounter<String>,
                chains: DataCounter<models::ChainId>,
                unit_qty: DataCounter<String>,
                quantity: DataCounter<String>,
                unit_of_measure: DataCounter<String>,
                b_is_weighted: DataCounter<bool>,
                qty_in_package: DataCounter<String>,
            }

            let mut items_aggregated_data: HashMap<ItemKey, AggregatedData> = HashMap::new();
            info!("Starting to build Aggregated data");
            for price in prices {
                for item in price.items {
                    let item_key = ItemKey::from_item_and_chain(&item, price.chain_id);

                    let data = items_aggregated_data
                        .entry(item_key)
                        .or_insert(AggregatedData::default());
                    data.prices.push(ItemPrice {
                        chain_id: price.chain_id,
                        store_id: price.store_id,
                        price: item.item_price,
                        unit_of_measure_price: item.unit_of_measure_price,
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
                }
            }
            info!("Finished to build Aggregated data");
            for (key, data) in items_aggregated_data.into_iter() {
                item_infos.data.insert(
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
                        prices: data.prices.clone(),
                    },
                );
            }

            if args.save_item_infos_to_json {
                info!("Saving item_infos.json");
                std::fs::write(
                    "item_infos.json",
                    serde_json::to_string(&item_infos).unwrap(),
                )?;
            }
        }
        if args.save_to_sqlite {
            sqlite_utils::save_to_sqlite(&chains, &item_infos.data, &args.save_to_sqlite_only)?;
        }
        if args.fetch_shufersal_metadata {
            let num_of_chunks = shufersal_item_codes.len() / 1000;
            for (i, chunk) in shufersal_item_codes.chunks(1000).enumerate() {
                info!("Fetching shufersal metadata chunk {i}/{num_of_chunks}");
                let shufersal_metadata =
                    online_store_data::fetch_shufersal_metadata(chunk, args.metadata_fetch_limit)
                        .await?;
                sqlite_utils::save_shufersal_metadata_to_sqlite(&shufersal_metadata)?;
                if args.metadata_fetch_limit > 0 {
                    break;
                }
            }
        }
        if args.fetch_rami_levy_metadata {
            let num_of_chunks = rami_levy_item_codes.len() / 1000;
            for (i, chunk) in rami_levy_item_codes.chunks(1000).enumerate() {
                info!("Fetching rami levy metadata chunk {i}/{num_of_chunks}");
                let rami_levy_metadata =
                    online_store_data::fetch_rami_levy_metadata(chunk, args.metadata_fetch_limit)
                        .await?;
                sqlite_utils::save_rami_levy_metadata_to_sqlite(&rami_levy_metadata)?;
                if args.metadata_fetch_limit > 0 {
                    break;
                }
            }
        }
    }
    info!("{}", prometheus.render());
    Ok(())
}
