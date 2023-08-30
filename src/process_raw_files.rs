use std::collections::HashMap;

use anyhow::{Context, Result};
use tracing::{debug, info};

use crate::{
    counter::{self, DataCounter},
    models::{self, ItemInfo, ItemKey},
    sanitization, xml_to_standard,
};

pub struct ProcessedData {
    pub chains: Vec<models::Chain>,
    pub item_infos: HashMap<ItemKey, ItemInfo>,
}

pub fn process_raw_files(dir: &str, store_to_filter: &str) -> Result<ProcessedData> {
    let mut chains: Vec<models::Chain> = Vec::new();
    let mut prices: Vec<models::Prices> = Vec::new();

    info!("Starting processing of files");
    let paths = walkdir::WalkDir::new(std::path::Path::new(&dir))
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|d| d.into_path())
        .filter(|path| path.is_file())
        .filter_map(|path| path.to_str().map(|s| s.to_owned()))
        .filter(|path| !path.ends_with(".gz"))
        .filter(|path| store_to_filter.is_empty() || path.contains(&store_to_filter));

    let (price_paths, non_price_paths): (Vec<String>, Vec<String>) = paths.partition(|path| {
        let filename = path.rsplit_once("/").unwrap().1;
        filename.starts_with("Price") || filename.starts_with("price")
    });
    let (stores_paths, other_paths): (Vec<String>, Vec<String>) =
        non_price_paths.into_iter().partition(|path| {
            let filename = path.rsplit_once("/").unwrap().1;
            filename.starts_with("Store") || filename.starts_with("store")
        });

    info!(
        "There are {} stores files, {} prices files, and {} other files",
        stores_paths.len(),
        price_paths.len(),
        other_paths.len()
    );
    info!("Starting to handle stores");
    for store_path in stores_paths {
        debug!("Reading file: {store_path}");
        let chain = xml_to_standard::handle_stores_file(&store_path)?;
        chains.push(chain);
    }
    info!("Writing chains.json");
    std::fs::write("chains.json", serde_json::to_string(&chains).unwrap())?;
    info!("Finished to handle stores, starting to handle prices");
    for price_path in price_paths {
        debug!("Reading file: {price_path}");
        let price = xml_to_standard::hande_price_file(&price_path)?;
        prices.push(price);
    }
    info!("Writing prices.json");
    std::fs::write("prices.json", serde_json::to_string(&prices).unwrap())?;
    info!("Finished to handle prices");

    let mut item_infos = models::ItemInfos::default();

    #[derive(Default, Debug)]
    struct AggregatedData {
        prices: Vec<models::ItemPrice>,
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

    let mut items_aggregated_data: HashMap<models::ItemKey, AggregatedData> = HashMap::new();
    info!("Starting to build Aggregated data");
    for price in prices {
        for item in price.items {
            let item_key = models::ItemKey::from_item_and_chain(&item, price.chain_id);

            let data = items_aggregated_data
                .entry(item_key)
                .or_insert(AggregatedData::default());
            data.prices.push(models::ItemPrice {
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
            models::ItemInfo {
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
    info!("Saving item_infos.json");
    std::fs::write(
        "item_infos.json",
        serde_json::to_string(&item_infos).unwrap(),
    )?;

    Ok(ProcessedData {
        chains,
        item_infos: item_infos.data,
    })
}
