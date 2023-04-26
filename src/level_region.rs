use std::{
    fs::File,
    io::Stdout,
    path::Path,
    sync::{Arc, Mutex},
};

use anvil_region::{
    position::RegionPosition,
    provider::{FolderRegionProvider, RegionProvider},
    region::Region,
};
use anyhow::{bail, Context, Result};
use log::{info, warn};
use pbr::ProgressBar;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::cli::CleanOpts;

pub(crate) fn process_level_regions(target_dir: &Path, options: Arc<CleanOpts>) -> Result<()> {
    let path = options.world.join("region");

    let provider = FolderRegionProvider::new(
        path.to_str()
            .context("Could not locate region folder path")?,
    );

    let region_positions: Vec<RegionPosition> = provider
        .iter_positions()
        .context("Could not fetch all regions")?
        .collect();

    let mut progress_bar = ProgressBar::new(region_positions.len() as u64);
    progress_bar.set_units(pbr::Units::Default);
    progress_bar.message("⏣ Processing Regions ");
    progress_bar.format("▕█▓░▏");
    progress_bar.tick_format("◜◜◜◠◠◝◝◝◞◞◞◡◡◟◟◟");

    let progress_bar = Mutex::new(progress_bar);

    info!("Beginning to process {} regions", region_positions.len());

    region_positions
        .par_iter()
        .for_each(move |region_position| {
            let Ok(region) = provider.get_region(*region_position) else {
                panic!("Error in reading region {}, {}", region_position.x, region_position.z);
            };

            let Err(e) = process_region(region, &progress_bar) else {
                return;
            };

            panic!("Error in processing region! {:#?}", e);
        });

    Ok(())
}

fn process_region(region: Region<File>, progress_bar: &Mutex<ProgressBar<Stdout>>) -> Result<()> {
    for chunk in region.into_iter() {
        let Ok(level) = chunk.get_compound_tag("Level") else {
            warn!("This chunk got no level ???");
            continue;
        };

        let Ok(light_populated) = level.get_bool("LightPopulated") else {
            warn!("Invalid chunk; could not get LightPopulated! {},{}", chunk.get_i32("xPos").unwrap_or(0), chunk.get_i32("zPos").unwrap_or(0));
            continue;
        };
        let Ok(terrain_populated) = level.get_bool("TerrainPopulated") else {
            warn!("Invalid chunk; could not get TerrainPopulated!");
            continue;
        };
        let Ok(entities) = level.get_compound_tag_vec("Entities") else {
            warn!("Invalid chunk; could not read Entities!");
            continue;
        };

        if !light_populated && !terrain_populated && entities.is_empty() {
            // if it looks empty check one last time just to make sure it is actually empty
        }
    }
    increment_progress_bar(progress_bar)?;

    Ok(())
}

fn increment_progress_bar(progress_bar: &Mutex<ProgressBar<Stdout>>) -> Result<()> {
    let Ok(mut progress_bar) = progress_bar.lock() else {
        bail!("Unable to lock progress bar!");
    };

    if progress_bar.inc() >= progress_bar.total - 1 {
        progress_bar.show_tick = false;
        progress_bar.message("✔ Processing Regions ");
        progress_bar.finish();
    }

    Ok(())
}
