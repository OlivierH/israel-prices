use std::sync::Arc;

use reqwest::Client;
use tokio::sync::Semaphore;

pub async fn post_to_text_with_retries<'a>(
    client: &Client,
    url: &str,
    body: String,
    download_semaphore: Arc<Semaphore>,
) -> Option<String> {
    for _ in 0..10 {
        let _permit = download_semaphore.clone().acquire_owned().await.unwrap();

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
