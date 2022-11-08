use futures::StreamExt;
use roxmltree::Node;
use serde::Serialize;
use std::collections::HashSet;
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

fn to_country_code(n: &roxmltree::Node) -> String {
    let mut s = to_string(n);
    if let Some(country_code) = country_code::to_country_code(&s) {
        s = country_code.to_string();
    }
    s
}

fn to_string(n: &roxmltree::Node) -> String {
    let mut s = match n.text().unwrap_or("") {
        "לא ידוע" | "כללי" | "," => return "".to_string(),
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

fn hande_price_file(path: &Path) -> std::io::Result<()> {
    let contents = {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        contents
    };

    let doc = roxmltree::Document::parse(&contents).unwrap();

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

    assert!(!prices.chain_id.is_empty());
    assert!(!prices.subchain_id.is_empty());
    assert!(!prices.store_id.is_empty());
    assert!(!prices.verification_num.is_empty());

    doc.descendants()
        .filter(|n| {
            n.tag_name().name() == "Item"
                || n.tag_name().name() == "Product"
                || n.tag_name().name() == "Line"
        })
        // .take(5)
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

fn handle_file(path: &str) -> std::io::Result<()> {
    println!("Handling file {path}");
    let path = Path::new(&path);
    let filename = path.file_name().unwrap();
    if filename.to_str().unwrap().starts_with("PriceFull") {
        hande_price_file(path)?;
    } else if filename.to_str().unwrap().starts_with("Stores") {
    } else {
        panic!("{}", filename.to_str().unwrap());
    }
    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut dirs = HashSet::new();

    let tasks: Vec<_> = args
        .into_iter()
        .filter(|arg| {
            let path = Path::new(&arg);
            let store = path
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();
            dirs.insert(store.to_string())
        })
        .map(|arg| {
            tokio::spawn(async move {
                handle_file(&arg);
            })
        })
        .collect();
    for task in tasks {
        task.await.unwrap();
    }
}
