use anyhow::{bail, Context, Result};
use std::{fs, path::PathBuf, sync::Arc};

use crate::{cli::CleanOpts, level_data};

fn ensure_target(options: Arc<CleanOpts>) -> Result<PathBuf> {
    let target_dir = match options.output.clone() {
        Some(thing) => thing,
        None => {
            let parent_dir = options.world.parent().context(
                "Could not find parent directory of world to find a place to put the new world",
            )?;

            let world_file_name = options
                .world
                .file_name()
                .context("The world does not have a name for some reason")?;

            parent_dir.join(format!(
                "{}-clean",
                world_file_name
                    .to_str()
                    .context("Could not convert the name of the original world file to a string")?
            ))
        }
    };

    if target_dir.exists() {
        bail!("Target world already exists!");
    }

    fs::create_dir(target_dir.clone()).context("Could not create target directory")?;

    Ok(target_dir)
}

pub(crate) fn process_level(options: Arc<CleanOpts>) -> Result<()> {
    let target_dir = ensure_target(options.clone())?;

    level_data::process_level_data(&target_dir, options)?;

    Ok(())
}
