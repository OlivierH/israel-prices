use std::collections::HashMap;

use anyhow::Result;
use israel_prices;
use israel_prices::models::VictoryMetadata;
use tracing_subscriber::prelude::*;

async fn fetch_am_pm() -> Result<HashMap<String, VictoryMetadata>> {
    let am_pm_metadata = israel_prices::online_store_data::fetch_victory_metadata(
        "https://www.ampm.co.il/v2/retailers/2",
        0,
    )
    .await?;
    return Ok(am_pm_metadata);
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new("israel_prices=DEBUG"));

    tracing::subscriber::set_global_default(subscriber)?;

    // let data = israel_prices::online_store_data::fetch_rami_levy_metadata()
    //     .await
    //     .unwrap();

    let data = fetch_am_pm().await?;
    println!("Found {} elements.", data.len());
    Ok(())
}
