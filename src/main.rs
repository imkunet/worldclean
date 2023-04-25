use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use log::{error, info};
use logging::setup_logging;

use crate::cli::CleanOpts;

mod cli;
mod level;
mod level_data;
mod logging;

fn main() -> Result<()> {
    setup_logging().context("Failed to initialize logger")?;

    let options = Arc::from(CleanOpts::parse());
    info!(
        "worldclean {} by KuNet & contributors",
        env!("CARGO_PKG_VERSION")
    );

    if let Err(err) = options.world.metadata() {
        error!("The specified world is not a directory: {}", err);
        return Err(err.into());
    }

    level::process_level(options)?;

    Ok(())
}