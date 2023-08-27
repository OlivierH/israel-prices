use crate::{
    models::{
        self, Barcode, ImageUrl, ImageUrlMetadata, RamiLevyMetadata, ScrapedData,
        ShufersalMetadata, VictoryMetadata, YochananofMetadata,
    },
    nutrition::{self, NutritionalValue, NutritionalValues},
    reqwest_utils::{self, get_to_text_with_retries, post_to_text_with_headers_with_retries},
};
use anyhow::{anyhow, Result};
use futures::{stream::FuturesUnordered, StreamExt};
use itertools::Itertools;
use metrics::increment_counter;
use reqwest::Client;
use scraper::{Element, ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::Instrument;
use tracing::{debug, info, instrument, span, Level};

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

fn get_categories(document: &Html) -> Result<Vec<String>> {
    let selector = create_selector(".modal-dialog")?;
    let modal_dialog = match document.select(&selector).next() {
        Some(element) => element,
        None => return Ok(Vec::new()),
    };
    let attrs = match modal_dialog.value().attr("data-gtm") {
        Some(attrs) => attrs,
        None => return Ok(Vec::new()),
    };
    let attrs: serde_json::Value = serde_json::from_str(attrs)?;
    let attrs = attrs
        .as_object()
        .ok_or_else(|| anyhow!("data-gtm is not an obejct"))?
        .into_iter()
        .filter(|pair| pair.0.starts_with("categoryLevel"))
        .sorted_by(|a, b| Ord::cmp(a.0, b.0))
        .map(|pair| pair.1.as_str().unwrap().to_string())
        .collect::<Vec<String>>();
    Ok(attrs)
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
            categories: match categories.is_empty() {
                false => Some(serde_json::to_string(&categories)?),
                true => None,
            },
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
    Ok(data)
}

#[instrument(skip_all)]
pub async fn fetch_rami_levy_metadata() -> Result<HashMap<models::Barcode, RamiLevyMetadata>> {
    let departments = vec![
        49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 951, 1236, 1237, 1238, 1239, 1240,
        1243, 1244, 1245, 1246,
    ];

    #[derive(Deserialize, Debug)]
    struct RamiLevyJsonFoodSymbol {
        value: String,
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct RamiLevyJsonImages {
        small: Option<String>,
        original: Option<String>,
        trim: Option<String>,
        transparent: Option<String>,
    }
    #[derive(Deserialize, Debug)]
    struct RamiLevyJsonNutritionalValueField {
        #[serde(rename = "UOM")]
        unit_of_measurement: String,
        value: String,
    }
    #[derive(Deserialize, Debug)]
    struct RamiLevyJsonNutritionalValues {
        label: String,
        fields: Vec<RamiLevyJsonNutritionalValueField>,
    }
    #[derive(Deserialize, Debug)]
    struct RamiLevyJsonDetails {
        #[serde(rename = "Nutritional_Values")]
        nutritional_values: Vec<RamiLevyJsonNutritionalValues>,
        #[serde(rename = "Ingredient_Sequence_and_Name")]
        ingredient_sequence_and_name: String,
        #[serde(rename = "Food_Symbol_Red")]
        product_symbols: Option<Vec<RamiLevyJsonFoodSymbol>>,
    }
    #[derive(Deserialize, Debug)]
    struct RamiLevyJsonCategory {
        name: String,
    }
    #[derive(Deserialize, Debug)]
    struct RamiLevyJsonData {
        department: Option<RamiLevyJsonCategory>,
        group: Option<RamiLevyJsonCategory>,
        #[serde(rename = "subGroup")]
        sub_group: Option<RamiLevyJsonCategory>,
        #[serde(rename = "gs")]
        details: RamiLevyJsonDetails,
        images: RamiLevyJsonImages,
        barcode: models::Barcode,
    }

    #[derive(Deserialize, Debug)]
    struct RamiLevyJsonValue {
        data: Vec<RamiLevyJsonData>,
    }

    let client = reqwest::Client::new();
    let url = "https://www.rami-levy.co.il/api/catalog";
    let mut all_products = HashMap::new();
    for department in departments {
        let response_str = reqwest_utils::post_to_text_with_retries(
            &client,
            url,
            format!("{{\"d\":{department},\"size\":10000}}"),
            None,
        )
        .await
        .ok_or(anyhow!("Error fetching rami levy department {department}"))?;
        let data = serde_json::from_str::<RamiLevyJsonValue>(&response_str)?;
        for data in data.data {
            let categories = Some(
                serde_json::to_string(&[
                    &data.department.as_ref().map_or("", |c| &c.name),
                    &data.group.as_ref().map_or("", |c| &c.name),
                    &data.sub_group.as_ref().map_or("", |c| &c.name),
                ])
                .unwrap(),
            );
            let ingredients = Some(data.details.ingredient_sequence_and_name.to_string())
                .filter(|s| !s.is_empty());
            let product_symbols = data.details.product_symbols.as_ref().and_then(|symbols| {
                let symbols = symbols
                    .iter()
                    .map(|p| p.value.clone())
                    .collect::<Vec<String>>();
                if symbols.is_empty() {
                    None
                } else {
                    Some(symbols)
                }
            });
            let product_symbols = match product_symbols {
                Some(product_symbols) => Some(serde_json::to_string(&product_symbols).unwrap()),
                None => None,
            };
            let nutrition_info = data
                .details
                .nutritional_values
                .iter()
                .map(|v| {
                    let (value, unit) = if let Some(field) = v.fields.get(0) {
                        (field.value.as_str(), field.unit_of_measurement.as_str())
                    } else {
                        ("", "")
                    };
                    (unit.to_string(), value.to_string(), v.label.clone())
                })
                .collect::<Vec<(String, String, String)>>();
            let nutrition_info = if nutrition_info.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&nutrition_info)?)
            };

            all_products.insert(
                data.barcode,
                RamiLevyMetadata {
                    categories,
                    nutrition_info,
                    ingredients,
                    product_symbols,
                    image_url_original: data.images.original.clone(),
                    image_url_small: data.images.small.clone(),
                    image_url_transparent: data.images.transparent.clone(),
                    image_url_trim: data.images.trim.clone(),
                },
            );
        }
    }
    Ok(all_products)
}

// Note this works for more than Victory
#[instrument]
pub async fn fetch_victory_metadata(
    url_start: &str,
    fetch_limit: usize,
) -> Result<HashMap<String, VictoryMetadata>> {
    #[derive(Deserialize, Debug)]
    struct VictoryJsonSizeValues {
        #[serde(rename = "unitOfMeasure")]
        unit_of_measure: Option<VictoryJsonNames>,
        value: Option<f32>,
        #[serde(rename = "valueLessThan")]
        value_less_than: bool,
    }

    #[derive(Deserialize, Debug)]
    struct VictoryJsonImage {
        url: String,
    }
    #[derive(Deserialize, Debug)]
    struct OneStringField {
        #[serde(rename = "1")]
        name: Option<String>,
    }
    #[derive(Deserialize, Debug)]
    struct VictoryJsonFamily {
        categories: Option<Vec<VictoryJsonNames>>,
    }
    #[derive(Deserialize, Debug)]
    struct VictoryJsonNames {
        names: Option<OneStringField>,
    }
    impl VictoryJsonNames {
        fn str(&self) -> String {
            self.names
                .as_ref()
                .and_then(|n| n.name.clone())
                .unwrap_or_default()
        }
    }
    #[derive(Deserialize, Debug)]
    struct VictoryJsonNutritionValue {
        names: Option<OneStringField>,
        #[serde(rename = "sizeValues")]
        size: Vec<VictoryJsonSizeValues>,
    }
    #[derive(Deserialize, Debug)]
    struct VictoryJsonNutritionValues {
        values: Vec<VictoryJsonNutritionValue>,
        sizes: Vec<VictoryJsonNames>,
    }
    #[derive(Deserialize, Debug)]
    struct VictoryJsonData {
        ingredients: Option<String>,
    }
    #[derive(Deserialize, Debug)]
    struct VictoryJsonDataWrapper {
        #[serde(rename = "1")]
        data: Option<VictoryJsonData>,
    }

    #[derive(Deserialize, Debug)]
    struct VictoryJsonProduct {
        barcode: String,
        data: Option<VictoryJsonDataWrapper>,
        family: Option<VictoryJsonFamily>,
        #[serde(rename = "nutritionValues")]
        nutrition: Option<VictoryJsonNutritionValues>,
        image: Option<VictoryJsonImage>,
    }

    #[derive(Deserialize, Debug)]
    struct VictoryJsonResponse {
        products: Vec<VictoryJsonProduct>,
    }

    let mut v = HashMap::new();

    // We expect to need <150 requests, but this protects against infinite loops while being future proof.
    for i in 0..1000 {
        let from = i * 500;
        let url = format!("{url_start}/products?filters={{\"must\":{{}}}}&from={from}&size=500");
        info!("{i}: fetching url {url}");
        let text = reqwest_utils::get_to_text_with_retries(&url).await;
        if let Some(t) = &text {
            std::fs::write("last_victory.json", t)?;
        }
        let response: VictoryJsonResponse = serde_json::from_str(&text.unwrap()).unwrap();

        if response.products.is_empty() {
            break;
        }
        for product in response.products {
            let nutritional_values = product.nutrition.map(|n| NutritionalValues {
                size: n.sizes.get(0).map(|s| s.str()),
                values: n
                    .values
                    .iter()
                    .flat_map(|nutrition_value| {
                        let size = match nutrition_value.size.get(0) {
                            Some(x) => x,
                            None => return None,
                        };

                        nutrition::NutritionalValue::create(
                            size.value.unwrap_or(0.0).to_string(),
                            size.unit_of_measure
                                .as_ref()
                                .map_or(String::default(), |n| n.str()),
                            nutrition_value
                                .names
                                .as_ref()
                                .map_or(String::default(), |n| {
                                    n.name.clone().unwrap_or_default().replace("‎", "")
                                }),
                            size.value_less_than,
                        )
                    })
                    .collect::<Vec<NutritionalValue>>(),
            });

            let ingredients = product
                .data
                .and_then(|d| d.data)
                .and_then(|d| d.ingredients);
            let categories = product.family.and_then(|f| {
                f.categories
                    .map(|cs| cs.iter().map(|c| c.str()).collect::<Vec<String>>())
            });
            let barcode = product.barcode;
            let image_url = product.image.map(|i| i.url);
            v.insert(
                barcode,
                VictoryMetadata {
                    categories,
                    nutrition_info: nutritional_values,
                    ingredients,
                    image_url,
                },
            );
        }
        if fetch_limit > 0 && fetch_limit < v.len() {
            break;
        }
    }
    Ok(v)
}

// Note: this doesn't work currently, need to pass a cookie.
#[allow(dead_code)]
#[instrument]
pub async fn fetch_hatzi_hinam() -> Result<()> {
    let catalog = reqwest_utils::get_json_to_text_with_retries(
        "https://shop.hazi-hinam.co.il/proxy/api/Catalog/get",
    )
    .await
    .ok_or(anyhow!("Could not get Hatzi Hinam catalog"))?;

    #[derive(Deserialize, Debug)]
    struct HatziHinamJsonResponse {
        #[serde(rename = "Results")]
        results: HatziHinamJsonResults,
    }
    #[derive(Deserialize, Debug)]
    struct HatziHinamJsonResults {
        #[serde(rename = "Categories")]
        categories: Vec<HatziHinamJsonCategory>,
    }
    #[derive(Deserialize, Debug)]
    struct HatziHinamJsonCategory {
        #[serde(rename = "SubCategories")]
        subcategories: Vec<HatziHinamJsonSubCategory>,
    }
    #[derive(Deserialize, Debug)]
    struct HatziHinamJsonSubCategory {
        #[serde(rename = "Id")]
        id: i32,
    }
    let catalog = serde_json::from_str::<HatziHinamJsonResponse>(&catalog)?;

    let subcategories = catalog
        .results
        .categories
        .iter()
        .flat_map(|c| c.subcategories.iter())
        .map(|c| c.id);

    for subcategory in subcategories {
        let _url = format!(
            "https://shop.hazi-hinam.co.il/proxy/api/item/getItemsBySubCategory?Id={subcategory}"
        );
        // need a cookie here
    }
    Ok(())
}

#[instrument]
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

    for i in 1..300 {
        span!(tracing::Level::DEBUG, "Fetching page", page = i);
        debug!("Fetching page");
        let link = format!("{url}?p={i}");
        let page = get_to_text_with_retries(&link)
            .await
            .ok_or(anyhow!("Couldn't fetch yochananof link : {link}"))?;
        let document = Html::parse_document(&page);

        let product_ids = document
            .select(&product_selector)
            .flat_map(|e| e.value().attr("data-product-id"))
            .collect_vec();
        if product_ids.is_empty() {
            break;
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
                    let nutritional_value = NutritionalValue::new(number, unit, nutrition_type);
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
            let metadata = YochananofMetadata {
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

#[instrument]
pub async fn fetch_yochananof_metadata() -> Result<HashMap<String, YochananofMetadata>> {
    info!("Starting");
    let page = get_to_text_with_retries("https://yochananof.co.il/s59")
        .await
        .ok_or(anyhow!("Couldn't fetch yochananof main page"))?;
    debug!("Fetched yochananof main page");
    let document = Html::parse_document(&page);
    let category_selector = create_selector(".category-item a")?;

    let links = document
        .select(&category_selector)
        .flat_map(|e| e.value().attr("href"))
        .map(|e| e.to_string())
        .collect_vec();
    let mut previous = "9999999999".to_string();
    let mut tasks = Vec::new();
    info!("Got {} raw links", links.len());
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
    info!("Started {total_tasks} tasks");
    let mut data = HashMap::new();
    for (i, task) in tasks.into_iter().enumerate() {
        let result = task.await??;
        info!("Got {} results in task {i}/{total_tasks}", result.len());
        data.extend(result.into_iter());
    }
    Ok(data)
}

// excalibur is a codename for various stores, including Victory,that use the same backend.
#[instrument]
pub async fn scrap_excalibur_data(
    source: &str,
    url_start: &str,
    fetch_limit: usize,
) -> Result<Vec<ScrapedData>> {
    #[derive(Deserialize, Debug)]
    struct ExcaliburJsonSizeValues {
        #[serde(rename = "unitOfMeasure")]
        unit_of_measure: Option<ExcaliburJsonNames>,
        value: Option<f32>,
        #[serde(rename = "valueLessThan")]
        value_less_than: bool,
    }

    #[derive(Deserialize, Debug)]
    struct ExcaliburJsonImage {
        url: String,
    }
    #[derive(Deserialize, Debug)]
    struct OneStringField {
        #[serde(rename = "1")]
        name: Option<String>,
    }
    #[derive(Deserialize, Debug)]
    struct ExcaliburJsonFamily {
        categories: Option<Vec<ExcaliburJsonNames>>,
    }
    #[derive(Deserialize, Debug)]
    struct ExcaliburJsonNames {
        names: Option<OneStringField>,
    }
    impl ExcaliburJsonNames {
        fn str(&self) -> String {
            self.names
                .as_ref()
                .and_then(|n| n.name.clone())
                .unwrap_or_default()
        }
    }
    #[derive(Deserialize, Debug)]
    struct ExcaliburJsonNutritionValue {
        names: Option<OneStringField>,
        #[serde(rename = "sizeValues")]
        size: Vec<ExcaliburJsonSizeValues>,
    }
    #[derive(Deserialize, Debug)]
    struct ExcaliburJsonNutritionValues {
        values: Vec<ExcaliburJsonNutritionValue>,
        sizes: Vec<ExcaliburJsonNames>,
    }
    #[derive(Deserialize, Debug)]
    struct ExcaliburJsonData {
        ingredients: Option<String>,
    }
    #[derive(Deserialize, Debug)]
    struct ExcaliburJsonDataWrapper {
        #[serde(rename = "1")]
        data: Option<ExcaliburJsonData>,
    }

    #[derive(Deserialize, Debug)]
    struct ExcaliburJsonProduct {
        barcode: String,
        data: Option<ExcaliburJsonDataWrapper>,
        family: Option<ExcaliburJsonFamily>,
        #[serde(rename = "nutritionValues")]
        nutrition: Option<ExcaliburJsonNutritionValues>,
        image: Option<ExcaliburJsonImage>,
    }

    #[derive(Deserialize, Debug)]
    struct ExcaliburJsonResponse {
        products: Vec<ExcaliburJsonProduct>,
    }

    let mut v = Vec::new();

    // We expect to need <150 requests, but this protects against infinite loops while being future proof.
    for i in 0..1000 {
        let from = i * 500;
        let url = format!("{url_start}/products?filters={{\"must\":{{}}}}&from={from}&size=500");
        info!("{i}: fetching url {url}");
        let text = reqwest_utils::get_to_text_with_retries(&url).await;
        // if let Some(t) = &text {
        //     std::fs::write("last_victory.json", t)?;
        // }
        let response: ExcaliburJsonResponse = serde_json::from_str(&text.unwrap()).unwrap();

        if response.products.is_empty() {
            break;
        }
        for product in response.products {
            let nutritional_values = product
                .nutrition
                .map(|n| NutritionalValues {
                    size: n.sizes.get(0).and_then(|s| {
                        if s.str().is_empty() {
                            None
                        } else {
                            Some(s.str())
                        }
                    }),
                    values: n
                        .values
                        .iter()
                        .flat_map(|nutrition_value| {
                            let size = match nutrition_value.size.get(0) {
                                Some(x) => x,
                                None => return None,
                            };

                            nutrition::NutritionalValue::create(
                                size.value.unwrap_or(0.0).to_string(),
                                size.unit_of_measure
                                    .as_ref()
                                    .map_or(String::default(), |n| n.str()),
                                nutrition_value
                                    .names
                                    .as_ref()
                                    .map_or(String::default(), |n| {
                                        n.name.clone().unwrap_or_default().replace("‎", "")
                                    }),
                                size.value_less_than,
                            )
                        })
                        .collect::<Vec<NutritionalValue>>(),
                })
                .and_then(|nv| if nv.values.is_empty() { None } else { Some(nv) });

            let ingredients = product
                .data
                .and_then(|d| d.data)
                .and_then(|d| d.ingredients);
            let categories = product
                .family
                .and_then(|f| {
                    f.categories
                        .map(|cs| cs.iter().map(|c| c.str()).collect::<Vec<String>>())
                })
                .unwrap_or_default();
            let barcode = product.barcode;
            let image_urls = product
                .image
                .map(|i| {
                    vec![ImageUrl {
                        link: i.url,
                        metadata: models::ImageUrlMetadata::Templated,
                    }]
                })
                .unwrap_or_default();
            v.push(ScrapedData {
                source: source.to_string(),
                barcode,
                categories,
                nutrition_info: nutritional_values.map(|nv| vec![nv]).unwrap_or_default(),
                ingredients,
                image_urls,
            });
        }
        if fetch_limit > 0 && fetch_limit < v.len() {
            break;
        }
    }
    Ok(v)
}

#[instrument]
pub async fn scrap_rami_levy(fetch_limit: usize) -> Result<Vec<ScrapedData>> {
    let departments = vec![
        49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 951, 1236, 1237, 1238, 1239, 1240,
        1243, 1244, 1245, 1246,
    ];

    #[derive(Deserialize, Debug)]
    struct RamiLevyJsonFoodSymbol {
        value: String,
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct RamiLevyJsonImages {
        small: Option<String>,
        original: Option<String>,
        trim: Option<String>,
        transparent: Option<String>,
    }
    #[derive(Deserialize, Debug)]
    struct RamiLevyJsonNutritionalValueField {
        #[serde(rename = "UOM")]
        unit_of_measurement: String,
        value: String,
    }
    #[derive(Deserialize, Debug)]
    struct RamiLevyJsonNutritionalValues {
        label: String,
        fields: Vec<RamiLevyJsonNutritionalValueField>,
    }
    #[derive(Deserialize, Debug)]
    struct RamiLevyJsonDetails {
        #[serde(rename = "Nutritional_Values")]
        nutritional_values: Vec<RamiLevyJsonNutritionalValues>,
        #[serde(rename = "Ingredient_Sequence_and_Name")]
        ingredient_sequence_and_name: String,
        #[serde(rename = "Food_Symbol_Red")]
        product_symbols: Option<Vec<RamiLevyJsonFoodSymbol>>,
    }
    #[derive(Deserialize, Debug)]
    struct RamiLevyJsonCategory {
        name: String,
    }
    #[derive(Deserialize, Debug)]
    struct RamiLevyJsonData {
        department: Option<RamiLevyJsonCategory>,
        group: Option<RamiLevyJsonCategory>,
        #[serde(rename = "subGroup")]
        sub_group: Option<RamiLevyJsonCategory>,
        #[serde(rename = "gs")]
        details: RamiLevyJsonDetails,
        images: RamiLevyJsonImages,
        barcode: models::Barcode,
    }

    #[derive(Deserialize, Debug)]
    struct RamiLevyJsonValue {
        data: Vec<RamiLevyJsonData>,
    }

    let client = reqwest::Client::new();
    let url = "https://www.rami-levy.co.il/api/catalog";
    let mut all_products = Vec::new();
    for department in departments {
        let span = span!(Level::DEBUG, "department", department = department);
        span.in_scope(|| {
            info!("Starting to handle department");
        });
        let response_str = reqwest_utils::post_to_text_with_retries(
            &client,
            url,
            format!("{{\"d\":{department},\"size\":10000}}"),
            None,
        )
        .instrument(span.clone())
        .await
        .ok_or(anyhow!("Error fetching rami levy department {department}"))?;
        let _entered = span.enter();
        let data = serde_json::from_str::<RamiLevyJsonValue>(&response_str)?;
        for data in data.data {
            let categories = vec![&data.department, &data.group, &data.sub_group]
                .iter()
                .filter_map(|c| c.as_ref())
                .map(|c| c.name.clone())
                .collect::<Vec<String>>();
            let ingredients = Some(data.details.ingredient_sequence_and_name.to_string())
                .filter(|s| !s.is_empty());
            let product_symbols = data.details.product_symbols.as_ref().and_then(|symbols| {
                let symbols = symbols
                    .iter()
                    .map(|p| p.value.clone())
                    .collect::<Vec<String>>();
                if symbols.is_empty() {
                    None
                } else {
                    Some(symbols)
                }
            });
            let _product_symbols = match product_symbols {
                Some(product_symbols) => Some(serde_json::to_string(&product_symbols).unwrap()),
                None => None,
            };
            let nutrition_info = data
                .details
                .nutritional_values
                .iter()
                .filter_map(|v| {
                    let (value, unit) = if let Some(field) = v.fields.get(0) {
                        (field.value.as_str(), field.unit_of_measurement.as_str())
                    } else {
                        ("", "")
                    };
                    NutritionalValue::new(value.to_string(), unit.to_string(), v.label.clone())
                })
                .collect::<Vec<NutritionalValue>>();
            let nutrition_info = match nutrition_info.is_empty() {
                false => vec![NutritionalValues {
                    size: None,
                    values: nutrition_info,
                }],
                true => Vec::new(),
            };

            let mut image_urls = Vec::new();
            if let Some(link) = data.images.original {
                image_urls.push(ImageUrl {
                    link,
                    metadata: models::ImageUrlMetadata::Original,
                });
            };
            if let Some(link) = data.images.small {
                image_urls.push(ImageUrl {
                    link,
                    metadata: models::ImageUrlMetadata::Small,
                });
            };
            if let Some(link) = data.images.transparent {
                image_urls.push(ImageUrl {
                    link,
                    metadata: models::ImageUrlMetadata::Transparent,
                });
            };
            if let Some(link) = data.images.trim {
                image_urls.push(ImageUrl {
                    link,
                    metadata: models::ImageUrlMetadata::Trim,
                });
            };
            all_products.push(ScrapedData {
                source: "rami_levy".to_string(),
                barcode: data.barcode.to_string(),
                categories,
                nutrition_info,
                ingredients,
                // product_symbols,
                image_urls,
            });
            if fetch_limit > 0 && all_products.len() > fetch_limit {
                break;
            }
        }
        if fetch_limit > 0 && all_products.len() > fetch_limit {
            break;
        }
    }
    Ok(all_products)
}

#[instrument(skip(item_codes, limit))]
pub async fn scrap_shufersal(
    item_codes: &[Barcode],
    _chunk: usize,
    limit: usize,
) -> Result<Vec<ScrapedData>> {
    fn scrap_nutrition_info(document: &Html) -> Result<Vec<NutritionalValues>> {
        let selector = create_selector(".nutritionListTitle")?;
        let sub_info_selector = create_selector(".subInfo")?;
        let nutrition_item_selector = create_selector(".nutritionItem")?;
        let number_selector = create_selector(".number")?;
        let name_selector = create_selector(".name")?;
        let text_selector = create_selector(".text")?;

        let nutrition_titles: Vec<ElementRef<'_>> =
            document.select(&selector).collect::<Vec<ElementRef>>();
        let mut all_nutritional_values = Vec::new();
        for title in nutrition_titles {
            let li = title
                .parent()
                .ok_or(anyhow!("No parent for nutrition_title"))?;
            let size = title
                .select(&sub_info_selector)
                .next()
                .map(|elem| elem.text().collect::<String>());

            let mut values = Vec::new();
            for item in ElementRef::wrap(li)
                .unwrap()
                .select(&nutrition_item_selector)
            {
                let number = get_text(&item, &number_selector)?;
                let unit = get_text(&item, &name_selector)?;
                let nutrition_type = get_text(&item, &text_selector)?;
                if let Some(n) = NutritionalValue::new(number, unit, nutrition_type) {
                    values.push(n);
                }
            }
            all_nutritional_values.push(NutritionalValues { size, values });
        }
        return Ok(all_nutritional_values);
    }

    async fn scrap_item(item_code: Barcode) -> Result<Option<ScrapedData>> {
        let url = format!("https://www.shufersal.co.il/online/he/p/P_{item_code}/json");
        debug!("Fetching url {url} for itemcode {item_code}");
        let document = match reqwest_utils::get_to_text_with_retries(&url).await {
            Some(text) => text,
            None => return Ok(None),
        };
        let document = Html::parse_document(&document);
        let categories = get_categories(&document)?;
        let nutrition_info = scrap_nutrition_info(&document)?;
        let ingredients = get_ingredients(&document)?;
        let image_url_selector = create_selector("img")?;
        let image_urls = document
            .select(&image_url_selector)
            .map(|e| e.value().attr("src").unwrap_or_default())
            .filter(|url| url.contains("product_images"))
            .map(|url| {
                {
                    (
                        url,
                        url.split("/").filter(|u| u.starts_with("product")).last(),
                    )
                }
            })
            .filter_map(|(url, suffix)| match suffix {
                Some(suffix) => Some((url, suffix)),
                None => None,
            })
            .map(|(url, suffix)| ImageUrl {
                link: url.to_string(),
                metadata: ImageUrlMetadata::from(suffix),
            })
            .collect::<Vec<ImageUrl>>();
        let _product_symbols = get_product_symbols(&document)?;
        increment_counter!("fetch_shufersal_item_completed");
        Ok(Some(ScrapedData {
            source: "shufersal".to_string(),
            barcode: item_code.to_string(),
            categories,
            nutrition_info,
            ingredients,
            image_urls,
        }))
    }

    let mut data = Vec::new();
    let futures = FuturesUnordered::new();

    let item_codes = if limit == 0 {
        &item_codes[0..item_codes.len()]
    } else {
        &item_codes[0..limit]
    };

    info!("Starting to create tasks");
    for (i, item_code) in item_codes.into_iter().enumerate() {
        futures.push(tokio::spawn(scrap_item(*item_code)));
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
        if let Some(result) = result {
            data.push(result);
        }
    }
    info!("Finished to await tasks");
    Ok(data)
}
