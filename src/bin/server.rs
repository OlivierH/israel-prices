use anyhow::{anyhow, Result};
use askama::Template;
use axum::{
    extract,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use chrono::format::Item;
use israel_prices::models;
use rusqlite::{params, Connection};
use serde::Deserialize;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("server=debug"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/compare/:store_1/:store_2", get(compare))
        .route("/", get(index))
        .route("/stores", get(stores))
        .route("/search/:query", get(search))
        .route("/store/:chain_id/:store_id", get(store));

    tracing::debug!("listening on http://0.0.0.0:3000");
    let _ = axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn connection() -> Result<Connection> {
    let path = "data.sqlite";
    Ok(rusqlite::Connection::open(path)?)
}

async fn index() -> Result<impl IntoResponse, AppError> {
    Ok(Html(
        std::fs::read_to_string("templates/index.html").unwrap_or("Error".to_string()),
    ))
}
async fn stores() -> Result<impl IntoResponse, AppError> {
    let connection = connection()?;
    let mut stmt = connection.prepare("SELECT subchains.ChainId, subchains.SubchainId, ChainName, SubchainName, StoreName, StoreId, City FROM Stores JOIN Subchains on Stores.chainId = Subchains.chainId AND Stores.subchainid = Subchains.subchainid")?;
    #[derive(Debug)]
    struct StoreRow {
        chain_id: i64,
        subchain_id: i64,
        chain_name: String,
        subchain_name: String,
        store_name: String,
        store_id: i64,
        city: String,
    }
    let mut result = stmt.query(())?;
    let mut stores = Vec::new();
    while let Some(row) = result.next()? {
        stores.push(StoreRow {
            chain_id: row.get(0)?,
            subchain_id: row.get(1)?,
            chain_name: row.get(2)?,
            subchain_name: row.get(3)?,
            store_name: row.get(4)?,
            store_id: row.get(5)?,
            city: row.get(6)?,
        });
    }
    #[derive(Template)]
    #[template(path = "stores.html")]
    struct StoresTemplate {
        stores: Vec<StoreRow>,
    }
    let template = StoresTemplate { stores };

    Ok(HtmlTemplate(template))
}

async fn search(
    extract::Path(query): extract::Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let connection = connection()?;
    let mut stmt = connection.prepare(
        "
    SELECT 
        items.itemname, prices.chainid, prices.storeid, subchains.chainname
    FROM prices JOIN items JOIN subchains
    ON prices.itemcode = items.itemcode AND prices.chainid = subchains.chainid
    WHERE items.itemname LIKE ?1;
    ",
    )?;
    let s = format!("%{query}%");
    let mut result: rusqlite::Rows<'_> = stmt.query(params![s])?;

    struct ItemRecord {
        name: String,
        chain_id: i64,
        store_id: i64,
        chain_name: String,
    }
    let mut items = Vec::new();

    while let Some(row) = result.next()? {
        items.push(ItemRecord {
            name: row.get(0)?,
            chain_id: row.get(1)?,
            store_id: row.get(2)?,
            chain_name: row.get(3)?,
        });
    }
    #[derive(Template)]
    #[template(path = "search_results.html")]
    struct SearchTemplate {
        items: Vec<ItemRecord>,
    }
    let template = SearchTemplate { items };
    Ok(HtmlTemplate(template))
}

async fn store(
    extract::Path((chain_id, store_id)): extract::Path<(models::ChainId, models::StoreId)>,
) -> Result<impl IntoResponse, AppError> {
    let connection = connection()?;
    let mut stmt = connection.prepare("
    SELECT 
        subchains.SubchainId, ChainName, SubchainName, StoreName, City 
    FROM Stores JOIN Subchains on Stores.chainId = Subchains.chainId AND Stores.subchainid = Subchains.subchainid
    WHERE Stores.chainId = ?1 AND StoreId = ?2   
    ")?;
    let mut result: rusqlite::Rows<'_> = stmt.query(params![chain_id, store_id])?;
    let row = result.next()?.ok_or(anyhow!("No such store found"))?;

    let subchain_id: models::SubchainId = row.get(0)?;
    let chain_name: String = row.get(1)?;
    let subchain_name: String = row.get(2)?;
    let store_name: String = row.get(3)?;
    let city: String = row.get(4)?;

    let mut stmt = connection.prepare("
    SELECT items.itemname, prices.itemprice 
    FROM prices JOIN items 
    ON prices.itemcode = items.itemcode
    WHERE prices.chainid = ?1 AND (items.chainid is null or items.chainid = ?1) AND prices.storeid = ?2")?;
    let mut result = stmt.query(params![chain_id, store_id])?;
    let mut items = Vec::new();
    struct Item {
        price: String,
        name: String,
    }
    while let Some(row) = result.next()? {
        items.push(Item {
            name: row.get(0)?,
            price: row.get(1)?,
        });
    }

    #[derive(Template)]
    #[template(path = "store.html")]
    struct StoreTemplate {
        chain_id: models::ChainId,
        subchain_id: models::SubchainId,
        chain_name: String,
        subchain_name: String,
        store_id: models::StoreId,
        store_name: String,
        city: String,
        items: Vec<Item>,
    }
    let template = StoreTemplate {
        chain_id,
        subchain_id,
        chain_name,
        subchain_name,
        store_id,
        store_name,
        city,
        items,
    };

    Ok(HtmlTemplate(template))
}

#[derive(Deserialize)]
struct CompareParams {
    name1: String,
    name2: String,
}

async fn compare(
    name_params: extract::Query<CompareParams>,
    extract::Path((store_1, store_2)): extract::Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let mut store_1 = store_1.split("_");
    let chain_id_1 = store_1.next().ok_or(anyhow!("Error parsing param"))?;
    let _subchain_id_1 = store_1.next().ok_or(anyhow!("Error parsing param"))?;
    let store_id_1 = store_1.next().ok_or(anyhow!("Error parsing param"))?;

    let mut store_2 = store_2.split("_");
    let chain_id_2 = store_2.next().ok_or(anyhow!("Error parsing param"))?;
    let _subchain_id_2 = store_2.next().ok_or(anyhow!("Error parsing param"))?;
    let store_id_2 = store_2.next().ok_or(anyhow!("Error parsing param"))?;

    info!("Parsing finished. comparing ({chain_id_1}, {store_id_1}) with ({chain_id_2}, {store_id_2})");

    let connection = connection()?;
    let mut stmt = connection.prepare(
        "
    select 
        avg(ratio) as avg, count(*) as cnt from (
            select cast(price_1 as float) / cast(price_2 as float) as ratio from (
                select 
                    itemcode, 
                    MAX(CASE WHEN storeid = ?1 and chainid = ?2 THEN ItemPrice END) as price_1,
                    MAX(CASE WHEN storeid = ?3 and chainid = ?4 then ItemPrice END) as price_2
                FROM PRICES 
                WHERE (storeid = ?1 and chainid = ?2) or (storeid = ?3 and chainid = ?4)
                GROUP BY itemcode
            ) 
            where 
                price_1 is not null and
                price_2 is not null and 
                ratio is not null
        );",
    )?;
    let mut result = stmt.query(params![store_id_1, chain_id_1, store_id_2, chain_id_2])?;
    let row = result.next()?.ok_or(anyhow!("Failure during query"))?;
    let avg: f64 = row.get(0)?;
    let cnt: usize = row.get(1)?;

    #[derive(Template)]
    #[template(path = "compare.html")]
    struct CompareTemplate {
        avg: f64,
        cnt: usize,
        name_params: CompareParams,
    }
    let template = CompareTemplate {
        avg,
        cnt,
        name_params: name_params.0,
    };

    Ok(HtmlTemplate(template))
}

/* Error handling magic */
// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}
