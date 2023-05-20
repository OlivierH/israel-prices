use std::collections::HashMap;

use anyhow::{Context, Result};
use rusqlite::params;
use tracing::info;

use crate::models::{Barcode, Chain, ItemInfo, ItemKey, ShufersalMetadata};

pub fn save_to_sqlite(
    chains: &Vec<Chain>,
    item_infos: &HashMap<ItemKey, ItemInfo>,
    shufersal_metadata: &Option<HashMap<Barcode, ShufersalMetadata>>,
) -> Result<()> {
    let path = "data.sqlite";
    let mut connection = rusqlite::Connection::open(path)?;
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
        connection.execute(
            "CREATE TABLE Prices (
                        ChainId int NOT NULL,
                        StoreId int NOT NULL,
                        ItemCode TEXT,
                        ItemPrice TEXT,
                        UnitOfMeasurePrice TEXT,
                        PRIMARY KEY(ChainId, StoreId, ItemCode)) ",
            (),
        )?;
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
        transaction.commit()?;
    }
    if let Some(shufersal_metadata) = shufersal_metadata {
        info!("Saving table ShufersalMetadata to sqlite");
        connection.execute(
            "CREATE TABLE ShufersalMetadata (
                        ItemCode TEXT NOT NULL PRIMARY KEY,
                        Categories TEXT,
                        NutritionInfo TEXT,
                        Ingredients TEXT,
                        ProductSymbols TEXT )",
            (),
        )?;
        let transaction = connection.transaction()?;
        {
            let tx = &transaction;
            let mut statement = tx
            .prepare("INSERT INTO ShufersalMetadata (ItemCode, Categories, NutritionInfo, Ingredients, ProductSymbols) VALUES (?1,?2,?3,?4,?5)")?;
            for (item_code, metadata) in shufersal_metadata.iter() {
                statement
                    .execute(params![
                        item_code,
                        metadata.categories,
                        metadata.nutrition_info,
                        metadata.ingredients,
                        metadata.product_symbols
                    ])
                    .with_context(|| format!("With item_code = {:?}", item_code))?;
            }
        }
        transaction.commit()?;
    }
    Ok(())
}
