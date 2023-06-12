use std::collections::HashMap;

use anyhow::Result;
use israel_prices;
use israel_prices::models::VictoryMetadata;
use israel_prices::online_store_data;
use tracing_subscriber::prelude::*;

async fn _fetch_am_pm() -> Result<HashMap<String, VictoryMetadata>> {
    let am_pm_metadata = israel_prices::online_store_data::fetch_victory_metadata(
        "https://www.ampm.co.il/v2/retailers/2",
        0,
    )
    .await?;
    return Ok(am_pm_metadata);
}

async fn _fetch_tiv_taam() -> Result<HashMap<String, VictoryMetadata>> {
    let data =
        online_store_data::fetch_victory_metadata("https://www.tivtaam.co.il/v2/retailers/1062", 0)
            .await?;
    println!("Found {} elements.", data.len());
    let with_image = data.iter().filter(|e| e.1.image_url.is_some()).count();
    let with_ingredients = data
        .iter()
        .filter(|e: &(&String, &VictoryMetadata)| e.1.ingredients.is_some())
        .count();
    let with_categories = data
        .iter()
        .filter(|e: &(&String, &VictoryMetadata)| e.1.categories.is_some())
        .count();
    dbg!(with_image);
    dbg!(with_ingredients);
    dbg!(with_categories);
    Ok(data)
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new("israel_prices=DEBUG"));

    tracing::subscriber::set_global_default(subscriber)?;

    // fetch_tiv_taam().await?;
    online_store_data::fetch_hatzi_hinam().await?;
    Ok(())
}
