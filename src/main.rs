#[macro_use] // to parse --json in video.rs
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use clap::Parser; // cli.rs

mod cli;
mod recipe;
mod video;
// mod exec;

fn main() {
    cli::void_args();
    color_eyre::install().expect("Failed setting up error handler");

    let args = cli::Arguments::parse();

    let _rc = recipe::get_recipe(&args);
    let _videos = video::resolve_input(args);
    // exec::smoothing(videos);
}
