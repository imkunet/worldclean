use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "worldclean", about = "Quickly and easily clean Minecraft ANVIL worlds.")]
struct CleanOpts {
    #[structopt(help = "The path to the world directory")]
    world: PathBuf,

    #[structopt(
    short,
    long,
    help = "The path to the output directory. Defaults to the same as the input directory."
    )]
    output: Option<PathBuf>,
}

fn main() -> std::io::Result<()> {
    let opts = CleanOpts::from_args();
    println!("{:?}", opts);

    if !opts.world.is_dir() {
        eprintln!("Error: The specified world directory '{}' does not exist", opts.world.display());
        return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
    }

    Ok(())
}