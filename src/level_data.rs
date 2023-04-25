use anyhow::{bail, Context, Result};
use log::{info, warn};
use nbt::{decode::read_gzip_compound_tag, encode::write_gzip_compound_tag, CompoundTag};
use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
    sync::Arc,
};

use crate::cli::CleanOpts;

pub(crate) fn process_level_data(target_dir: &PathBuf, options: Arc<CleanOpts>) -> Result<()> {
    let mut target_file_reader = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(target_dir.join("level.dat"))
        .context("Could not create new level.dat")?;

    info!("Outputting world at {:#?}", target_dir);

    let mut level_file_reader = File::open(options.world.clone().join("level.dat"))
        .context("Could not resolve level.dat")?;
    let read = read_gzip_compound_tag(&mut level_file_reader)
        .context("level.dat is not in a readable format! (corrupt?!)")?;
    let Ok(data_tag) = read.get_compound_tag("Data") else {
      bail!("Unable to find the Data tag in level.dat");
  };

    info!(
        "Transforming level.dat for {}",
        data_tag.get_str("LevelName").unwrap_or("undefined")
    );

    let transformed = apply_level_data_transformations(data_tag)
        .context("Could not properly transform the data in level.dat")?;

    let mut new_root_compound_tag = CompoundTag::new();
    new_root_compound_tag.insert_compound_tag("Data", transformed);

    write_gzip_compound_tag(&mut target_file_reader, &new_root_compound_tag)
        .context("Could not properly encode new level.dat")?;

    Ok(())
}

fn apply_level_data_transformations(data_tag: &CompoundTag) -> Result<CompoundTag> {
    let mut root = CompoundTag::new();

    // sort for consistency
    let mut v: Vec<(&String, &nbt::Tag)> = data_tag.iter().collect();
    v.sort_by(|a, b| a.0.cmp(b.0));

    // look for things to change the values in here!
    for element in v.iter() {
        match element.0.as_str() {
            "allowCommands" => root.insert_bool("allowCommands", true),
            "generatorName" => root.insert_str("generatorName", "flat"),
            "generatorOptions" => root.insert_str("generatorOptions", "0;"),
            "rainTime" => root.insert_i32("rainTime", i32::MAX),
            "thunderTime" => root.insert_i32("thunderTime", i32::MAX),
            "raining" => root.insert_bool("raining", false),
            "thundering" => root.insert_bool("thundering", false),
            "GameRules" => {
                let nbt::Tag::Compound(game_rules) = element.1 else {
                  warn!("GameRules is not a valid CompoundTag");
                  continue;
              };

                let mut rules = CompoundTag::new();

                for sub_element in game_rules.iter() {
                    match sub_element.0.as_str() {
                        "doDaylightCycle" => rules.insert_str("doDaylightCycle", "false"),
                        "doMobSpawning" => rules.insert_str("doMobSpawning", "false"),
                        "mobGriefing" => rules.insert_str("mobGriefing", "false"),
                        "randomTickSpeed" => rules.insert_str("randomTickSpeed", "0"),
                        _ => rules.insert(sub_element.0, sub_element.1.clone()),
                    }
                }

                root.insert_compound_tag("GameRules", rules);
            }
            _ => root.insert(element.0, element.1.clone()),
        }
    }

    Ok(root)
}
