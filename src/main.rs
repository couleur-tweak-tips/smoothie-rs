#[macro_use] // to parse --json in video.rs
extern crate serde_derive;
extern crate ffprobe;
extern crate serde;
extern crate serde_json;
#[allow(unused_imports)]
use std::path::PathBuf;

use clap::Parser; // cli.rs

mod cli;
mod parse;
mod recipe;
mod video;
// mod exec;

fn main() {
    cli::void_args();
    color_eyre::install().expect("Failed setting up error handler");

    let mut args = cli::Arguments::parse();
    // mut bc args.input is cleaned up from non-existant files

    let mut rc: crate::recipe::Recipe = recipe::get_recipe(&mut args);
    // mut bc --override

    let _videos = video::resolve_input(&mut args, &mut rc);
    // exec::smoothing(videos);
}
