use roxmltree::{Document, Node};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
mod country_code;
use tokio;
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

fn validate_chain(chain: &Chain) {
    assert!(!chain.chain_id.is_empty());
    assert!(!chain.subchains.is_empty());
    for subchain in &chain.subchains {
        for store in &subchain.stores {
            assert_ne!(store.store_id, 0)
        }
    }
}

fn to_country_code(n: &roxmltree::Node) -> String {
    let mut s = to_string(n);
    if let Some(country_code) = country_code::to_country_code(&s) {
        s = country_code.to_string();
    }
    s
}

fn to_string(n: &roxmltree::Node) -> String {
    let mut s = match n.text().unwrap_or("") {
        "לא ידוע" | "כללי" | "unknown" | "," => return "".to_string(),
        s => s.trim().to_string(),
    };
    s = s.replace('\u{00A0}', " "); // remove non-breaking spaces
    if s.parse::<f64>().is_ok() && s.contains(".") {
        s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    if let Ok(i) = s.parse::<i64>() {
        s = i.to_string();
    }
    s
}

fn to_i32(n: &roxmltree::Node) -> i32 {
    n.text().unwrap_or("0").parse().unwrap()
}

fn get_child_content(node: &Node, tag: &str) -> String {
    to_string(
        &node
            .children()
            .find(|elem| elem.tag_name().name() == tag)
            .unwrap(),
    )
}

fn hande_price_file(path: &Path) -> std::io::Result<()> {
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
            "ChainId" | "ChainID" => prices.chain_id = to_string(&elem),
            "SubChainId" | "SubChainID" => prices.subchain_id = to_string(&elem),
            "StoreId" | "StoreID" => prices.store_id = to_string(&elem),
            "BikoretNo" => prices.verification_num = to_string(&elem),
            unknown => panic!("Unknown field: {unknown}"), // TODO: do not panic in prod
        });

    doc.descendants()
        .filter(|n| {
            n.tag_name().name() == "Item"
                || n.tag_name().name() == "Product"
                || n.tag_name().name() == "Line"
        })
        .map(|n| {
            let mut item = Item::default();
            for elem in n.children().filter(Node::is_element) {
                match elem.tag_name().name() {
                    "PriceUpdateDate" => item.price_update_date = to_string(&elem),
                    "ItemCode" => item.item_code = to_string(&elem).parse().unwrap_or(-999),
                    "ItemType" => item.internal_code = to_string(&elem) == "0",
                    "ItemName" | "ItemNm" => item.item_name = to_string(&elem),
                    "ManufacturerName" | "ManufactureName" => {
                        item.manufacturer_name = to_string(&elem)
                    }
                    "ManufactureCountry" => item.manufacture_country = to_country_code(&elem),
                    "ManufacturerItemDescription" | "ManufactureItemDescription" => {
                        item.manufacturer_item_description = to_string(&elem)
                    }
                    "UnitQty" => item.unit_qty = to_string(&elem),
                    "Quantity" => item.quantity = to_string(&elem),
                    "UnitOfMeasure" | "UnitMeasure" => item.unit_of_measure = to_string(&elem),
                    "bIsWeighted" | "BisWeighted" | "blsWeighted" => {
                        item.b_is_weighted = to_string(&elem) == "1"
                    }
                    "QtyInPackage" => item.qty_in_package = to_string(&elem),
                    "ItemPrice" => item.item_price = to_string(&elem),
                    "UnitOfMeasurePrice" => item.unit_of_measure_price = to_string(&elem),
                    "AllowDiscount" => item.allow_discount = to_string(&elem) == "1",
                    "ItemStatus" | "itemStatus" => {
                        item.item_status = to_string(&elem).parse().unwrap()
                    }
                    "ItemId" => item.item_id = to_string(&elem),
                    "LastUpdateDate" => item.last_update_date = to_string(&elem),
                    "LastUpdateTime" => item.last_update_time = to_string(&elem),
                    unknown => panic!("Unknown field: {unknown}"), // TODO: do not panic in prod
                }
            }
            item
        })
        .for_each(|item| prices.items.push(item));

    prices.items.sort_by_key(|i| i.item_code);

    std::fs::create_dir_all("data_json").unwrap();
    let file = File::create(format!(
        "data_json/prices_{}_{}.json",
        prices.chain_id, prices.store_id
    ))?;
    serde_json::to_writer_pretty(&file, &prices)?;

    Ok(())
}

fn get_chain_from_asx_values(node: Node, path: &str) -> Chain {
    let mut chain = Chain::default();

    chain.chain_id = get_child_content(&node, "CHAINID");

    let mut subchains: HashMap<i32, Subchain> = HashMap::new();

    node.descendants()
        .filter(|n| n.tag_name().name() == "STORE")
        .for_each(|node| {
            let mut store = Store::default();
            let mut subchain_id: i32 = 0;
            let mut subchain_name = "".to_string();
            node.children().filter(Node::is_element).for_each(|elem| {
                match elem.tag_name().name() {
                    "SUBCHAINID" => subchain_id = to_i32(&elem),
                    "STOREID" => store.store_id = to_i32(&elem),
                    "BIKORETNO" => store.verification_num = to_i32(&elem),
                    "STORETYPE" => store.store_type = to_string(&elem),
                    "CHAINNAME" => chain.chain_name = to_string(&elem),
                    "SUBCHAINNAME" => subchain_name = to_string(&elem),
                    "STORENAME" => store.store_name = to_string(&elem),
                    "ADDRESS" => store.address = to_string(&elem),
                    "CITY" => store.city = to_string(&elem),
                    "ZIPCODE" => store.zip_code = to_string(&elem),
                    unknown => panic!("Unknown field: {unknown} in file {path}"), // TODO: do not panic in prod
                }
            });

            match subchains.get_mut(&subchain_id) {
                Some(subchain) => subchain,
                None => {
                    subchains.insert(
                        subchain_id.clone(),
                        Subchain {
                            subchain_id: subchain_id,
                            subchain_name,
                            stores: vec![],
                        },
                    );
                    subchains.get_mut(&subchain_id).unwrap()
                }
            }
            .stores
            .push(store);
        });
    chain.subchains.extend(subchains.into_values());
    chain
}

fn get_chain_from_envelope(node: Node, path: &str) -> Chain {
    let mut chain = Chain::default();
    chain.chain_id = get_child_content(&node, "ChainId");

    let mut subchain = Subchain::default();
    subchain.subchain_id = get_child_content(&node, "SubChainId").parse().unwrap();

    node.descendants()
        .filter(|n| n.tag_name().name() == "Line")
        .for_each(|line| {
            let mut store = Store::default();

            line.children().filter(Node::is_element).for_each(|elem| {
                match elem.tag_name().name() {
                    "ChainName" => chain.chain_name = to_string(&elem),
                    "SubChainName" => subchain.subchain_name = to_string(&elem),
                    "StoreId" => store.store_id = to_i32(&elem),
                    "BikoretNo" => store.verification_num = to_i32(&elem),
                    "StoreType" => store.store_type = to_string(&elem),
                    "StoreName" => store.store_name = to_string(&elem),
                    "Address" => store.address = to_string(&elem),
                    "City" => store.city = to_string(&elem),
                    "ZipCode" => store.zip_code = to_string(&elem),
                    "LastUpdateDate" => (),
                    "LastUpdateTime" => (),
                    unknown => panic!("Unknown field: {unknown} in file {path}"), // TODO: do not panic in prod
                }
            });
            subchain.stores.push(store)
        });
    chain.subchains.push(subchain);
    chain
}
fn get_chain_from_stores(node: Node, path: &str) -> Chain {
    let mut chain = Chain::default();

    let mut subchains: HashMap<i32, Subchain> = HashMap::new();
    node.descendants()
        .filter(|n| n.tag_name().name() == "Branch")
        .for_each(|branch| {
            let mut store = Store::default();
            let mut subchain_id: i32 = 0;
            let mut subchain_name = "".to_string();

            branch.children().filter(Node::is_element).for_each(|elem| {
                match elem.tag_name().name() {
                    "ChainID" => {
                        let mut chain_id = to_string(&elem);
                        if chain_id == "7290058103393" {
                            // Victory inconsistency
                            chain_id = "7290696200003".to_string();
                        }
                        chain.chain_id = chain_id;
                    }
                    "SubChainID" => subchain_id = to_i32(&elem),
                    "ChainName" => chain.chain_name = to_string(&elem),
                    "SubChainName" => subchain_name = to_string(&elem),
                    "StoreID" => store.store_id = to_i32(&elem),
                    "BikoretNo" => store.verification_num = to_i32(&elem),
                    "StoreType" => store.store_type = to_string(&elem),
                    "StoreName" => store.store_name = to_string(&elem),
                    "Address" => store.address = to_string(&elem),
                    "City" => store.city = to_string(&elem),
                    "ZIPCode" => store.zip_code = to_string(&elem),
                    "LastUpdateDate" => (),
                    "Latitude" | "Longitude" => (), // These would be interesting, but are never set.
                    unknown => panic!("Unknown field: {unknown} in file {path}"), // TODO: do not panic in prod
                }
            });
            match subchains.get_mut(&subchain_id) {
                Some(subchain) => subchain,
                None => {
                    subchains.insert(
                        subchain_id.clone(),
                        Subchain {
                            subchain_id: subchain_id,
                            subchain_name,
                            stores: vec![],
                        },
                    );
                    subchains.get_mut(&subchain_id).unwrap()
                }
            }
            .stores
            .push(store);
        });
    assert!(!chain.chain_id.is_empty(), "with file: {path}");
    chain.subchains.extend(subchains.into_values());
    chain
}
fn get_chain_from_root(node: Node, path: &str) -> Chain {
    let mut stores = Chain::default();
    node.children()
        .filter(Node::is_element)
        .for_each(|elem| match elem.tag_name().name() {
            "XmlDocVersion" | "DllVerNo" => (),
            "LastUpdateDate" | "LastUpdateTime" => (),
            "SubChains" => (),
            "ChainId" => stores.chain_id = to_string(&elem),
            "ChainName" => stores.chain_name = to_string(&elem),
            unknown => panic!("Unknown field: {unknown} in file {path}"), // TODO: do not panic in prod
        });

    assert!(!stores.chain_id.is_empty(), "{}", path);
    assert!(!stores.chain_name.is_empty(), "{}", path);

    node.descendants()
        .filter(|n| n.tag_name().name() == "SubChain")
        .for_each(|n| {
            let mut subchain = Subchain::default();
            n.children().filter(Node::is_element).for_each(|elem| {
                match elem.tag_name().name() {
                    "Stores" => (),
                    "SubChainId" => subchain.subchain_id = to_i32(&elem),
                    "SubChainName" => subchain.subchain_name = to_string(&elem),
                    unknown => panic!("Unknown field: {unknown} in file {path}"), // TODO: do not panic in prod
                };
            });

            n.descendants()
                .filter(|n| n.tag_name().name() == "Store")
                // .take(5)
                .map(|n| {
                    let mut store = Store::default();
                    for elem in n.children().filter(Node::is_element) {
                        match elem.tag_name().name() {
                            "StoreId" => store.store_id = to_i32(&elem),
                            "BikoretNo" => store.verification_num = to_i32(&elem),
                            "StoreType" => store.store_type = to_string(&elem),
                            "StoreName" => store.store_name = to_string(&elem),
                            "Address" => store.address = to_string(&elem),
                            "City" => store.city = to_string(&elem),
                            "ZipCode" => store.zip_code = to_string(&elem),
                            unknown => panic!("Unknown field: {unknown} in file {path}"), // TODO: do not panic in prod
                        }
                    }
                    store
                })
                .for_each(|store| subchain.stores.push(store));

            stores.subchains.push(subchain);
        });
    stores
}

fn handle_stores_file(path: &Path) -> std::io::Result<()> {
    let contents = {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        contents
    };

    let doc = Document::parse(&contents).unwrap();

    let mut chain = {
        if let Some(node) = doc.descendants().find(|n| n.tag_name().name() == "Root") {
            get_chain_from_root(node, path.to_str().unwrap())
        } else if let Some(node) = doc.descendants().find(|n| n.tag_name().name() == "Store") {
            get_chain_from_stores(node, path.to_str().unwrap())
        } else if let Some(node) = doc
            .descendants()
            .find(|n| n.tag_name().name() == "Envelope")
        {
            get_chain_from_envelope(node, path.to_str().unwrap())
        } else if let Some(node) = doc.descendants().find(|n| n.tag_name().name() == "values") {
            get_chain_from_asx_values(node, path.to_str().unwrap())
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
    println!("Handling file {}", path.to_str().unwrap());

    chain
        .subchains
        .sort_by(|x, y| x.subchain_id.cmp(&y.subchain_id));
    for subchain in &mut chain.subchains {
        subchain.stores.sort_by(|x, y| x.store_id.cmp(&y.store_id));
    }
    validate_chain(&chain);

    std::fs::create_dir_all("data_json").unwrap();
    let file = File::create(format!("data_json/stores_{}.json", chain.chain_id))?;
    serde_json::to_writer_pretty(&file, &chain)?;

    Ok(())
}

fn handle_file(path: &str) -> std::io::Result<()> {
    println!("Handling file {path}");
    let path = Path::new(&path);
    let filename = path.file_name().unwrap().to_str().unwrap();
    if filename.starts_with("Price") || filename.starts_with("price") {
        hande_price_file(path)?;
    } else if filename.starts_with("Stores") || filename.starts_with("stores") {
        handle_stores_file(path)?;
    } else {
        panic!("{}", filename);
    }
    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let mut dirs = HashSet::new();

    let args = std::env::args().skip(1).filter(|arg| {
        let path = Path::new(&arg);
        let store = path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        dirs.insert(store.to_string());
        true
    });

    let parralel = true;
    if parralel {
        let tasks: Vec<_> = args
            .map(|arg| tokio::spawn(async move { handle_file(&arg) }))
            .collect();
        for task in tasks {
            match task.await {
                Ok(Ok(())) => (),
                Ok(Err(err)) => println!("Error: {err}"),
                Err(err) => println!("Error: {err}"),
            };
        }
    } else {
        for arg in args {
            handle_file(&arg);
        }
    }
}
