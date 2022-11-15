use std::collections::HashMap;

use anyhow::{Ok, Result};
use clap::Parser;
use models::Item;

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(short, long, default_value = "../data")]
    input: String,

    #[arg(short, long, default_value = "")]
    output: String,
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash)]

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

fn read_all_price_data(input: &str) -> Result<HashMap<ItemKey, Vec<ItemPrice>>> {
    let mut data: HashMap<ItemKey, Vec<ItemPrice>> = HashMap::new();
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

            data.entry(ItemKey {
                chain_id: match item.internal_code {
                    true => Some(chain_id),
                    false => None,
                },
                item_code: item.item_code,
            })
            .or_insert_with(|| Vec::new())
            .push(ItemPrice {
                chain_id: chain_id,
                store_id: store_id,
                price: item.item_price,
            });
        }
    }
    Ok(data)
}

fn write_all_price_data(data: HashMap<ItemKey, Vec<ItemPrice>>, output: &str) -> Result<()> {
    let dir = std::path::Path::new(output).join("prices_per_product");
    std::fs::create_dir_all(&dir)?;
    for (key, mut prices) in data.into_iter() {
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

fn run() -> Result<()> {
    let mut args = Args::parse();

    if args.output.is_empty() {
        args.output = args.input.clone();
    }

    let data = read_all_price_data(&args.input)?;

    write_all_price_data(data, &args.output)?;
    Ok(())
}
fn main() {
    if let Err(err) = run() {
        println!("Error: {err}");
    }
}
