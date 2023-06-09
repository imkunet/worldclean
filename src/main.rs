use std::{sync::Arc, time::Instant};

use anyhow::Result;
use clap::Parser;
use log::info;
use logging::setup_logging;

use crate::cli::CleanOpts;

mod cli;
mod level;
mod level_data;
mod level_region;
mod logging;
mod region_iterator;

fn main() -> Result<()> {
    setup_logging();

    let options = Arc::from(CleanOpts::parse());
    info!(
        "worldclean {} by KuNet & contributors",
        env!("CARGO_PKG_VERSION")
    );

    if !options.world.is_dir() {
        panic!("The specified world is not a directory!");
    }

    let start = Instant::now();
    level::process_level(options)?;
    info!(
        "Finished process in {:?}",
        Instant::now().duration_since(start)
    );

    Ok(())
}
