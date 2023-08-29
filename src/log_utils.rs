use std::fs::File;

use anyhow::Result;
use std::sync::Mutex;
use tracing_subscriber::prelude::*;
pub fn init() -> Result<()> {
    let log_file = File::create("log.txt")?;
    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(true)
                .with_line_number(true),
        )
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(true)
                .with_line_number(true)
                .with_writer(Mutex::new(log_file)),
        );

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
