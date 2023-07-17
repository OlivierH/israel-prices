use anyhow::Result;
use tracing::info;
use tracing_subscriber::prelude::*;

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("annotate_tags=debug"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting");

    let mut item_infos = israel_prices::models::ItemInfos::default();

    {
        let item_infos_file = std::io::BufReader::new(std::fs::File::open("item_infos.json")?);
        info!("Reading item_infos from item_infos.json");
        item_infos = serde_json::from_reader(item_infos_file)?;
        info!(
            "Read {} item_infos from item_infos.json",
            item_infos.data.len()
        );
    }

    for (item_key, item_info) in item_infos.data {
        let item_info: israel_prices::models::ItemInfo = item_info;
    }

    Ok(())
}
