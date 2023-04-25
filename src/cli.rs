use std::path::PathBuf;

use clap::{arg, Parser};

#[derive(Parser)]
#[command(
    about = "Quickly and easily clean Minecraft ANVIL worlds.",
    author = "KuNet & contributors",
    version = env!("CARGO_PKG_VERSION"),
)]
pub(crate) struct CleanOpts {
    #[arg()]
    pub(crate) world: PathBuf,
    #[arg()]
    pub(crate) output: Option<PathBuf>,
}
