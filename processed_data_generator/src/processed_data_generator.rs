use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    path::PathBuf,
};

use anyhow::anyhow;
use anyhow::{Ok, Result};
use clap::Parser;
use models::Item;
use slog::{self, info, o, trace, Drain, Logger};
use slog_async;
use slog_term;

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(short, long, default_value = "../data")]
    input: String,

    #[arg(short, long, default_value = "")]
    output: String,

    #[arg(short, long, default_value = "false")]
    debug: bool,
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
struct ItemKey {
    chain_id: Option<i64>,
    item_code: i64,
}

#[derive(Default, serde::Serialize)]
struct ItemPrice {
    chain_id: i64,
    store_id: i32,
    price: String,
}

fn prices_paths(input: &str) -> Vec<PathBuf> {
    walkdir::WalkDir::new(std::path::Path::new(input).join("prices"))
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|dir| dir.into_path())
        .filter(|path| path.is_file())
        .filter(|path| !path.ends_with("csv_new"))
        .collect::<Vec<PathBuf>>()
}
fn extract_chain_id_and_store_id(path: &PathBuf) -> Result<(i64, i32)> {
    let (chain_id, store_id) = path
        .file_name()
        .ok_or(anyhow!("Could not extract filename"))?
        .to_str()
        .ok_or(anyhow!("Could not convert filename to str"))?
        .split_once(".")
        .ok_or(anyhow!("No dot in filename"))?
        .0
        .split_once("_")
        .ok_or(anyhow!("No _ in filename"))?;
    let chain_id = chain_id.parse::<i64>()?;
    let store_id = store_id.parse::<i32>()?;
    Ok((chain_id, store_id))
}

fn read_all_price_data(
    input: &str,
    debug: bool,
) -> Result<(
    HashMap<ItemKey, Vec<ItemPrice>>,
    HashMap<ItemKey, HashSet<String>>, // names
    HashMap<ItemKey, HashSet<String>>, // manufacturer names
)> {
    let mut prices: HashMap<ItemKey, Vec<ItemPrice>> = HashMap::new();
    let mut names: HashMap<ItemKey, HashSet<String>> = HashMap::new();
    let mut manufacturer_names: HashMap<ItemKey, HashSet<String>> = HashMap::new();
    let paths = prices_paths(input);

    let paths = match debug {
        true => &paths[0..20],
        false => &paths[..],
    };

    for path in paths {
        let (chain_id, store_id) = extract_chain_id_and_store_id(&path)?;
        let mut reader = csv::Reader::from_path(&path)?;
        for item in reader.deserialize::<Item>() {
            let item = item?;

            let item_key = ItemKey {
                chain_id: match item.internal_code {
                    true => Some(chain_id),
                    false => None,
                },
                item_code: item.item_code,
            };
            prices
                .entry(item_key)
                .or_insert_with(|| Vec::new())
                .push(ItemPrice {
                    chain_id: chain_id,
                    store_id: store_id,
                    price: item.item_price,
                });
            names
                .entry(item_key)
                .or_insert_with(|| HashSet::new())
                .insert(item.item_name);
            manufacturer_names
                .entry(item_key)
                .or_insert_with(|| HashSet::new())
                .insert(item.manufacturer_name);
        }
    }
    Ok((prices, names, manufacturer_names))
}

fn write_all_price_data(prices: HashMap<ItemKey, Vec<ItemPrice>>, output: &str) -> Result<()> {
    let dir = std::path::Path::new(output).join("prices_per_product");
    std::fs::create_dir_all(&dir)?;
    for (key, mut prices) in prices.into_iter() {
        let mut filename = key.item_code.to_string();
        if let Some(chain_id) = key.chain_id {
            filename += &format!("_{}", chain_id);
        }
        filename += ".csv";
        prices.sort_by_key(|k| (k.chain_id, k.store_id));
        let mut writer = csv::Writer::from_path(dir.join(filename))?;
        for price in prices {
            writer.serialize(&price)?;
        }
    }

    Ok(())
}

fn get_canonical_names(names: &HashMap<ItemKey, HashSet<String>>) -> HashMap<ItemKey, &String> {
    let mut output: HashMap<ItemKey, &String> = HashMap::new();
    for (key, names) in names.iter() {
        let name = names.iter().max_by_key(|s| s.len()).unwrap();
        output.insert(*key, name);
    }

    output
}

fn write_all_product_data(names: HashMap<ItemKey, HashSet<String>>, output: &str) -> Result<()> {
    println!(
        "Starting to write product data, got {} elements",
        names.len()
    );
    for (_key, names) in names.into_iter() {
        if names.len() > 5 {
            println!("-----");
            for name in names {
                println!("  {name}");
            }
        }
    }

    Ok(())
}

fn update_store_data_in_place(
    dir: &str,
    names: &HashMap<ItemKey, &String>,
    manufacturer_names: &HashMap<ItemKey, &String>,
    log: &Logger,
) -> Result<()> {
    let log = log.new(o!("op" => "update_store_data_in_place"));
    let paths = prices_paths(&dir);
    for path in paths {
        let new_path = path
            .as_os_str()
            .to_str()
            .ok_or(anyhow!("Path is not unicode"))?
            .to_owned()
            + "_new";
        trace!(log, "Handling {}", path.as_os_str().to_str().unwrap());
        {
            let (chain_id, _) = extract_chain_id_and_store_id(&path)?;
            let mut reader = csv::Reader::from_path(&path)?;
            let mut writer = csv::Writer::from_path(&new_path)?;
            for item in reader.deserialize::<Item>() {
                let mut item = item?;

                let item_key = ItemKey {
                    chain_id: match item.internal_code {
                        true => Some(chain_id),
                        false => None,
                    },
                    item_code: item.item_code,
                };
                names
                    .get(&item_key)
                    .map(|name| item.item_name = name.to_string());
                manufacturer_names
                    .get(&item_key)
                    .map(|name| item.manufacturer_name = name.to_string());
                writer.serialize(item)?;
            }
        }
        std::fs::rename(new_path, path)?;
    }
    Ok(())
}

fn run() -> Result<()> {
    let mut args = Args::parse();

    if args.output.is_empty() {
        args.output = args.input.clone();
    }

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = slog::Logger::root(drain, o!());

    info!(log, "Initialization complete.");
    let (prices, all_names, all_manufacturer_names) = read_all_price_data(&args.input, args.debug)?;
    info!(log, "All data read.");

    let names = get_canonical_names(&all_names);
    info!(log, "Canonical names obtained.");

    let manufacturer_names = get_canonical_names(&all_manufacturer_names);
    info!(log, "Canonical manufacturer names obtained.");

    update_store_data_in_place(&args.input, &names, &manufacturer_names, &log)?;
    info!(log, "Store data updated in place.");

    write_all_price_data(prices, &args.output)?;
    info!(log, "Wrote all prices data.");

    info!(log, "Complete.");
    Ok(())
}
fn main() {
    if let Err(err) = run() {
        println!("Error: {err}");
    }
}
