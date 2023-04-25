use std::{
    fs::{self, File},
    sync::Arc,
};

use anyhow::{bail, Context, Result};
use clap::Parser;
use log::{info, warn};
use logging::setup_logging;
use nbt::{decode::read_gzip_compound_tag, encode::write_gzip_compound_tag, CompoundTag};

use crate::cli::CleanOpts;

mod cli;
mod logging;

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

    process_level_data(options)?;

    Ok(())
}

fn ensure_target(options: Arc<CleanOpts>) -> Result<File> {
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

    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(target_dir.join("level.dat"))
        .context("Could not create new level.dat")
}

fn process_level_data(options: Arc<CleanOpts>) -> Result<()> {
    let mut target_file_reader = ensure_target(options.clone())?;

    let mut level_file_reader = File::open(options.world.clone().join("level.dat"))
        .context("Could not resolve level.dat")?;
    let read = read_gzip_compound_tag(&mut level_file_reader)
        .context("level.dat is not in a readable format! (corrupt?!)")?;
    let Ok(data_tag) = read.get_compound_tag("Data") else {
        bail!("Unable to find the Data tag in level.dat");
    };

    info!(
        "Copying level.dat for {}",
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
