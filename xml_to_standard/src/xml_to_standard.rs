use roxmltree::{Document, Node};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
mod country_code;
mod xml;
use anyhow::bail;
use anyhow::Result;
use clap::Parser;
use tokio;
use walkdir::WalkDir;

fn is_false(b: &bool) -> bool {
    return !b;
}

#[derive(Debug, Default, Serialize)]
struct Item {
    #[serde(rename(serialize = "code"))]
    item_code: i64,
    #[serde(skip_serializing_if = "is_false")]
    internal_code: bool,
    #[serde(rename(serialize = "name"))]
    item_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    manufacturer_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    manufacture_country: String,
    manufacturer_item_description: String,
    unit_qty: String,
    quantity: String,
    unit_of_measure: String,
    #[serde(rename(serialize = "weighted"), skip_serializing_if = "is_false")]
    b_is_weighted: bool,
    qty_in_package: String,
    #[serde(rename(serialize = "price"))]
    item_price: String,
    unit_of_measure_price: String,
    #[serde(skip_serializing_if = "is_false")]
    allow_discount: bool,
    #[serde(rename(serialize = "status"))]
    item_status: i8,
    #[serde(skip_serializing_if = "String::is_empty")]
    item_id: String,

    #[serde(skip_serializing)]
    price_update_date: String,
    #[serde(skip_serializing)]
    last_update_date: String,
    #[serde(skip_serializing)]
    last_update_time: String,
}

#[derive(Debug, Default, Serialize)]
struct Prices {
    chain_id: String,
    subchain_id: String,
    store_id: String,
    verification_num: String,
    items: Vec<Item>,
}

#[derive(Debug, Default, Serialize)]
struct Store {
    store_id: i32,
    verification_num: i32,
    store_type: String,
    store_name: String,
    address: String,
    city: String,
    zip_code: String,
}

#[derive(Debug, Default)]
struct FullStore {
    store: Store,
    chain_id: String,
    chain_name: String,
    subchain_id: i32,
    subchain_name: String,
}

#[derive(Debug, Default, Serialize)]
struct Subchain {
    subchain_id: i32,
    subchain_name: String,
    stores: Vec<Store>,
}

#[derive(Debug, Default, Serialize)]
struct Chain {
    chain_id: String,
    chain_name: String,
    subchains: Vec<Subchain>,
}

#[derive(Debug, Serialize)]
struct SubchainRecord {
    chain_id: String,
    chain_name: String,
    subchain_id: i32,
    subchain_name: String,
}

fn validate_chain(chain: &Chain) {
    assert!(!chain.chain_id.is_empty());
    assert!(!chain.subchains.is_empty());
    for subchain in &chain.subchains {
        for store in &subchain.stores {
            assert_ne!(store.store_id, 0)
        }
    }
}

fn to_full_store(node: &roxmltree::Node, path: &str) -> Result<FullStore> {
    let mut full_store = FullStore::default();

    for elem in node.children().filter(Node::is_element) {
        match elem.tag_name().name() {
            "ChainID" => {
                let mut chain_id = xml::to_string(&elem);
                if chain_id == "7290058103393" {
                    // Victory inconsistency
                    chain_id = "7290696200003".to_string();
                }
                full_store.chain_id = chain_id;
            }
            "SubChainID" | "SUBCHAINID" => full_store.subchain_id = xml::to_i32(&elem)?,
            "ChainName" | "CHAINNAME" => full_store.chain_name = xml::to_string(&elem),
            "SubChainName" | "SUBCHAINNAME" => full_store.subchain_name = xml::to_string(&elem),
            "StoreID" | "STOREID" | "StoreId" => full_store.store.store_id = xml::to_i32(&elem)?,
            "BikoretNo" | "BIKORETNO" => full_store.store.verification_num = xml::to_i32(&elem)?,
            "StoreType" | "STORETYPE" => full_store.store.store_type = xml::to_string(&elem),
            "StoreName" | "STORENAME" => full_store.store.store_name = xml::to_string(&elem),
            "Address" | "ADDRESS" => full_store.store.address = xml::to_string(&elem),
            "City" | "CITY" => full_store.store.city = xml::to_string(&elem),
            "ZIPCode" | "ZIPCODE" | "ZipCode" => full_store.store.zip_code = xml::to_string(&elem),
            "LastUpdateDate" | "LastUpdateTime" => (),
            "Latitude" | "Longitude" => (), // These would be interesting, but are never set.
            unknown => panic!("Unknown field: {unknown} in file {path}"), // TODO: do not panic in prod
        }
    }
    Ok(full_store)
}

fn hande_price_file(path: &Path, args: &Args) -> Result<()> {
    let contents = {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        contents
    };

    let doc = Document::parse(&contents).unwrap();

    let mut prices = Prices::default();

    doc.descendants()
        .find(|n| {
            n.tag_name().name() == "Prices"
                || n.tag_name().name().to_lowercase() == "root"
                || n.tag_name().name() == "Envelope"
        })
        .unwrap()
        .children()
        .filter(Node::is_element)
        .for_each(|elem| match elem.tag_name().name() {
            "XmlDocVersion" | "DllVerNo" => (),
            "Items" | "Products" | "Header" => (),
            "ChainId" | "ChainID" => prices.chain_id = xml::to_string(&elem),
            "SubChainId" | "SubChainID" => prices.subchain_id = xml::to_string(&elem),
            "StoreId" | "StoreID" => prices.store_id = xml::to_string(&elem),
            "BikoretNo" => prices.verification_num = xml::to_string(&elem),
            unknown => panic!("Unknown field: {unknown}"), // TODO: do not panic in prod
        });

    let items = doc
        .descendants()
        .filter(|n| {
            n.tag_name().name() == "Item"
                || n.tag_name().name() == "Product"
                || n.tag_name().name() == "Line"
        })
        .map(|n| {
            let mut item = Item::default();
            for elem in n.children().filter(Node::is_element) {
                match elem.tag_name().name() {
                    "PriceUpdateDate" => item.price_update_date = xml::to_string(&elem),
                    "ItemCode" => item.item_code = xml::to_string(&elem).parse().unwrap_or(-999),
                    "ItemType" => item.internal_code = xml::to_string(&elem) == "0",
                    "ItemName" | "ItemNm" => item.item_name = xml::to_string(&elem),
                    "ManufacturerName" | "ManufactureName" => {
                        item.manufacturer_name = xml::to_string(&elem)
                    }
                    "ManufactureCountry" => item.manufacture_country = xml::to_country_code(&elem),
                    "ManufacturerItemDescription" | "ManufactureItemDescription" => {
                        item.manufacturer_item_description = xml::to_string(&elem)
                    }
                    "UnitQty" => item.unit_qty = xml::to_string(&elem),
                    "Quantity" => item.quantity = xml::to_string(&elem),
                    "UnitOfMeasure" | "UnitMeasure" => item.unit_of_measure = xml::to_string(&elem),
                    "bIsWeighted" | "BisWeighted" | "blsWeighted" => {
                        item.b_is_weighted = xml::to_string(&elem) == "1"
                    }
                    "QtyInPackage" => item.qty_in_package = xml::to_string(&elem),
                    "ItemPrice" => item.item_price = xml::to_string(&elem),
                    "UnitOfMeasurePrice" => item.unit_of_measure_price = xml::to_string(&elem),
                    "AllowDiscount" => item.allow_discount = xml::to_string(&elem) == "1",
                    "ItemStatus" | "itemStatus" => {
                        item.item_status = xml::to_string(&elem).parse().unwrap()
                    }
                    "ItemId" => item.item_id = xml::to_string(&elem),
                    "LastUpdateDate" => item.last_update_date = xml::to_string(&elem),
                    "LastUpdateTime" => item.last_update_time = xml::to_string(&elem),
                    unknown => bail!("Unknown field: {unknown}"), // TODO: do not panic in prod
                }
            }
            Ok(item)
        });
    for item in items {
        let i = item?;
        prices.items.push(i);
    }

    prices.items.sort_by_key(|i| i.item_code);

    let file = File::create(format!(
        "{}/prices_{}_{}.json",
        &args.output, prices.chain_id, prices.store_id
    ))?;
    serde_json::to_writer_pretty(&file, &prices)?;

    Ok(())
}

fn get_chain_from_asx_values(node: Node, path: &str) -> Result<Chain> {
    let mut chain = Chain::default();

    chain.chain_id = xml::to_child_content(&node, "CHAINID")?;

    let mut subchains: HashMap<i32, Subchain> = HashMap::new();

    for elem in node
        .descendants()
        .filter(|n| n.tag_name().name() == "STORE")
    {
        let full_store = to_full_store(&elem, path)?;

        match subchains.get_mut(&full_store.subchain_id) {
            Some(subchain) => subchain,
            None => {
                subchains.insert(
                    full_store.subchain_id.clone(),
                    Subchain {
                        subchain_id: full_store.subchain_id,
                        subchain_name: full_store.subchain_name,
                        stores: vec![],
                    },
                );
                subchains.get_mut(&full_store.subchain_id).unwrap()
            }
        }
        .stores
        .push(full_store.store);
        chain.chain_name = full_store.chain_name;
    }
    chain.subchains.extend(subchains.into_values());
    Ok(chain)
}

fn get_chain_from_envelope(node: Node, path: &str) -> Result<Chain> {
    let mut chain = Chain::default();
    chain.chain_id = xml::to_child_content(&node, "ChainId")?;

    let mut subchain = Subchain::default();
    subchain.subchain_id = xml::to_child_content(&node, "SubChainId")?.parse().unwrap();

    for line in node.descendants().filter(|n| n.tag_name().name() == "Line") {
        subchain.stores.push(to_full_store(&line, path)?.store);
    }
    chain.subchains.push(subchain);
    Ok(chain)
}
fn get_chain_from_stores(node: Node, path: &str) -> Result<Chain> {
    let mut chain = Chain::default();

    let mut subchains: HashMap<i32, Subchain> = HashMap::new();
    for branch in node
        .descendants()
        .filter(|n| n.tag_name().name() == "Branch")
    {
        let full_store = to_full_store(&branch, path)?;
        let subchain = subchains
            .entry(full_store.subchain_id)
            .or_insert_with(|| Subchain {
                subchain_id: full_store.subchain_id,
                subchain_name: full_store.subchain_name,
                stores: vec![],
            });
        subchain.stores.push(full_store.store);
        chain.chain_id = full_store.chain_id;
        chain.chain_name = full_store.chain_name;
    }
    chain.subchains.extend(subchains.into_values());
    Ok(chain)
}
fn get_chain_from_root(root: Node, path: &str) -> Result<Chain> {
    let mut chain = Chain::default();
    root.children()
        .filter(Node::is_element)
        .for_each(|elem| match elem.tag_name().name() {
            "XmlDocVersion" | "DllVerNo" => (),
            "LastUpdateDate" | "LastUpdateTime" => (),
            "SubChains" => (),
            "ChainId" => chain.chain_id = xml::to_string(&elem),
            "ChainName" => chain.chain_name = xml::to_string(&elem),
            unknown => panic!("Unknown field: {unknown} in file {path}"), // TODO: do not panic in prod
        });

    for node in root
        .descendants()
        .filter(|n| n.tag_name().name() == "SubChain")
    {
        let mut subchain = Subchain::default();
        for elem in node.children().filter(Node::is_element) {
            match elem.tag_name().name() {
                "Stores" => (),
                "SubChainId" => subchain.subchain_id = xml::to_i32(&elem)?,
                "SubChainName" => subchain.subchain_name = xml::to_string(&elem),
                unknown => panic!("Unknown field: {unknown} in file {path}"), // TODO: do not panic in prod
            };
        }

        for store in node
            .descendants()
            .filter(|n| n.tag_name().name() == "Store")
        {
            subchain.stores.push(to_full_store(&store, path)?.store);
        }

        chain.subchains.push(subchain);
    }
    Ok(chain)
}

// This method returs all subchains found, so that data about them can be printed if needed.
fn handle_stores_file(path: &Path, args: &Args) -> Result<Vec<SubchainRecord>> {
    let contents = {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        contents
    };

    let doc = Document::parse(&contents).unwrap();

    let mut chain = {
        if let Some(node) = xml::get_descendant(&doc, "Root") {
            get_chain_from_root(node, path.to_str().unwrap())?
        } else if let Some(node) = xml::get_descendant(&doc, "Store") {
            get_chain_from_stores(node, path.to_str().unwrap())?
        } else if let Some(node) = xml::get_descendant(&doc, "Envelope") {
            get_chain_from_envelope(node, path.to_str().unwrap())?
        } else if let Some(node) = xml::get_descendant(&doc, "values") {
            get_chain_from_asx_values(node, path.to_str().unwrap())?
        } else {
            let x = doc
                .descendants()
                .into_iter()
                .take(20)
                .filter(Node::is_element)
                .map(|x| x.tag_name().name().to_string() + ", ")
                .collect::<String>();
            panic!("{}; root: {:?}", path.to_str().unwrap(), x);
        }
    };

    chain
        .subchains
        .sort_by(|x, y| x.subchain_id.cmp(&y.subchain_id));
    for subchain in &mut chain.subchains {
        subchain.stores.sort_by(|x, y| x.store_id.cmp(&y.store_id));
    }
    validate_chain(&chain);

    match args.format.as_str() {
        "json" => {
            let file = File::create(format!("{}/stores_{}.json", &args.output, chain.chain_id))?;
            serde_json::to_writer_pretty(&file, &chain)?;
        }
        "csv" => {
            for subchain in &chain.subchains {
                let mut x = csv::Writer::from_path(Path::new(&args.output).join(format!(
                    "stores_{}_{}.csv",
                    chain.chain_id, subchain.subchain_id
                )))?;
                for store in &subchain.stores {
                    x.serialize(&store)?;
                }
            }
        }
        other => panic!("Unknown format: {other}"),
    }

    Ok(chain
        .subchains
        .iter()
        .map(|subchain| SubchainRecord {
            chain_id: chain.chain_id.clone(),
            chain_name: chain.chain_name.clone(),
            subchain_id: subchain.subchain_id.clone(),
            subchain_name: subchain.subchain_name.clone(),
        })
        .collect())
}

fn handle_file(
    path: &Path,
    args: &Args,
    subchains: &Arc<Mutex<Vec<SubchainRecord>>>,
) -> Result<()> {
    println!("Handling file {}", path.display());
    let filename = path.file_name().unwrap().to_str().unwrap();
    if filename.starts_with("Price") || filename.starts_with("price") {
        if !args.stores_only {
            hande_price_file(path, &args)?;
        }
    } else if filename.starts_with("Stores") || filename.starts_with("stores") {
        let new_records = handle_stores_file(path, &args)?;
        subchains.lock().unwrap().extend(new_records);
    } else {
        panic!("{}", filename);
    }
    Ok(())
}

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long, default_value = "")]
    output: String,

    #[arg(short, long, default_value = "json",
    value_parser = clap::builder::PossibleValuesParser::new(["json", "csv"]))]
    format: String,

    #[arg(short, long)]
    parallel: bool,

    #[arg(short, long)]
    stores_only: bool,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let mut args = Args::parse();

    if args.output.is_empty() {
        args.output = format!("data_{}", args.format);
    }
    std::fs::create_dir_all(&args.output).unwrap();

    let mut dirs = HashSet::new();

    let paths = WalkDir::new(&args.input)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|dir| dir.into_path())
        .filter(|path| path.is_file());

    let subchains: Arc<Mutex<Vec<SubchainRecord>>> = Arc::new(Mutex::new(Vec::new()));

    if args.parallel {
        let tasks: Vec<_> = paths
            .map(|path| {
                tokio::spawn({
                    let args = args.clone();
                    let subchains = subchains.clone();
                    async move { handle_file(&path, &args, &subchains) }
                })
            })
            .collect();
        for task in tasks {
            match task.await {
                Ok(Ok(())) => (),
                Ok(Err(err)) => println!("Error: {err}"),
                Err(err) => println!("Error: {err}"),
            };
        }
    } else {
        for path in paths {
            if let Err(err) = handle_file(&path, &args, &subchains) {
                println!("Error: {err}");
            }
        }
    }
    match write_subchains(&subchains, args) {
        Ok(()) => (),
        Err(err) => println!("Error writing chains : {err}"),
    }
}

fn write_subchains(subchains: &Arc<Mutex<Vec<SubchainRecord>>>, args: Args) -> Result<()> {
    if args.format != "csv" {
        return Ok(());
    }
    let mut writer = csv::Writer::from_path(Path::new(&args.output).join("chains.csv"))?;
    for record in subchains.lock().unwrap().iter() {
        writer.serialize(&record)?;
    }
    Ok(())
}
