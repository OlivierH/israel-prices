use anyhow::Result;
use tracing::info;
use tracing_subscriber::prelude::*;

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("compare_stores=debug"))
        .with(tracing_subscriber::fmt::layer())
        .init();
    let mut prices: Vec<israel_prices::models::Prices>;

    let prices_file = std::io::BufReader::new(std::fs::File::open("prices.json")?);
    info!("Reading prices from prices.json - this may take some time");
    prices = serde_json::from_reader(prices_file)?;
    info!("Read {} prices from prices.json", prices.len());

    let len = prices.len();

    for p in &mut prices {
        p.items.sort_by_key(|i| i.item_code);
    }

    for i in 0..len {
        let price_1 = &prices[i];
        for j in (i + 1)..len {
            let price_2 = &prices[j];

            // if price_1.chain_id != 7290785400000
            //     || price_1.store_id != 3
            //     || price_2.chain_id != 7290696200003
            //     || price_2.store_id != 67
            // {
            //     continue;
            // }

            let mut total: f64 = 0.0;
            let mut count: f64 = 0.0;
            println!(
                "{} - {}, {} - {}, ",
                price_1.chain_id, price_1.store_id, price_2.chain_id, price_2.store_id
            );
            let mut i1 = price_1.items.iter();
            let mut i2 = price_2.items.iter();
            'outer: loop {
                let p1 = i1.next();
                let p2 = i2.next();
                if p1.is_none() || p2.is_none() {
                    break;
                }
                let mut p1 = p1.unwrap();
                let mut p2 = p2.unwrap();
                while p1.item_code != p2.item_code {
                    if p1.item_code < p2.item_code {
                        let p = i1.next();
                        if p.is_none() {
                            break 'outer;
                        }
                        p1 = p.unwrap();
                    }
                    if p1.item_code > p2.item_code {
                        let p = i2.next();
                        if p.is_none() {
                            break 'outer;
                        }
                        p2 = p.unwrap();
                    }
                }
                if price_1.chain_id != price_2.chain_id && p1.item_code < 1000000 {
                    continue;
                }
                let ratio = p1.item_price.parse::<f64>()? / p2.item_price.parse::<f64>()?;
                if ratio.is_infinite() {
                    println!(
                        "Infinite {} - {}, {} - {},",
                        price_1.chain_id, price_1.store_id, price_2.chain_id, price_2.store_id,
                    );
                    return Ok(());
                }
                if ratio.is_nan() {
                    continue;
                }
                total += ratio;
                count += 1.0;
            }
            if total > 0.5 {
                println!(
                    "{} - {}, {} - {}, total = {total}, count = {count}, avg_ratio = {:.4}",
                    price_1.chain_id,
                    price_1.store_id,
                    price_2.chain_id,
                    price_2.store_id,
                    total / count
                );
            }
            // break;
        }
    }
    Ok(())
}
