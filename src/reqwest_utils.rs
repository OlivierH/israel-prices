use std::sync::Arc;

use reqwest::Client;
use tokio::sync::Semaphore;

pub async fn post_to_text_with_retries(
    client: &Client,
    url: &str,
    body: String,
    download_semaphore: Option<Arc<Semaphore>>,
) -> Option<String> {
    for _ in 0..10 {
        let _permit = match download_semaphore.as_ref() {
            Some(download_semaphore) => {
                Some(download_semaphore.clone().acquire_owned().await.unwrap())
            }
            None => None,
        };

        match client
            .post(url)
            .header("content-type", "application/json;charset=UTF-8")
            .body(body.clone())
            .send()
            .await
        {
            Ok(resp) => match resp.text().await {
                Ok(text) => {
                    return Some(text);
                }
                Err(_) => continue,
            },
            Err(_) => continue,
        };
    }
    None
}

pub async fn post_to_text_with_headers_with_retries(
    client: &Client,
    url: &str,
    body: String,
    download_semaphore: Option<Arc<Semaphore>>,
    header_map: reqwest::header::HeaderMap,
) -> Option<String> {
    for _ in 0..10 {
        let _permit = match download_semaphore.as_ref() {
            Some(download_semaphore) => {
                Some(download_semaphore.clone().acquire_owned().await.unwrap())
            }
            None => None,
        };

        match client
            .post(url)
            .headers(header_map.clone())
            .header("content-type", "application/json;charset=UTF-8")
            .body(body.clone())
            .send()
            .await
        {
            Ok(resp) => match resp.text().await {
                Ok(text) => {
                    return Some(text);
                }
                Err(_) => continue,
            },
            Err(_) => continue,
        };
    }
    None
}

pub async fn get_to_text_with_retries(url: &str) -> Option<String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .unwrap();
    for _ in 0..10 {
        match client.get(url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => {
                    return Some(text);
                }
                Err(_) => continue,
            },
            Err(_) => continue,
        };
    }
    None
}

pub async fn get_json_to_text_with_retries(url: &str) -> Option<String> {
    let client = reqwest::Client::new();

    for _ in 0..10 {
        match client
            .get(url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await
        {
            Ok(resp) => match resp.text().await {
                Ok(text) => {
                    return Some(text);
                }
                Err(_) => continue,
            },
            Err(_) => continue,
        };
    }
    None
}
