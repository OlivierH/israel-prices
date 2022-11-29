use anyhow::Result;
use slog::{self, debug, info, o, Drain, Logger};
use slog_async;
use slog_term;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = slog::Logger::root(drain, o!("P" => "shufersal_annotator"));

    let code: i64 = 5053827215862;

    let url = format!("https://www.shufersal.co.il/online/he/p/P_{code}/json");

    debug!(log, "Fetching {url}");

    let client = reqwest::Client::new();
    let html = client.get(url).send().await?.text().await?;
    println!("{html}");
    // let document = Html::parse_document(&html);
    // let selector = Selector::parse("meta[name=\"csrftoken\"]").unwrap();

    // scraper::Document::from_

    println!("Hello, world!");
    Ok(())
}
