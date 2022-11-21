use std::collections::{HashMap, HashSet};

use anyhow::{Ok, Result};
use clap::Parser;
use models::Item;

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(short, long, default_value = "../data")]
    input: String,

    #[arg(short, long, default_value = "")]
    output: String,

    #[arg(short, long, default_value = "false")]
    debug: bool,
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
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

fn read_all_price_data(
    input: &str,
) -> Result<(
    HashMap<ItemKey, Vec<ItemPrice>>,
    HashMap<ItemKey, HashSet<String>>,
)> {
    let mut prices: HashMap<ItemKey, Vec<ItemPrice>> = HashMap::new();
    let mut names: HashMap<ItemKey, HashSet<String>> = HashMap::new();
    let paths = walkdir::WalkDir::new(std::path::Path::new(input).join("prices"))
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|dir| dir.into_path())
        .filter(|path| path.is_file());

    for path in paths {
        let (chain_id, store_id) = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .split_once(".")
            .unwrap()
            .0
            .split_once("_")
            .unwrap();
        let chain_id = chain_id.parse::<i64>().unwrap();
        let store_id = store_id.parse::<i32>().unwrap();
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
                .entry(item_key.clone())
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
        }
    }
    Ok((prices, names))
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

fn write_all_product_data(names: HashMap<ItemKey, HashSet<String>>, output: &str) -> Result<()> {
    println!(
        "Starting to write product data, got {} elements",
        names.len()
    );
    for (key, names) in names.into_iter() {
        if names.len() > 5 {
            println!("-----");
            for name in names {
                println!("  {name}");
            }
        }
    }

    Ok(())
}

fn run() -> Result<()> {
    let mut args = Args::parse();

    if args.output.is_empty() {
        args.output = args.input.clone();
    }

    let (prices, names) = read_all_price_data(&args.input)?;

    if args.debug {
        write_all_product_data(names, &args.output)?;
        return Ok(());
    }
    write_all_price_data(prices, &args.output)?;
    // write_all_product_data(names, &args.output)?;
    Ok(())
}
fn main() {
    if let Err(err) = run() {
        println!("Error: {err}");
    }
}
