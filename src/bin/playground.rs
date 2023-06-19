use std::collections::HashMap;

use anyhow::{anyhow, Result};
use israel_prices;
use israel_prices::models::VictoryMetadata;
use israel_prices::models::YochananofMetadata;
use israel_prices::nutrition::NutritionalValues;
use israel_prices::online_store_data;
use israel_prices::reqwest_utils::get_to_text_with_retries;
use israel_prices::reqwest_utils::post_to_text_with_headers_with_retries;
use itertools::Itertools;
use reqwest::Client;
use scraper::Element;
use scraper::{Html, Selector};
use tracing::info;
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
fn create_selector(selectors: &str) -> Result<Selector> {
    Ok(Selector::parse(selectors).map_err(|_| anyhow!("couldn't build selector"))?)
}

async fn fetch_products_from_page_yochananof(
    url: String,
) -> Result<HashMap<String, YochananofMetadata>> {
    info!("Start fetching items from page {url}");

    let client = Client::new();
    let product_selector = create_selector(".price-box")?;
    let image_selector = create_selector("img[itemprop=\"image\"")?;
    let type_selector = create_selector("td.type")?;
    let collapsible_selector = create_selector("div[data-role=\"collapsible\"")?;
    let qty_selector = create_selector(".qty")?;
    let unit_selector = create_selector(".weight")?;
    let ingredient_selector = create_selector(".ingredient")?;
    let nutritional_row_selector = create_selector(".nutritional-row")?;
    let title_selector = create_selector(".title")?;
    let nutritional_box_selector = create_selector(".nutritional-box")?;
    let mut header_map = reqwest::header::HeaderMap::new();
    header_map.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());
    let mut data = HashMap::new();

    'pages: for i in 1..300 {
        let link = format!("{url}?p={i}");
        let page = get_to_text_with_retries(&link)
            .await
            .ok_or(anyhow!("Couldn't fetch yohananof link : {link}"))?;
        let document = Html::parse_document(&page);

        let product_ids = document
            .select(&product_selector)
            .flat_map(|e| e.value().attr("data-product-id"))
            .collect_vec();
        if product_ids.is_empty() {
            break 'pages;
        }

        for product_id in product_ids {
            let text = post_to_text_with_headers_with_retries(
                &client,
                &format!(
                    "https://yochananof.co.il/s59/catalog/product/view/id/{product_id}?mpquickview=1"
                ),
        format!("productId={product_id}"),
                None,
                header_map.clone()
            ).await.unwrap();
            let document = Html::parse_document(&text);
            let image_url = document
                .select(&image_selector)
                .next()
                .and_then(|e| e.value().attr("src"))
                .map(|s| s.to_string());
            let barcode = document
                .select(&type_selector)
                .flat_map(|e| {
                    let text = e.text().collect::<String>();
                    if text != "ברקוד" {
                        return None;
                    }
                    let sibling = e
                        .next_sibling_element()
                        .map(|e: scraper::ElementRef<'_>| e.text().collect::<String>());
                    return sibling;
                })
                .next()
                .unwrap_or("".to_string());
            let ingredients = document
                .select(&collapsible_selector)
                .find(|e| e.text().collect::<String>().trim() == "רכיבים")
                .and_then(|e| e.next_sibling_element())
                .map(|e| e.text().collect::<String>().trim().to_string());

            let mut nutritional_values_full = Vec::new();
            for nutritional_row in document.select(&nutritional_row_selector) {
                let size = nutritional_row
                    .select(&title_selector)
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string());

                let mut nutritional_values = Vec::new();
                for nutrition_element in nutritional_row.select(&nutritional_box_selector) {
                    let number = nutrition_element
                        .select(&qty_selector)
                        .next()
                        .map(|e| e.text().collect::<String>().trim().to_string())
                        .unwrap_or_default();
                    let unit = nutrition_element
                        .select(&unit_selector)
                        .next()
                        .map(|e| e.text().collect::<String>().trim().to_string())
                        .unwrap_or_default();
                    let nutrition_type = nutrition_element
                        .select(&ingredient_selector)
                        .next()
                        .map(|e| e.text().collect::<String>().trim().to_string())
                        .unwrap_or_default();
                    let nutritional_value = israel_prices::nutrition::NutritionalValue::new(
                        number,
                        unit,
                        nutrition_type,
                    );
                    if let Some(nutritional_value) = nutritional_value {
                        nutritional_values.push(nutritional_value);
                    }
                }
                if !nutritional_values.is_empty() {
                    nutritional_values_full.push(NutritionalValues {
                        size: size,
                        values: nutritional_values,
                    });
                };
            }
            let metadata = israel_prices::models::YochananofMetadata {
                nutrition_info: nutritional_values_full,
                ingredients,
                image_url,
            };
            data.insert(barcode, metadata);
        }
    }
    info!(
        "Finished fetching items from page {url}, found {} items.",
        data.len()
    );
    Ok(data)
}

async fn _fetch_yohananof() -> Result<()> {
    let page = get_to_text_with_retries("https://yochananof.co.il/s59")
        .await
        .ok_or(anyhow!("Couldn't fetch yohananof"))?;
    let document = Html::parse_document(&page);
    let category_selector = create_selector(".category-item a")?;

    let links = document
        .select(&category_selector)
        .flat_map(|e| e.value().attr("href"))
        .map(|e| e.to_string())
        .collect_vec();
    let mut previous = "9999999999".to_string();
    let mut tasks = Vec::new();
    for link in links {
        if link.contains("all-promotions") || link.starts_with(&previous) {
            continue;
        }
        previous = link
            .strip_suffix(".html")
            .map(|s| s.to_string())
            .unwrap_or(previous);
        tasks.push(tokio::spawn(fetch_products_from_page_yochananof(
            link.clone(),
        )));
    }
    let total_tasks = tasks.len();
    for (i, task) in tasks.into_iter().enumerate() {
        let result = task.await??;
        info!("Got {} results in task {i}/{total_tasks}", result.len());
    }
    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new("israel_prices=DEBUG"));

    tracing::subscriber::set_global_default(subscriber)?;

    // fetch_tiv_taam().await?;
    // online_store_data::fetch_hatzi_hinam().await?;
    _fetch_yohananof().await?;
    Ok(())
}
