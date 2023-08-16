use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;

use crate::nutrition::NutritionalValues;
pub type Barcode = i64;
pub type ChainId = i64;
pub type SubchainId = i32;
pub type StoreId = i32;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Item {
    #[serde(rename = "code")]
    pub item_code: Barcode,
    pub internal_code: bool,
    #[serde(rename = "name")]
    pub item_name: String,
    pub manufacturer_name: String,
    pub manufacture_country: String,
    pub manufacturer_item_description: String,
    pub unit_qty: String,
    pub quantity: String,
    pub unit_of_measure: String,
    #[serde(rename = "weighted")]
    pub b_is_weighted: bool,
    pub qty_in_package: String,
    #[serde(rename = "price")]
    pub item_price: String,
    pub unit_of_measure_price: String,
    pub allow_discount: bool,
    #[serde(rename = "status")]
    pub item_status: i8,
    pub item_id: String,

    #[serde(skip_serializing, skip_deserializing)]
    pub price_update_date: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub last_update_date: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub last_update_time: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemPrice {
    pub chain_id: i64,
    pub store_id: i32,
    pub price: String,
    pub unit_of_measure_price: String,
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct ItemInfo {
    pub item_name: String,
    pub manufacturer_name: String,
    pub manufacture_country: String,
    pub manufacturer_item_description: String,
    pub unit_qty: String,
    pub quantity: String,
    pub unit_of_measure: String,
    pub b_is_weighted: bool,
    pub qty_in_package: String,
    pub prices: Vec<ItemPrice>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct ItemKey {
    pub item_code: Barcode,
    pub chain_id: Option<ChainId>,
}

impl ItemKey {
    pub fn from_item_and_chain(item: &Item, chain_id: ChainId) -> Self {
        ItemKey {
            item_code: item.item_code,
            chain_id: match item.internal_code {
                true => Some(chain_id),
                false => None,
            },
        }
    }
}

impl std::fmt::Display for ItemKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} <{:?}>", self.item_code, self.chain_id)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Prices {
    pub chain_id: ChainId,
    pub subchain_id: SubchainId,
    pub store_id: StoreId,
    pub verification_num: i32,
    pub items: Vec<Item>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Store {
    pub store_id: StoreId,
    pub verification_num: i32,
    pub store_type: String,
    pub store_name: String,
    pub address: String,
    pub city: String,
    pub zip_code: String,
}

#[derive(Debug, Default)]
pub struct FullStore {
    pub store: Store,
    pub chain_id: ChainId,
    pub chain_name: String,
    pub subchain_id: SubchainId,
    pub subchain_name: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Subchain {
    pub subchain_id: SubchainId,
    pub subchain_name: String,
    pub stores: Vec<Store>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Chain {
    pub chain_id: ChainId,
    pub chain_name: String,
    pub subchains: Vec<Subchain>,
}

#[derive(Debug, Serialize)]
pub struct SubchainRecord {
    pub chain_id: ChainId,
    pub chain_name: String,
    pub subchain_id: SubchainId,
    pub subchain_name: String,
}

#[serde_as]
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ItemInfos {
    #[serde_as(as = "Vec<(_, _)>")]
    pub data: HashMap<ItemKey, ItemInfo>,
}

// The following structs represent the data scrappable
// from supermarkets online websites.
#[derive(Debug, Serialize)]
pub struct ShufersalMetadata {
    pub categories: Option<String>,
    pub nutrition_info: Option<String>,
    pub ingredients: Option<String>,
    pub product_symbols: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RamiLevyMetadata {
    pub categories: Option<String>,
    pub nutrition_info: Option<String>,
    pub ingredients: Option<String>,
    pub product_symbols: Option<String>,
    pub image_url_small: Option<String>,
    pub image_url_original: Option<String>,
    pub image_url_trim: Option<String>,
    pub image_url_transparent: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VictoryMetadata {
    pub categories: Option<Vec<String>>,
    pub nutrition_info: Option<NutritionalValues>,
    pub ingredients: Option<String>,
    pub image_url: Option<String>,
    // It should be possible to extract the products symbols for Victory, but
    // I wasn't able to.
    // pub product_symbols: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct YochananofMetadata {
    // No categories yet, but it should be possible to add them
    pub nutrition_info: Vec<NutritionalValues>,
    pub ingredients: Option<String>,
    pub image_url: Option<String>,
}

pub struct ImageUrl {
    pub link: String,
    pub metadata: Option<String>,
}

pub struct ScrappedData {
    pub barcode: String, // Not always a number
    pub categories: Vec<String>,
    pub nutrition_info: Vec<NutritionalValues>,
    pub ingredients: Option<String>,
    pub image_urls: Vec<ImageUrl>,
}
