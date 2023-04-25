use anyhow::{anyhow, Context, Result};
use std::{fs, path::PathBuf, sync::Arc};

use crate::{cli::CleanOpts, level_data};

fn ensure_target(options: &CleanOpts) -> Result<PathBuf> {
    let target_dir = options
        .output
        .clone()
        .unwrap_or_else(|| {
            let parent_dir = options.world.parent().with_context(|| {
                "Could not find parent directory of world to find a place to put the new world"
            })?;

            let world_file_name = options.world.file_name().with_context(|| {
                "The world does not have a name for some reason"
            })?;

            parent_dir.join(format!("{}-clean", world_file_name.to_string_lossy()))
        });

    if target_dir.exists() {
        return Err(anyhow!("Target world already exists!"));
    }

    fs::create_dir(target_dir.clone())
        .with_context(|| "Could not create target directory")?;

    Ok(target_dir)
}

pub(crate) fn process_level(options: Arc<CleanOpts>) -> Result<()> {
    let target_dir = ensure_target(&options)?;

    level_data::process_level_data(&target_dir, &options)?;

    Ok(())
}