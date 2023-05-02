use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

use encoding_rs::UTF_16LE;
use models::*;
use roxmltree::{Document, Node};
use std::collections::HashMap;
use std::io::prelude::*;
use std::time::Instant;
use tracing::debug;
use tracing::span;
use tracing::Level;

use crate::models;
use crate::xml;
fn validate_chain(chain: &Chain) {
    assert!(chain.chain_id > 0);
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
                let chain_id = xml::to_chain_id(&elem)?;
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

pub fn hande_price_file(path: &str) -> Result<Prices> {
    let current = Instant::now();
    let contents = read_as_utf_8(&path)?;

    let doc = Document::parse(&contents).unwrap();
    debug!(
        "It took {} ms to parse the file",
        current.elapsed().as_millis()
    );

    let mut prices: Prices = Prices::default();

    for elem in doc
        .descendants()
        .find(|n| {
            n.tag_name().name() == "Prices"
                || n.tag_name().name().to_lowercase() == "root"
                || n.tag_name().name() == "Envelope"
        })
        .unwrap()
        .children()
        .filter(Node::is_element)
    {
        match elem.tag_name().name() {
            "XmlDocVersion" | "DllVerNo" => (),
            "Items" | "Products" | "Header" => (),
            "ChainId" | "ChainID" => prices.chain_id = xml::to_chain_id(&elem)?,
            "SubChainId" | "SubChainID" => prices.subchain_id = xml::to_i32(&elem)?,
            "StoreId" | "StoreID" => prices.store_id = xml::to_i32(&elem)?,
            "BikoretNo" => prices.verification_num = xml::to_i32(&elem)?,
            unknown => panic!("Unknown field: {unknown}"), // TODO: do not panic in prod
        }
    }

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

    // prices.items.sort_by_key(|i| i.item_code);

    // match args.format.as_str() {
    //     "json" => {
    //         let file = File::create(format!(
    //             "{}/prices/{}_{}.json",
    //             &args.output, prices.chain_id, prices.store_id
    //         ))?;
    //         serde_json::to_writer_pretty(&file, &prices)?;
    //     }
    //     "csv" => {
    //         let mut writer = csv::Writer::from_path(
    //             Path::new(&args.output)
    //                 .join("prices")
    //                 .join(format!("{}_{}.csv", prices.chain_id, prices.store_id)),
    //         )?;
    //         for item in &prices.items {
    //             writer.serialize(&item)?;
    //         }
    //     }
    //     other => panic!("Unknown format: {other}"),
    // }

    Ok(prices)
}

fn get_chain_from_asx_values(node: Node, path: &str) -> Result<Chain> {
    let mut chain = Chain::default();

    chain.chain_id = xml::to_child_content(&node, "CHAINID")?.parse()?;

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
    chain.chain_id = xml::to_child_content(&node, "ChainId")?.parse()?;

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
    for elem in root.children().filter(Node::is_element) {
        match elem.tag_name().name() {
            "XmlDocVersion" | "DllVerNo" => (),
            "LastUpdateDate" | "LastUpdateTime" => (),
            "SubChains" => (),
            "ChainId" => chain.chain_id = xml::to_chain_id(&elem)?,
            "ChainName" => chain.chain_name = xml::to_string(&elem),
            unknown => panic!("Unknown field: {unknown} in file {path}"), // TODO: do not panic in prod
        }
    }

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
pub fn handle_stores_file(path: &str) -> Result<Chain> {
    let contents = read_as_utf_8(path)?;

    let doc = Document::parse(&contents).unwrap();

    let mut chain = {
        if let Some(node) = xml::get_descendant(&doc, "Root") {
            get_chain_from_root(node, path)?
        } else if let Some(node) = xml::get_descendant(&doc, "Store") {
            get_chain_from_stores(node, path)?
        } else if let Some(node) = xml::get_descendant(&doc, "Envelope") {
            get_chain_from_envelope(node, path)?
        } else if let Some(node) = xml::get_descendant(&doc, "values") {
            get_chain_from_asx_values(node, path)?
        } else {
            let x = doc
                .descendants()
                .into_iter()
                .take(20)
                .filter(Node::is_element)
                .map(|x| x.tag_name().name().to_string() + ", ")
                .collect::<String>();
            panic!("{}; root: {:?}", path, x);
        }
    };

    chain
        .subchains
        .sort_by(|x, y| x.subchain_id.cmp(&y.subchain_id));
    for subchain in &mut chain.subchains {
        subchain.stores.sort_by(|x, y| x.store_id.cmp(&y.store_id));
    }
    validate_chain(&chain);
    Ok(chain)
}

fn read_as_utf_8(path: &str) -> Result<String> {
    let span = span!(Level::DEBUG, "read_as_utf_8", path = path,);
    let _enter = span.enter();
    let mut file: std::fs::File = std::fs::File::open(&path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let out = String::from_utf8(buf.clone());
    if let Ok(s) = out {
        debug!("Using encoding utf-8");
        return Ok(s);
    }
    let (first_line, _, _) = encoding_rs::UTF_8.decode(&buf[..80]);
    if first_line.contains("encoding") {
        debug!("Found encoding in first line of file");
        let encoding = first_line
            .split_whitespace()
            .find(|s| s.starts_with("encoding"))
            .and_then(|s| s.rsplit_once('='))
            .ok_or(anyhow!("Counldn't exract encoding"))?
            .1
            .trim_matches('"');
        let encoding = encoding_rs::Encoding::for_label(encoding.as_bytes())
            .ok_or(anyhow!("Couldn't find encoding for {encoding}"))?;

        debug!("Found encoding {}", encoding.name());
        let (decoded, _, _) = encoding.decode(&buf);
        return Ok(decoded.into_owned());
    }

    debug!("Using encoding utf-16 le");
    let (decoded, _, _) = UTF_16LE.decode(&buf);
    Ok(decoded.into_owned())
}
