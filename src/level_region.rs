use std::{
    fmt::Display,
    fs::File,
    io::Stdout,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use anvil_region::{
    position::{RegionChunkPosition, RegionPosition},
    provider::{FolderRegionProvider, RegionProvider},
    region::Region,
};
use anyhow::{bail, Context, Result};
use lazy_init::Lazy;
use log::{info, warn};
use nbt::CompoundTag;
use pbr::ProgressBar;
use rayon::prelude::*;

use crate::cli::CleanOpts;

struct PruneStats {
    prune_empty: AtomicU64,
    prune_invalid: AtomicU64,
}

impl PruneStats {
    fn new() -> Self {
        Self {
            prune_empty: AtomicU64::new(0),
            prune_invalid: AtomicU64::new(0),
        }
    }

    fn increment_empty(&self) {
        self.prune_empty.fetch_add(1, Ordering::Relaxed);
    }

    fn increment_invalid(&self) {
        self.prune_invalid.fetch_add(1, Ordering::Relaxed);
    }
}

impl Display for PruneStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Empty: {}, Invalid: {}",
            self.prune_empty.load(Ordering::Acquire),
            self.prune_invalid.load(Ordering::Acquire)
        )
    }
}

pub(crate) fn process_level_regions(target_dir: &Path, options: Arc<CleanOpts>) -> Result<()> {
    let path = options.world.join("region");

    let provider = FolderRegionProvider::new(
        path.to_str()
            .context("Could not locate region folder path")?,
    );

    let target_region_dir = target_dir.join("region");
    let target_provider = &FolderRegionProvider::new(
        target_region_dir
            .to_str()
            .context("Could not locate region folder for target path")?,
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
    println!();

    let stats = &PruneStats::new();

    region_positions
        .par_iter()
        .for_each(move |region_position| {
            let Ok(region) = provider.get_region(*region_position) else {
                panic!("Error in reading region {}, {}", region_position.x, region_position.z);
            };

            let Err(e) = process_region(target_provider, stats, region_position, region, &progress_bar) else {
                return;
            };

            panic!("Error in processing region! {:#?}", e);
        });

    info!("Prune stats:");
    info!("{}", stats);
    Ok(())
}

fn process_region(
    target_provider: &FolderRegionProvider,
    prune_stats: &PruneStats,
    region_position: &RegionPosition,
    region: Region<File>,
    progress_bar: &Mutex<ProgressBar<Stdout>>,
) -> Result<()> {
    let target_region: Lazy<Mutex<Region<File>>> = Lazy::new();

    region
        .into_iter()
        .enumerate()
        .par_bridge()
        .try_for_each_with(&target_region, |target_region, (i, chunk)| {
            let x = i % 32;
        let z = i / 32;

        let region_chunk_pos = RegionChunkPosition::new(x as u8, z as u8);

        let Ok(level) = chunk.get_compound_tag("Level") else {
            warn!("Skipping invalid chunk with no position or Level tag in region r:{:?} p:{:?}", region_position, region_chunk_pos);
            prune_stats.increment_invalid();
            return Ok(());
        };

        let Ok(light_populated) = level.get_bool("LightPopulated") else {
            warn!("Invalid chunk; could not get LightPopulated!");
            prune_stats.increment_invalid();
            return Ok(());
        };
        let Ok(terrain_populated) = level.get_bool("TerrainPopulated") else {
            warn!("Invalid chunk; could not get TerrainPopulated!");
            prune_stats.increment_invalid();
            return Ok(());
        };
        let Ok(entities) = level.get_compound_tag_vec("Entities") else {
            warn!("Invalid chunk; could not read Entities!");
            prune_stats.increment_invalid();
            return Ok(());
        };
        let Ok(tile_entities) = level.get_compound_tag_vec("TileEntities") else {
            warn!("Invalid chunk; could not read TileEntities!");
            prune_stats.increment_invalid();
            return Ok(());
        };

        if !light_populated && !terrain_populated && entities.is_empty() && tile_entities.is_empty()
        {
            // TODO: if it looks empty check one last time just to make sure it is actually empty
            prune_stats.increment_empty();
            return Ok(());
        }

        let mut level_tag = CompoundTag::new();
        for element in level.iter() {
            level_tag.insert(element.0, element.1.clone());
        }

        let mut chunk_tag = CompoundTag::new();
        chunk_tag.insert_compound_tag("Level", level_tag);

        let target = target_region.get_or_create(|| {
            Mutex::from(target_provider.get_region(*region_position)
            .with_context(|| {
                format!("Unable to create target region {:#?}", region_position)
            }).unwrap())
        });

        match target.write_chunk(region_chunk_pos, chunk_tag) {
            Ok(()) => {Ok(())}
            Err(_) => bail!("Error in writing chunk"), // TODO: comprehensive error
        }
    });

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
        println!();
    }

    Ok(())
}
