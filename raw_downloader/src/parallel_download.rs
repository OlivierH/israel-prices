use bytes::Bytes;
use futures::StreamExt;
use reqwest::{header::HeaderMap, Client};
use slog::{debug, info, Logger};
use std::fs::File;
use std::io::Cursor;
pub struct Download {
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
pub async fn parallel_download(downloads: Vec<Download>, log: &Logger) {
    info!(log, "Starting parallel download");
    futures::stream::iter(downloads)
        .map(|download| {
            let log = log.clone();
            async move {
                let mut should_retry = true;

                while should_retry {
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

                    should_retry = match client.send().await {
                        Ok(resp) => match resp.bytes().await {
                            Ok(content) => match write_file(&content, dest) {
                                Ok(()) => {
                                    debug!(log, "Success in writing {dest}");
                                    false
                                }
                                Err(e) => {
                                    debug!(log, "Error in writing {dest}: {e}");
                                    false
                                }
                            },
                            Err(e) => {
                                debug!(log, "ERROR reading {path}: {e}");
                                false
                            }
                        },
                        Err(e) => {
                            debug!(log, "ERROR downloading {path}: {e}");
                            true
                        }
                    };
                    if should_retry {
                        debug!(log, "Retrying {path}");
                    }
                }
            }
        })
        .buffer_unordered(32)
        .collect::<Vec<()>>()
        .await;
}
