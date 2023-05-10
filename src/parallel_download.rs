use bytes::Bytes;
use futures::StreamExt;
use metrics::increment_counter;
use reqwest::{header::HeaderMap, Client};
use std::fs::File;
use std::io::Cursor;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};
pub struct Download {
    pub store: String,
    pub path: String,
    pub headers: Option<HeaderMap>,
    pub dest: String,
}
fn write_file(content: &Bytes, dest: &str) -> std::io::Result<()> {
    let path = std::path::Path::new(&dest);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();

    let mut file = File::create(&path)?;
    let mut content = Cursor::new(content);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}
pub async fn parallel_download(downloads: Vec<Download>, download_semaphore: Arc<Semaphore>) {
    info!("Starting parallel download");
    futures::stream::iter(downloads)
        .map(|download| {
            increment_counter!("download_start", "store" => download.store.clone());
            let download_semaphore = download_semaphore.clone();
            async move {
                let mut should_retry = true;

                while should_retry {
                    let download_semaphore = download_semaphore.clone();
                    let store = download.store.clone();

                    let path = &download.path;
                    let client = Client::builder()
                        .timeout(std::time::Duration::from_secs(10))
                        .build()
                        .unwrap();
                    let mut client = client.get(path);
                    if let Some(headers) = &download.headers {
                        client = client.headers(headers.clone())
                    }
                    let dest = &download.dest;
                    let permit = match download_semaphore.acquire_owned().await {
                        Ok(permit) => permit,
                        Err(e) => {
                            debug!("Error in writing {dest}: {e}");
                            break;
                        }
                    };
                    should_retry = match client.send().await {
                        Ok(resp) => match resp.bytes().await {
                            Ok(content) => match write_file(&content, dest) {
                                Ok(()) => {
                                    debug!("Success in writing {dest}");
                                    increment_counter!("download_success", "store" => download.store.clone());
                                    false
                                }
                                Err(e) => {
                                    error!("Error in writing {dest}: {e}");
                                    increment_counter!("download_failure: writing", "store" => download.store.clone());
                                    false
                                }
                            },
                            Err(e) => {
                                warn!("ERROR reading {path}: {e}");
                                increment_counter!("download_failure: reading", "store" => download.store.clone());
                                true
                            }
                        },
                        Err(e) => {
                            debug!("ERROR downloading {path}: {e}");
                            increment_counter!("download_failure: downloading", "store" => download.store.clone());
                            true
                        }
                    };
                    if should_retry {
                        debug!("Retrying {path}");
                        increment_counter!("download retry", "store" => download.store.clone());
                    }
                    drop(permit);
                }
            }
        })
        .buffer_unordered(32)
        .collect::<Vec<()>>()
        .await;
}
