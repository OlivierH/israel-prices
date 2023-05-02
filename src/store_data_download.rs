use crate::file_info::*;
use crate::parallel_download::{self, Download};
use crate::store::*;
use anyhow::anyhow;
use anyhow::Result;
use chrono::Datelike;
use futures::StreamExt; // 0.3.5
use reqwest::header;
use reqwest::{Client, Response}; // 0.10.6
use scraper::{ElementRef, Html, Selector};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, instrument, Instrument, Span};

async fn get_text(url: &str) -> Result<String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    // number of retries
    for _ in 1..10 {
        if let Ok(resp) = client.get(url).send().await {
            if let Ok(text) = resp.text().await {
                return Ok(text);
            };
        };
    }
    Err(anyhow!("Could not download {url} after retries"))
}

fn extract_csrf(html: String) -> Result<String> {
    let document = Html::parse_document(&html);
    let selector = Selector::parse("meta[name=\"csrftoken\"]").unwrap();
    let input = document.select(&selector).next().unwrap();
    input
        .value()
        .attr("content")
        .map(str::to_string)
        .ok_or(anyhow!("Cannot extract csrf token"))
}

fn get_cookie_from_resp(resp: &Response) -> Result<String> {
    let cookie = resp
        .headers()
        .get(header::SET_COOKIE)
        .ok_or(anyhow!("Cannot get cookie"))?;
    Ok(cookie.to_str()?.split(";").next().unwrap_or("").to_string())
}

fn get_headers() -> header::HeaderMap {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/x-www-form-urlencoded".parse().unwrap(),
    );
    return headers;
}

async fn get_downloads_publishedprice(
    store: &Store,
    username: &str,
    password: &str,
    file_limit: Option<usize>,
    download_semaphore: Arc<Semaphore>,
) -> Result<Vec<Download>> {
    let client = Client::builder().cookie_store(true).build()?;

    let permit = download_semaphore.acquire_owned().await?;
    // Main Page
    let resp = client
        .get("https://url.publishedprices.co.il/login")
        .send()
        .await?;

    let csrftoken = extract_csrf(resp.text().await?)?;
    let resp = client
        .post("https://url.publishedprices.co.il/login/user")
        .headers(get_headers())
        .body(format!(
            "r=&username={}&password={}&Submit=Sign+in&csrftoken={csrftoken}",
            username, password
        ))
        .send()
        .await?;
    let cookie = get_cookie_from_resp(&resp)?;
    let csrftoken = extract_csrf(resp.text().await?)?;

    let mut headers = get_headers();
    headers.insert(header::COOKIE, (&cookie).parse().unwrap());

    let url = "https://url.publishedprices.co.il/file/json/dir";
    let data = client.post(url).headers(headers.clone())
    .body(format!("sEcho=1&iColumns=5&sColumns=%2C%2C%2C%2C&iDisplayStart=0&iDisplayLength=100000&mDataProp_0=fname&sSearch_0=&bRegex_0=false&bSearchable_0=true&bSortable_0=true&mDataProp_1=typeLabel&sSearch_1=&bRegex_1=false&bSearchable_1=true&bSortable_1=false&mDataProp_2=size&sSearch_2=&bRegex_2=false&bSearchable_2=true&bSortable_2=true&mDataProp_3=ftime&sSearch_3=&bRegex_3=false&bSearchable_3=true&bSortable_3=true&mDataProp_4=&sSearch_4=&bRegex_4=false&bSearchable_4=true&bSortable_4=false&sSearch=&bRegex=false&iSortingCols=0&cd=%2F&csrftoken={csrftoken}"))
    .send().await?;
    let text = data.text().await?;

    drop(permit);

    let json_root = serde_json::from_str::<Value>(&text)?;
    let downloads: Vec<Download> = FileInfo::from_str_iter(
        json_root["aaData"]
            .as_array()
            .ok_or(anyhow!("Empty json array"))?
            .into_iter()
            .map(|elem| elem["fname"].to_string().replace("\"", "")),
        file_limit,
    )
    .map(|file_info| parallel_download::Download {
        path: format!(
            "https://url.publishedprices.co.il/file/d/{}",
            file_info.filename
        ),
        headers: Some(headers.clone()),
        dest: format!("data_raw/{}/{}", store.name, file_info.filename),
    })
    .collect();

    return Ok(downloads);
}

async fn get_downloads_simple_json_to_get(
    store: &Store,
    file_limit: Option<usize>,
    initial_url: &str,
    download_prefix: &str,
) -> Result<Vec<Download>> {
    let text = get_text(initial_url).await?;

    let json_root = serde_json::from_str::<Value>(&text)?;

    let downloads: Vec<Download> = FileInfo::from_str_iter(
        json_root
            .as_array()
            .ok_or(anyhow!("Empty json array"))?
            .into_iter()
            .map(|v| {
                v.as_object()
                    .expect("Element is not an object")
                    .get("FileNm")
                    .expect("No filename field")
                    .as_str()
                    .expect("not a string")
                    .to_string()
            }),
        file_limit,
    )
    .map(|fi| Download {
        dest: format!("data_raw/{}/{}", store.name, fi.filename),
        path: format!("{}{}", download_prefix, fi.filename),
        headers: None,
    })
    .collect();
    Ok(downloads)
}

async fn get_downloads_superpharm(
    store: &Store,
    file_limit: Option<usize>,
) -> Result<Vec<Download>> {
    // The flow is complex here.
    // First, we get the total number of pages.
    // Then, we fetch all the pages; and for each page,
    // we keep the associated links, and the associated cookie.
    // Then, we fetch each link, (with the associated cookie), and receive
    // another link.
    // It is this last link that will provide us the final file, and it also needs
    // the cookie.

    let html = get_text("http://prices.super-pharm.co.il/").await?;
    let selector = Selector::parse(".page_link a").unwrap();
    let num_pages = Html::parse_document(&html)
        .select(&selector)
        .find(|&elem| elem.text().next() == Some(">>"))
        .ok_or(anyhow!("Cannot find link to SuperPharm last page"))?
        .value()
        .attr("href")
        .ok_or(anyhow!("Cannot find href for SuperPharm last page"))?
        .rsplit_once("=")
        .ok_or(anyhow!("No equal sign in SuperPharm link"))?
        .1
        .parse::<usize>()?;

    let fetches = futures::stream::iter(1..(num_pages + 1))
        .map(|page| async move {
            let path = format!("http://prices.super-pharm.co.il/?page={page}");
            match reqwest::get(&path).await {
                Ok(resp) => {
                    let cookie = get_cookie_from_resp(&resp).unwrap();
                    match &resp.text().await {
                        Ok(html) => {
                            debug!("Success reading {path}");
                            let selector = Selector::parse(".file_list tr").unwrap();
                            Ok(Html::parse_document(&html)
                                .select(&selector)
                                .skip(1)
                                .map(|elem| {
                                    let mut iter = elem.children().into_iter();
                                    let filename: String = ElementRef::wrap(iter.nth(1).unwrap())
                                        .unwrap()
                                        .text()
                                        .collect();
                                    let link: String = ElementRef::wrap(
                                        ElementRef::wrap(iter.nth(3).unwrap())
                                            .unwrap()
                                            .children()
                                            .nth(0)
                                            .unwrap(),
                                    )
                                    .unwrap()
                                    .value()
                                    .attr("href")
                                    .unwrap()
                                    .to_string();
                                    (filename, link, cookie.clone())
                                })
                                .collect::<Vec<(String, String, String)>>())
                        }
                        Err(_) => Err(format!("ERROR reading {}", path)),
                    }
                }
                Err(_) => Err(format!("ERROR downloading {}", path)),
            }
        })
        .buffer_unordered(32)
        .collect::<Vec<Result<Vec<(String, String, String)>, String>>>();

    let mut all_links: HashSet<(String, String, String)> = HashSet::new();

    for result in fetches.await {
        match result {
            Ok(links) => {
                all_links.extend(links);
            }
            Err(e) => error!("Error: {e}"),
        }
    }
    let file_infos = FileInfo::keep_most_recents(
        all_links
            .into_iter()
            .map(|(filename, link, cookie)| {
                filename
                    .parse::<FileInfo>()
                    .unwrap()
                    .with_source(&link)
                    .with_cookie(&cookie)
            })
            .filter(|file_info| file_info.is_interesting())
            .collect(),
        file_limit,
    );
    let downloads = futures::stream::iter(file_infos)
        .map(|file_info| async move {
            let mut headers = header::HeaderMap::new();
            headers.insert(header::COOKIE, (&file_info.cookie).parse().unwrap());
            let resp = Client::new()
                .get(format!(
                    "http://prices.super-pharm.co.il{}",
                    file_info.source
                ))
                .headers(headers)
                .send()
                .await
                .unwrap();

            let url = {
                let text = resp.text().await.unwrap();
                let json_root = serde_json::from_str::<Value>(&text).unwrap();
                let href = json_root
                    .as_object()
                    .unwrap()
                    .get("href")
                    .unwrap()
                    .as_str()
                    .unwrap();
                format!("http://prices.super-pharm.co.il{href}")
            };
            let mut headers = header::HeaderMap::new();
            headers.insert(header::COOKIE, (&file_info.cookie).parse().unwrap());
            Download {
                dest: format!("data_raw/{}/{}", store.name, file_info.filename),
                headers: Some(headers),
                path: url,
            }
        })
        .buffer_unordered(32)
        .collect::<Vec<Download>>()
        .await;

    Ok(downloads)
}

async fn get_downloads_netiv_hahesed(
    store: &Store,
    file_limit: Option<usize>,
) -> Result<Vec<Download>> {
    fn get_links(document: &Html) -> Vec<String> {
        let selector = Selector::parse("#download_content a").unwrap();
        document
            .select(&selector)
            .map(|a| a.value().attr("href").unwrap().to_string())
            .collect()
    }
    let html = get_text("http://141.226.222.202/").await?;
    let document = Html::parse_document(&html);
    let mut all_links = get_links(&document);

    // Netiv Hahesed has a per-day filter, which is initialised to the current day.
    // In various occasions (holidays, too early in the day), the page will be empty of incomplete.
    // To ensure that we get corret data, we fetch the pages of the last 7 days.
    {
        let date_selector = Selector::parse("#MainContent_MainContent_txtDate").unwrap();
        let date = document
            .select(&date_selector)
            .next()
            .ok_or(anyhow!("Cannot find date"))?
            .value()
            .attr("value")
            .ok_or(anyhow!("Cannot find date"))?
            .split("/")
            .collect::<Vec<&str>>();
        let date =
            chrono::NaiveDate::from_ymd_opt(date[2].parse()?, date[1].parse()?, date[0].parse()?)
                .unwrap();

        fn get_value<'a>(document: &'a Html, selector: &str) -> Result<&'a str> {
            let selector = Selector::parse(&selector).unwrap();
            document
                .select(&selector)
                .next()
                .ok_or(anyhow!("cannot find view state"))?
                .value()
                .attr("value")
                .ok_or(anyhow!("Cannot find view state"))
        }

        let view_state = get_value(&document, "#__VIEWSTATE")?;
        let view_state_generator = get_value(&document, "#__VIEWSTATEGENERATOR")?;
        let event_validation = get_value(&document, "#__EVENTVALIDATION")?;

        for i in 1..=7 {
            let date = date.checked_sub_days(chrono::Days::new(i)).unwrap();

            let date = format!("{}/{}/{}", date.day(), date.month(), date.year());
            let client = Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap();
            let mut params = HashMap::new();
            params.insert("__VIEWSTATE", view_state);
            params.insert("__VIEWSTATEGENERATOR", view_state_generator);
            params.insert("__EVENTVALIDATION", event_validation);
            params.insert("ctl00$MainContent$MainContent_txtDate", &date);

            let html = client
                .post("http://141.226.222.202/")
                .form(&params)
                .send()
                .await?
                .text()
                .await?;
            let document = Html::parse_document(&html);
            all_links.extend(get_links(&document));
        }
    }

    let downloads: Vec<Download> = FileInfo::from_str_iter(all_links.into_iter(), file_limit)
        .map(|fi| Download {
            dest: format!("data_raw/{}/{}", store.name, fi.filename),
            path: format!("http://141.226.222.202/prices/{}", fi.filename),
            headers: None,
        })
        .collect();

    Ok(downloads)
}

async fn get_downloads_publish_price(
    store: &Store,
    file_limit: Option<usize>,
    url: &str,
) -> Result<Vec<Download>> {
    // e.g. http://publishprice.mega.co.il/20221031/
    let data_url = {
        debug!("Downloading {url} ...");
        let html = get_text(url).await?;
        debug!("Done downloading {url}.");
        let selector = Selector::parse("#files tr:nth-child(4) a").unwrap();
        let document = Html::parse_document(&html);
        let date = document
            .select(&selector)
            .last()
            .unwrap()
            .value()
            .attr("href")
            .unwrap();
        format!("{url}{date}")
    };

    debug!("Downloading {data_url}");
    let html = reqwest::get(&data_url).await?.text().await?;
    debug!("Done.");

    let selector = Selector::parse("#files a").unwrap();
    let document = Html::parse_document(&html);

    let downloads: Vec<Download> = FileInfo::from_str_iter(
        document
            .select(&selector)
            .skip(3) // header
            .map(|a| a.value().attr("href").unwrap().to_string()),
        file_limit,
    )
    .map(|fi| Download {
        dest: format!("data_raw/{}/{}", store.name, fi.filename),
        path: format!("{data_url}{}", fi.filename),
        headers: None,
    })
    .collect();

    Ok(downloads)
}

async fn get_downloads_matrix_catalog(
    store: &Store,
    file_limit: Option<usize>,
    chain: &str,
) -> Result<Vec<Download>> {
    let html = get_text("http://matrixcatalog.co.il/NBCompetitionRegulations.aspx").await?;
    let selector = Selector::parse("#download_content tr").unwrap();
    let document = Html::parse_document(&html);
    let downloads: Vec<Download> = FileInfo::from_str_iter(
        document
            .select(&selector)
            .skip(1) // skip header
            .into_iter()
            .filter(|td| {
                ElementRef::wrap(
                    td.children()
                        .into_iter()
                        .filter(|child| child.value().is_element())
                        .nth(1)
                        .unwrap(),
                )
                .unwrap()
                .text()
                .collect::<String>()
                    == chain
            })
            .map(|td| {
                ElementRef::wrap(
                    td.children()
                        .filter(|child| child.value().is_element())
                        .last()
                        .unwrap()
                        .children()
                        .find(|child| {
                            child.value().is_element()
                                && child.value().as_element().unwrap().name() == "a"
                        })
                        .unwrap(),
                )
                .and_then(|v| v.value().attr("href"))
                .unwrap()
                .to_string()
            }),
        file_limit,
    )
    .map(|fi| parallel_download::Download {
        dest: format!("data_raw/{}/{}", store.name, fi.filename),
        path: format!("{}{}", "http://matrixcatalog.co.il/", fi.source),
        headers: None,
    })
    .collect();

    Ok(downloads)
}

async fn get_downloads_shufersal(
    store: &Store,
    file_limit: Option<usize>,
) -> Result<Vec<Download>> {
    let html = get_text("http://prices.shufersal.co.il/FileObject/UpdateCategory?page=1").await?;
    let selector = Selector::parse("tfoot a").unwrap();
    let num_pages = Html::parse_document(&html)
        .select(&selector)
        .find(|&elem| elem.text().next() == Some(">>"))
        .ok_or(anyhow!("Cannot find link to Shufersal last page"))?
        .value()
        .attr("href")
        .ok_or(anyhow!("Cannot find href for Shufersal last page"))?
        .rsplit_once("=")
        .ok_or(anyhow!("No equal sign in Shufersal link"))?
        .1
        .parse::<usize>()?;

    let fetches = futures::stream::iter(1..(num_pages + 1))
        .map(|page| async move {
            let path =
                format!("http://prices.shufersal.co.il/FileObject/UpdateCategory?page={page}");
            match reqwest::get(&path).await {
                Ok(resp) => match resp.text().await {
                    Ok(html) => {
                        debug!("Success reading {path}");
                        let selector = Selector::parse("tbody a").unwrap();
                        Ok(Html::parse_document(&html)
                            .select(&selector)
                            .map(|elem| elem.value().attr("href").ok_or("err").unwrap().to_string())
                            .collect::<Vec<String>>())
                    }
                    Err(_) => Err(format!("ERROR reading {}", path)),
                },
                Err(_) => Err(format!("ERROR downloading {}", path)),
            }
        })
        .buffer_unordered(32)
        .collect::<Vec<Result<Vec<String>, String>>>();

    let mut all_links: HashSet<String> = HashSet::new();

    for result in fetches.await {
        match result {
            Ok(links) => {
                all_links.extend(links);
            }
            Err(e) => error!("Error: {e}"),
        }
    }

    let downloads: Vec<Download> = FileInfo::from_str_iter(all_links.into_iter(), file_limit)
        .map(|fi| parallel_download::Download {
            dest: format!("data_raw/{}/{}", store.name, fi.filename),
            path: fi.source,
            headers: None,
        })
        .collect();
    Ok(downloads)
}

#[instrument(fields(store_name=store.name), skip_all)]
async fn download_store_data(
    store: Store,
    quick: bool,
    file_limit: Option<usize>,
    download_semaphore: Arc<Semaphore>,
) -> Result<()> {
    info!("Start handling store");
    let downloads = match store.website {
        Website::PublishedPrice(username) => {
            get_downloads_publishedprice(
                &store,
                username,
                "",
                file_limit,
                download_semaphore.clone(),
            )
            .await?
        }
        Website::PublishedPriceWithPassword(username, password) => {
            get_downloads_publishedprice(
                &store,
                username,
                password,
                file_limit,
                download_semaphore.clone(),
            )
            .await?
        }
        Website::Shufersal => get_downloads_shufersal(&store, file_limit).await?,
        Website::SimpleJsonToGet(initial_url, download_prefix) => {
            get_downloads_simple_json_to_get(&store, file_limit, initial_url, download_prefix)
                .await?
        }
        Website::MatrixCatalog(chain) => {
            get_downloads_matrix_catalog(&store, file_limit, chain).await?
        }
        Website::PublishPrice(url) => get_downloads_publish_price(&store, file_limit, url).await?,
        Website::NetivHahesed => get_downloads_netiv_hahesed(&store, file_limit).await?,
        Website::SuperPharm => get_downloads_superpharm(&store, file_limit).await?,
    };
    info!("Found a total of {} elements", downloads.len());
    if quick {
        return Ok(());
    }
    parallel_download::parallel_download(downloads, download_semaphore).await;
    Ok(())
}

pub async fn download_all_stores_data(stores: &Vec<Store>, quick: bool, file_limit: Option<usize>) {
    let download_semaphore = Arc::new(Semaphore::new(30));
    let tasks: Vec<_> = stores
        .iter()
        .map(|store| {
            let span = Span::current();
            tokio::spawn(
                download_store_data(store.clone(), quick, file_limit, download_semaphore.clone())
                    .instrument(span),
            )
        })
        .collect();
    info!(
        "All tasks are spawned. Total tasks spawned: {}.",
        tasks.len()
    );
    for task in tasks {
        // let x = task.await;
        match task.await {
            Ok(Ok(())) => (),
            Ok(Err(err)) => error!("Error: {err}"),
            Err(err) => error!("Error: {err}"),
        };
    }
    info!("Processing complete.");
}
