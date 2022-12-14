use serde::Deserialize;
use serde::Serialize;

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

#[derive(Debug, Default, Serialize)]
pub struct Prices {
    pub chain_id: ChainId,
    pub subchain_id: SubchainId,
    pub store_id: StoreId,
    pub verification_num: i32,
    pub items: Vec<Item>,
}

#[derive(Debug, Default, Serialize)]
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

#[derive(Debug, Default, Serialize)]
pub struct Subchain {
    pub subchain_id: SubchainId,
    pub subchain_name: String,
    pub stores: Vec<Store>,
}

#[derive(Debug, Default, Serialize)]
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
