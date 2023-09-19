use std::collections::HashMap;

use anyhow::{Context, Result};
use multimap::MultiMap;
use rusqlite::{params, Connection};
use tracing::{debug, info, instrument};

use crate::{
    models::{
        Barcode, Chain, ItemInfo, ItemKey, RamiLevyMetadata, ScrapedData, ShufersalMetadata,
        VictoryMetadata, YochananofMetadata,
    },
    tags::Tag,
};

fn connection() -> Result<Connection> {
    let path = "data.sqlite";
    Ok(rusqlite::Connection::open(path)?)
}

pub fn save_shufersal_metadata_to_sqlite(
    shufersal_metadata: &HashMap<Barcode, ShufersalMetadata>,
) -> Result<()> {
    let mut connection = connection()?;

    info!("Saving table ShufersalMetadata to sqlite");
    connection.execute(
        "CREATE TABLE IF NOT EXISTS ShufersalMetadata (
                        ItemCode TEXT NOT NULL PRIMARY KEY,
                        Categories TEXT,
                        NutritionInfo TEXT,
                        Ingredients TEXT,
                        ProductSymbols TEXT,
                        ImageUrl TEXT )",
        (),
    )?;
    let transaction = connection.transaction()?;
    {
        let tx = &transaction;
        let mut statement = tx
            .prepare("INSERT INTO ShufersalMetadata (ItemCode, Categories, NutritionInfo, Ingredients, ProductSymbols, ImageUrl) VALUES (?1,?2,?3,?4,?5,?6)")?;
        for (item_code, metadata) in shufersal_metadata.iter() {
            statement
                .execute(params![
                    item_code,
                    metadata.categories,
                    metadata.nutrition_info,
                    metadata.ingredients,
                    metadata.product_symbols,
                    metadata.image_url,
                ])
                .with_context(|| format!("With item_code = {:?}", item_code))?;
        }
    }
    transaction.commit()?;
    Ok(())
}

pub fn save_rami_levy_metadata_to_sqlite(
    rami_levy_metadata: &HashMap<Barcode, RamiLevyMetadata>,
) -> Result<()> {
    let mut connection = connection()?;

    info!("Saving table RamiLevyMetadata to sqlite");
    connection.execute(
        "CREATE TABLE IF NOT EXISTS RamiLevyMetadata (
                        ItemCode TEXT NOT NULL PRIMARY KEY,
                        Categories TEXT,
                        NutritionInfo TEXT,
                        Ingredients TEXT,
                        ProductSymbols TEXT,
                        ImageUrlSmall TEXT,
                        ImageUrlOriginal TEXT,
                        ImageUrlTrim TEXT,
                        ImageUrlTransparent TEXT)",
        (),
    )?;
    let transaction = connection.transaction()?;
    {
        let tx = &transaction;
        let mut statement = tx
            .prepare("INSERT INTO RamiLevyMetadata (ItemCode, Categories, NutritionInfo, Ingredients, ProductSymbols, ImageUrlSmall, ImageUrlOriginal, ImageUrlTrim, ImageUrlTransparent) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)")?;
        for (item_code, metadata) in rami_levy_metadata.iter() {
            statement
                .execute(params![
                    item_code,
                    metadata.categories,
                    metadata.nutrition_info,
                    metadata.ingredients,
                    metadata.product_symbols,
                    metadata.image_url_small,
                    metadata.image_url_original,
                    metadata.image_url_trim,
                    metadata.image_url_transparent,
                ])
                .with_context(|| format!("With item_code = {:?}", item_code))?;
        }
    }
    transaction.commit()?;
    Ok(())
}

pub fn save_to_sqlite(chains: &Vec<Chain>, item_infos: &HashMap<ItemKey, ItemInfo>) -> Result<()> {
    let mut connection = connection()?;
    {
        info!("Saving table Chains to sqlite");
        connection.execute(
            "CREATE TABLE Chains (
                        ChainId int NOT NULL PRIMARY KEY,
                        ChainName TEXT);",
            (),
        )?;
        let mut statement =
            connection.prepare("INSERT INTO Chains (ChainID, ChainName) VALUES (?1,?2)")?;
        for chain in chains {
            statement.execute(params![chain.chain_id, chain.chain_name])?;
        }
    }
    {
        info!("Saving table Subchains to sqlite");
        connection.execute(
            "CREATE TABLE Subchains (
                        ChainId int NOT NULL,
                        ChainName TEXT,
                        SubchainId int NOT NULL,
                        SubchainName TEXT,
                        PRIMARY KEY(ChainId,SubChainId)) ",
            (),
        )?;
        let mut statement = connection
            .prepare("INSERT INTO Subchains (ChainID, ChainName, SubchainId, SubchainName) VALUES (?1,?2,?3,?4)")?;
        for chain in chains {
            for subchain in &chain.subchains {
                statement.execute(params![
                    chain.chain_id,
                    chain.chain_name,
                    subchain.subchain_id,
                    subchain.subchain_name
                ])?;
            }
        }
    }
    {
        info!("Saving table Stores to sqlite");
        connection.execute(
            "CREATE TABLE Stores (
                        ChainId int NOT NULL,
                        SubchainId int NOT NULL,
                        StoreId int NOT NULL,
                        StoreType TEXT,
                        StoreName TEXT,
                        Address TEXT,
                        City TEXT,
                        ZipCode TEXT,
                        PRIMARY KEY(ChainId,SubChainId,StoreId)) ",
            (),
        )?;
        let mut statement = connection
            .prepare("INSERT INTO Stores (ChainId,SubchainId, StoreId, StoreType, StoreName, Address, City, ZipCode) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)")?;
        for chain in chains {
            for subchain in &chain.subchains {
                for store in &subchain.stores {
                    statement.execute(params![
                        chain.chain_id,
                        subchain.subchain_id,
                        store.store_id,
                        store.store_type,
                        store.store_name,
                        store.address,
                        store.city,
                        store.zip_code,
                    ])?;
                }
            }
        }
    }
    {
        info!("Saving table Items to sqlite");
        connection.execute(
            "CREATE TABLE Items (
                        ChainId int,
                        ItemCode int NOT NULL,
                        ItemName TEXT,
                        ManufactureName TEXT,
                        ManufactureCountry TEXT,
                        ManufactureItemDescription TEXT,
                        UnitQuantity TEXT,
                        Quantity TEXT,
                        UnitOfMeasure TEXT,
                        IsWeighted TEXT,
                        QuantityInPackage TEXT,
                        Tags TEXT,
                        PRIMARY KEY(ChainId, ItemCode)) ",
            (),
        )?;
        let transaction = connection.transaction()?;
        {
            let tx = &transaction;
            let mut statement = tx.prepare(
                "INSERT INTO Items (
                    ChainId,
                    ItemCode,
                    ItemName,
                    ManufactureName,
                    ManufactureCountry,
                    ManufactureItemDescription,
                    UnitQuantity,
                    Quantity,
                    UnitOfMeasure,
                    IsWeighted,
                    QuantityInPackage) VALUES (?,?,?,?,?,?,?,?,?,?,?)",
            )?;
            for (item_key, item_info) in item_infos {
                statement
                    .execute(params![
                        item_key.chain_id,
                        item_key.item_code,
                        item_info.item_name,
                        item_info.manufacturer_name,
                        item_info.manufacture_country,
                        item_info.manufacturer_item_description,
                        item_info.unit_qty,
                        item_info.quantity,
                        item_info.unit_of_measure,
                        item_info.b_is_weighted,
                        item_info.qty_in_package
                    ])
                    .with_context(|| format!("With item_key = {:?}", item_key))?;
            }
        }
        transaction.commit()?;
    }
    {
        info!("Saving table Prices to sqlite");
        connection
            .execute(
                "CREATE TABLE Prices (
                        ChainId int NOT NULL,
                        StoreId int NOT NULL,
                        ItemCode TEXT,
                        ItemPrice TEXT,
                        UnitOfMeasurePrice TEXT,
                        PRIMARY KEY(ChainId, StoreId, ItemCode)) ",
                (),
            )
            .context("when creating table Prices")?;
        info!("Finished saving table Prices to sqlite");
        let transaction = connection.transaction()?;
        {
            let tx = &transaction;
            let mut statement = tx
            .prepare("INSERT INTO Prices (ChainID, StoreId, ItemCode, ItemPrice, UnitOfMeasurePrice) VALUES (?1,?2,?3,?4,?5)")?;
            for (item_key, item_info) in item_infos {
                for price in &item_info.prices {
                    statement
                        .execute(params![
                            price.chain_id,
                            price.store_id,
                            item_key.item_code,
                            price.price,
                            price.unit_of_measure_price
                        ])
                        .with_context(|| {
                            format!("With item_key = {:?}, price = {:?}", item_key, price)
                        })?;
                }
            }
        }
        transaction.commit().context(format!(
            "Commiting transaction of saving prices to sqlite: {}:{}",
            file!(),
            line!()
        ))?;
    }
    Ok(())
}

pub fn save_victory_metadata_to_sqlite(
    table_prefix: &str,
    victory_metadata: &HashMap<String, VictoryMetadata>,
) -> Result<()> {
    let mut connection = connection()?;

    info!("Saving table {table_prefix}Metadata to sqlite");
    connection.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {table_prefix}Metadata (
                        ItemCode TEXT NOT NULL PRIMARY KEY,
                        Categories TEXT,
                        NutritionInfo TEXT,
                        Ingredients TEXT,
                        ImageUrl TEXT)"
        ),
        (),
    )?;

    let transaction = connection.transaction()?;
    {
        let tx = &transaction;
        let mut statement = tx
            .prepare(&format!("INSERT INTO {table_prefix}Metadata (ItemCode, Categories, NutritionInfo, Ingredients, ImageUrl) VALUES (?1,?2,?3,?4,?5)"))?;
        for (item_code, metadata) in victory_metadata.iter() {
            let categories = match metadata.categories.as_ref() {
                Some(v) => Some(serde_json::to_string(&v)?),
                None => None,
            };
            let nutrition_info = match metadata.nutrition_info.as_ref() {
                Some(n) => Some(serde_json::to_string(&n)?),
                None => None,
            };
            statement
                .execute(params![
                    item_code,
                    categories,
                    nutrition_info,
                    metadata.ingredients,
                    metadata.image_url,
                ])
                .with_context(|| format!("With item_code = {:?}", item_code))?;
        }
    }
    transaction.commit()?;
    Ok(())
}

pub fn save_yochananof_metadata_to_sqlite(
    yochananof_metadata: &HashMap<String, YochananofMetadata>,
) -> Result<()> {
    let mut connection = connection()?;

    info!("Saving table YochananofMetadata to sqlite");
    connection.execute(
        "CREATE TABLE IF NOT EXISTS YochananofMetadata (
                        ItemCode TEXT NOT NULL PRIMARY KEY,
                        NutritionInfo TEXT,
                        Ingredients TEXT,
                        ImageUrl TEXT)",
        (),
    )?;

    let transaction = connection.transaction()?;
    {
        let tx = &transaction;
        let mut statement = tx
            .prepare(&format!("INSERT INTO YochananofMetadata (ItemCode, NutritionInfo, Ingredients, ImageUrl) VALUES (?1,?2,?3,?4)"))?;
        for (item_code, metadata) in yochananof_metadata.iter() {
            let nutrition_info = match metadata.nutrition_info.is_empty() {
                false => Some(serde_json::to_string(&metadata.nutrition_info)?),
                true => None,
            };
            statement
                .execute(params![
                    item_code,
                    nutrition_info,
                    metadata.ingredients,
                    metadata.image_url,
                ])
                .with_context(|| format!("With item_code = {:?}", item_code))?;
        }
    }
    transaction.commit()?;
    Ok(())
}
pub fn save_scraped_data_to_sqlite(data: &Vec<ScrapedData>) -> Result<()> {
    let mut connection: Connection = connection()?;

    info!("Saving table ScrapedData to sqlite");
    connection.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS ScrapedData (
                        Source TEXT NOT NULL,
                        ItemCode TEXT NOT NULL,
                        Categories TEXT,
                        NutritionInfo TEXT,
                        Ingredients TEXT,
                        ImageUrls TEXT,
                        PRIMARY KEY(Source,ItemCode))"
        ),
        (),
    )?;

    let transaction = connection.transaction()?;
    {
        let tx = &transaction;
        let mut statement = tx
            .prepare(&format!("INSERT OR REPLACE INTO ScrapedData (Source, ItemCode, Categories, NutritionInfo, Ingredients, ImageUrls) VALUES (?1,?2,?3,?4,?5,?6)"))?;
        for elem in data {
            let categories = match elem.categories.is_empty() {
                false => Some(serde_json::to_string(&elem.categories)?),
                true => None,
            };
            let nutrition_info = match elem.nutrition_info.is_empty() {
                false => Some(serde_json::to_string(&elem.nutrition_info)?),
                true => None,
            };
            let image_urls = match elem.image_urls.is_empty() {
                false => Some(serde_json::to_string(&elem.image_urls)?),
                true => None,
            };
            statement
                .execute(params![
                    elem.source,
                    elem.barcode,
                    categories,
                    nutrition_info,
                    elem.ingredients,
                    image_urls,
                ])
                .with_context(|| format!("With item_code = {:?}", elem.barcode))?;
        }
    }
    transaction.commit()?;
    Ok(())
}

#[instrument(skip_all)]
pub fn save_tags_to_sqlite(tags: &MultiMap<ItemKey, Tag>) -> Result<()> {
    let mut connection: Connection = connection()?;
    info!("Saving table Tags to sqlite");
    connection.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS Tags (
                        ChainId int,
                        ItemCode TEXT NOT NULL,
                        Tags TEXT,
                        PRIMARY KEY(ChainId,ItemCode))"
        ),
        (),
    )?;
    let transaction = connection.transaction()?;
    {
        let tx = &transaction;
        let mut statement = tx.prepare(&format!(
            "INSERT OR REPLACE INTO Tags (ChainId, ItemCode, Tags) VALUES (?1,?2,?3)"
        ))?;
        for (item_key, tags_vec) in tags.iter_all() {
            let tags_str = serde_json::to_string(tags_vec)?;
            statement
                .execute(params![item_key.chain_id, item_key.item_code, tags_str,])
                .with_context(|| format!("With item_key = {:?}", item_key))?;
        }
    }
    transaction.commit()?;
    Ok(())
}

#[instrument]
pub fn get_codes_from_chain_id(chain_id: i64) -> Result<Vec<Barcode>> {
    let connection: Connection = connection()?;
    debug!("get_codes_from_chain_id");
    let mut stmt = connection
        .prepare("select distinct(itemcode) from prices where chainid = ?1")
        .context("get_codes_from_chain_id")?;
    let rows = stmt.query_map(params![chain_id], |row| row.get::<_, String>(0))?;
    let mut codes = Vec::new();
    for code in rows {
        codes.push(code?.parse()?);
    }
    debug!("get_codes_from_chain_id done");
    Ok(codes)
}

pub fn maybe_delete_database(should_delete: bool) -> Result<()> {
    if should_delete {
        info!("Deleting data.sqlite");
        let path = std::path::Path::new("data.sqlite");
        if !path.exists() {
            info!("data.sqlite doesn't exist already");
        } else {
            std::fs::remove_file("data.sqlite")?;
        }
    }
    Ok(())
}
