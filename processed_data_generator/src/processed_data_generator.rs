use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    path::PathBuf,
};

use anyhow::anyhow;
use anyhow::{Ok, Result};
use clap::Parser;
use models::{ChainId, Item};
use serde::Serialize;
use slog::{self, debug, info, o, Drain, Logger};
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

fn sanitize_name(input: &str) -> String {
    let mut out = input;

    while out.starts_with('"') && out.ends_with('"') {
        out = out
            .strip_prefix('"')
            .unwrap()
            .strip_suffix('"')
            .unwrap()
            .trim();
    }
    if out.starts_with('"') && !out[1..].contains('"') {
        out = &out[1..].trim();
    }
    if out.starts_with("'") && !out[1..].contains("'") {
        out = &out[1..].trim();
    }
    while out.ends_with('!') {
        out = out.strip_suffix('!').unwrap().trim();
    }
    if out.starts_with("*מבצע*") {
        out = out.strip_prefix("*מבצע*").unwrap().trim();
    }
    while out.starts_with("*") && !out[1..].contains("*") {
        out = &out[1..];
    }
    let out_str = out.replace("\"\"", "ֿֿֿֿ\"");
    out = &out_str;

    out.trim().to_string()
}

fn read_all_price_data(input: &str, debug: bool) -> Result<HashMap<ItemKey, AggregatedData>> {
    let mut all_aggregated_data: HashMap<ItemKey, AggregatedData> = HashMap::new();
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
            let aggregated_data = all_aggregated_data
                .entry(item_key)
                .or_insert_with(|| AggregatedData::default());

            aggregated_data.prices.push(ItemPrice {
                chain_id: chain_id,
                store_id: store_id,
                price: item.item_price,
            });
            if !item.item_name.is_empty() {
                aggregated_data.names.insert(sanitize_name(&item.item_name));
            }
            if !item.manufacturer_name.is_empty() {
                aggregated_data
                    .manufacturer_names
                    .insert(item.manufacturer_name);
            }
            if !item.manufacture_country.is_empty() {
                aggregated_data.country[&item.manufacture_country] += 1;
            }
            aggregated_data.chains.insert(chain_id);
        }
    }
    Ok(all_aggregated_data)
}

fn write_all_price_data(all_data: HashMap<ItemKey, AggregatedData>, output: &str) -> Result<()> {
    let dir = std::path::Path::new(output).join("prices_per_product");
    std::fs::create_dir_all(&dir)?;
    for (key, mut data) in all_data.into_iter() {
        let mut filename = key.item_code.to_string();
        if let Some(chain_id) = key.chain_id {
            filename += &format!("_{}", chain_id);
        }
        filename += ".csv";
        data.prices.sort_by_key(|k| (k.chain_id, k.store_id));
        let mut writer = csv::Writer::from_path(dir.join(filename))?;
        for price in data.prices {
            writer.serialize(&price)?;
        }
    }

    Ok(())
}

fn write_all_product_data(all_data: &HashMap<ItemKey, Product>, output: &str) -> Result<()> {
    let dir = std::path::Path::new(output).join("products");
    std::fs::create_dir_all(&dir)?;
    println!("{}", all_data.len());

    let mut all_products_writer =
        csv::Writer::from_path(std::path::Path::new(output).join("products.csv"))?;

    all_products_writer.write_record(&[
        "item_code",
        "name",
        "manufacturer_name",
        "manufacture_country",
    ])?;

    for (key, data) in all_data.iter() {
        let mut code = key.item_code.to_string();
        if let Some(chain_id) = key.chain_id {
            code += &format!("_{}", chain_id);
        }
        all_products_writer.write_record(&[
            &code,
            &data.item_name,
            &data.manufacturer_name,
            &data.manufacture_country,
        ])?;
        serde_json::to_writer(&std::fs::File::create(dir.join(code + ".json"))?, &data)?;
    }

    Ok(())
}

#[derive(Serialize)]
struct Product {
    item_name: String,
    manufacturer_name: String,
    manufacture_country: String,
    chains: Vec<ChainId>,
}

fn get_canonical_data(
    aggregated_data: &HashMap<ItemKey, AggregatedData>,
) -> HashMap<ItemKey, Product> {
    let mut output: HashMap<ItemKey, Product> = HashMap::new();
    fn longest(strings: &HashSet<String>) -> String {
        strings
            .iter()
            .max_by(|x, y| (x.len(), x).cmp(&(y.len(), y)))
            .unwrap_or(&"".to_string())
            .to_string()
    }

    for (key, data) in aggregated_data.iter() {
        output.insert(
            *key,
            Product {
                item_name: longest(&data.names),
                manufacturer_name: longest(&data.manufacturer_names),
                manufacture_country: data
                    .country
                    .most_common()
                    .get(0)
                    .map_or("", |v| &v.0)
                    .to_string(),
                chains: Vec::from_iter(data.chains.clone()),
            },
        );
    }

    output
}

fn update_store_data_in_place(
    dir: &str,
    canonical_data: &HashMap<ItemKey, Product>,
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
        debug!(log, "Handling {}", path.as_os_str().to_str().unwrap());
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
                let item_fields = canonical_data.get(&item_key).unwrap();
                item.item_name = item_fields.item_name.to_string();
                item.manufacturer_name = item_fields.manufacturer_name.to_string();
                item.manufacture_country = item_fields.manufacture_country.to_string();
                writer.serialize(item)?;
            }
        }
        std::fs::rename(new_path, path)?;
    }
    Ok(())
}

#[derive(Default)]
struct AggregatedData {
    prices: Vec<ItemPrice>,
    names: HashSet<String>,
    manufacturer_names: HashSet<String>,
    country: counter::Counter<String>,
    chains: HashSet<models::ChainId>,
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
    let all_aggregated_data = read_all_price_data(&args.input, args.debug)?;
    info!(log, "All data read.");

    let all_canonical_data = get_canonical_data(&all_aggregated_data);
    info!(log, "Canonical data obtained.");

    update_store_data_in_place(&args.input, &all_canonical_data, &log)?;
    info!(log, "Store data updated in place.");

    write_all_product_data(&all_canonical_data, &args.output)?;
    info!(log, "Wrote all product data.");

    write_all_price_data(all_aggregated_data, &args.output)?;
    info!(log, "Wrote all prices data.");

    info!(log, "Complete.");
    Ok(())
}
fn main() {
    if let Err(err) = run() {
        println!("Error: {err}");
    }
}
