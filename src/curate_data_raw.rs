use anyhow::Result;
use tracing::{debug, error, info, span, Level};

fn run(command: &str) -> Result<()> {
    let span = span!(Level::INFO, "Run command", command);
    let _enter = span.enter();
    debug!("Start");
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()?;
    if !output.stdout.is_empty() {
        debug!(output = String::from_utf8(output.stdout)?, "Output");
    }
    if !output.stderr.is_empty() {
        error!(error = String::from_utf8(output.stderr)?, "Error",);
    }
    Ok(())
}

pub fn curate_data_raw() -> Result<()> {
    let span = span!(Level::INFO, "curate_data_raw");
    let _enter = span.enter();
    // Rami levy has two different stores files, one of them with a single store that is already present in the first stores file.
    info!("Deleting superfluous and incomplete Rami levy store file");
    run("rm data_raw/rami_levy/storesfull* -f")?;

    info!("Deleting empty files");
    run("find data_raw -type f -empty -print -delete")?;

    info!("Deleting x1 files");
    run("find data_raw -type f -name \"*.x1\" -print -delete")?;

    // Note: we need a loop here in case a gz file is malformed and causes an error
    info!("Unzipping all files");
    run("for file in data_raw/*/*.gz; do gunzip $file; done")?;
    // run("gunzip data_raw/*/*.gz")?;

    Ok(())
}
