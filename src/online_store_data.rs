use crate::{
    models::{Barcode, RamiLevyMetadata, ShufersalMetadata},
    nutrition,
};
use anyhow::{anyhow, Result};
use futures::{stream::FuturesUnordered, StreamExt};
use itertools::Itertools;
use metrics::{gauge, increment_counter};
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;
use tracing::{debug, info, instrument};

fn create_selector(selectors: &str) -> Result<Selector> {
    Ok(Selector::parse(selectors).map_err(|_| anyhow!("couldn't build selector"))?)
}
fn get_text(item: &ElementRef, selector: &Selector) -> Result<String> {
    Ok(item
        .select(&selector)
        .next()
        .ok_or(anyhow!("No data in selector"))?
        .text()
        .collect::<String>()
        .trim()
        .to_string())
}

fn get_categories(document: &Html) -> Result<Option<String>> {
    let selector = create_selector(".modal-dialog")?;
    let modal_dialog = match document.select(&selector).next() {
        Some(element) => element,
        None => return Ok(None),
    };
    let attrs = match modal_dialog.value().attr("data-gtm") {
        Some(attrs) => attrs,
        None => return Ok(None),
    };
    let attrs: serde_json::Value = serde_json::from_str(attrs)?;
    let attrs = attrs
        .as_object()
        .ok_or_else(|| anyhow!("data-gtm is not an obejct"))?
        .into_iter()
        .filter(|pair| pair.0.starts_with("categoryLevel"))
        .sorted_by(|a, b| Ord::cmp(a.0, b.0))
        .map(|pair| pair.1.as_str().unwrap())
        .collect::<Vec<&str>>();
    let categories = serde_json::to_string(&attrs)?;
    Ok(Some(categories))
}
fn get_nutrition_info(document: &Html) -> Result<Option<String>> {
    let selector = create_selector(".nutritionList")?;
    let nutrition_lists = document.select(&selector).collect::<Vec<ElementRef>>();
    if nutrition_lists.is_empty() {
        return Ok(None);
    }
    let nutrition_list = if nutrition_lists.len() == 1 {
        nutrition_lists[0]
    } else {
        *nutrition_lists
            .iter()
            .filter_map(|elem| {
                elem.parent()
                    .and_then(|f| f.parent())
                    .and_then(|f| {
                        f.children()
                            .filter(|child| child.value().is_element())
                            .find_or_first(|child| {
                                child.value().as_element().unwrap().has_class(
                                    "subInfo",
                                    scraper::CaseSensitivity::AsciiCaseInsensitive,
                                )
                            })
                    })
                    .map(|v| (elem, v))
            })
            .find_or_first(|pair| {
                ElementRef::wrap(pair.1)
                    .unwrap()
                    .text()
                    .collect::<String>()
                    .contains("100 גרם")
            })
            .unwrap()
            .0
    };
    let nutrition_item_selector = create_selector(".nutritionItem")?;
    let number_selector = create_selector(".number")?;
    let name_selector = create_selector(".name")?;
    let text_selector = create_selector(".text")?;

    let mut values = Vec::new();
    for item in nutrition_list.select(&nutrition_item_selector) {
        let number = get_text(&item, &number_selector)?;
        let unit = get_text(&item, &name_selector)?;
        let nutrition_type = get_text(&item, &text_selector)?;

        if let Some(nutrition) = nutrition::NutritionalValue::new(number, unit, nutrition_type) {
            values.push(nutrition.to_tuple());
        }
    }

    return Ok(Some(serde_json::to_string(&values)?));
}

fn get_ingredients(document: &Html) -> Result<Option<String>> {
    let selector = create_selector(".componentsText")?;
    Ok(get_text(&document.root_element(), &selector).ok())
}

fn get_product_symbols(document: &Html) -> Result<Option<String>> {
    let product_symbols_selector = create_selector(".productSymbols .pic")?;
    let symbols = document
        .select(&product_symbols_selector)
        .filter_map(|e| e.value().attr("alt"))
        .filter_map(|alt| alt.rsplit_once("."))
        .map(|alt| alt.1)
        .collect::<Vec<&str>>();
    if symbols.is_empty() {
        return Ok(None);
    }
    Ok(Some(serde_json::to_string(&symbols)?))
}

fn get_image_url(document: &Html) -> Result<Option<String>> {
    let image_url_selector = create_selector(".img-responsive")?;
    let url = document
        .select(&image_url_selector)
        .next()
        .and_then(|e| e.value().attr("src"))
        .map(|s| s.to_string());
    return Ok(url);
}

async fn fetch(item_code: Barcode) -> Result<(Barcode, ShufersalMetadata)> {
    let url = format!("https://www.shufersal.co.il/online/he/p/P_{item_code}/json");
    debug!("Fetching url {url} for itemcode {item_code}");

    let document = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&document);
    let categories = get_categories(&document)?;
    let nutrition_info = get_nutrition_info(&document)?;
    let ingredients = get_ingredients(&document)?;
    let product_symbols = get_product_symbols(&document)?;
    let image_url = get_image_url(&document)?;
    increment_counter!("fetch_shufersal_item_completed");
    Ok((
        item_code,
        ShufersalMetadata {
            categories,
            nutrition_info,
            ingredients,
            product_symbols,
            image_url,
        },
    ))
}

#[instrument(skip_all)]
pub async fn fetch_shufersal_metadata(
    item_codes: &[Barcode],
    limit: usize,
) -> Result<HashMap<i64, ShufersalMetadata>> {
    let start = std::time::Instant::now();
    let mut data = HashMap::new();
    let futures = FuturesUnordered::new();

    let item_codes = if limit == 0 {
        &item_codes[0..item_codes.len()]
    } else {
        &item_codes[0..limit]
    };

    info!("Starting to create tasks");
    for (i, item_code) in item_codes.into_iter().enumerate() {
        futures.push(tokio::spawn(fetch(*item_code)));
        if (i % 100 == 0 && i < 1000) || (i % 1000 == 0) {
            debug!("Created task {i}");
        }
    }
    info!("Finished to create tasks");
    info!("Starting to await tasks");
    let mut stream = futures.enumerate();
    while let Some((i, result)) = stream.next().await {
        let result = result??;
        if (i % 100 == 0 && i < 1000) || (i % 1000 == 0) {
            debug!("Finished task {i}");
        }
        data.insert(result.0, result.1);
    }
    info!("Finished to await tasks");
    gauge!(
        "fetch_shufersal_metadata_time",
        start.elapsed().as_secs_f64()
    );
    info!(
        "It took {} mins and {} secs to fetch all shufersal metadata",
        start.elapsed().as_secs() / 60,
        start.elapsed().as_secs()
    );
    Ok(data)
}

async fn fetch_rami_levy(item_code: Barcode) -> Result<(Barcode, RamiLevyMetadata)> {
    let url = format!("https://www.shufersal.co.il/online/he/p/P_{item_code}/json");
    debug!("Fetching url {url} for itemcode {item_code}");

    let client = reqwest::Client::new();
    let document = client
        .post(url)
        .header("content-type", "application/json;charset=UTF-8")
        .body(format!("{{\"aggs\":1,\"type\":\"barcode\"}}"))
        .send()
        .await?
        .text()
        .await?;
    let document = Html::parse_document(&document);
    let categories = get_categories(&document)?;
    let nutrition_info = get_nutrition_info(&document)?;
    let ingredients = get_ingredients(&document)?;
    let product_symbols = get_product_symbols(&document)?;
    let image_url = get_image_url(&document)?;

    Ok((
        item_code,
        RamiLevyMetadata {
            categories,
            nutrition_info,
            ingredients,
            product_symbols,
            image_url,
        },
    ))
}

#[instrument(skip_all)]
pub async fn fetch_rami_levy_metadata(
    item_codes: Vec<Barcode>,
    limit: usize,
) -> Result<HashMap<i64, RamiLevyMetadata>> {
    let start = std::time::Instant::now();
    let mut data = HashMap::new();
    let futures = FuturesUnordered::new();

    let item_codes = if limit == 0 {
        &item_codes[0..item_codes.len()]
    } else {
        &item_codes[0..limit]
    };

    info!("Starting to create tasks");
    for (i, item_code) in item_codes.into_iter().enumerate() {
        futures.push(tokio::spawn(fetch_rami_levy(*item_code)));
        if (i % 100 == 0 && i < 1000) || (i % 1000 == 0) {
            debug!("Created task {i}");
        }
    }
    info!("Finished to create tasks");
    info!("Starting to await tasks");
    let mut stream = futures.enumerate();
    while let Some((i, result)) = stream.next().await {
        let result = result??;
        if (i % 100 == 0 && i < 1000) || (i % 1000 == 0) {
            debug!("Finished task {i}");
        }
        data.insert(result.0, result.1);
    }
    info!("Finished to await tasks");
    gauge!(
        "fetch_rami_levy_metadata_time",
        start.elapsed().as_secs_f64()
    );
    info!(
        "It took {} mins and {} secs to fetch all shufersal metadata",
        start.elapsed().as_secs() / 60,
        start.elapsed().as_secs()
    );
    Ok(data)
}
