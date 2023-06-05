use anyhow::Result;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    let data = israel_prices::online_store_data::fetch_rami_levy_metadata()
        .await
        .unwrap();

    println!("Found {} rami levy elements.", data.len());
    Ok(())
}
