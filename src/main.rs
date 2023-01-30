#[macro_use] // to parse --json in video.rs
extern crate serde_derive;
extern crate clap;
extern crate ffprobe;
extern crate serde;
extern crate serde_json;

use clap::Parser; // cli.rs

// structs, in order of use
use crate::cli::Arguments;
use crate::cmd::Command;
use crate::recipe::Recipe;
use crate::video::Payload;

mod cli;
mod cmd;
mod exec;
mod parse;
mod recipe;
mod video;
// mod exec;

fn main() {
    cli::void_args();

    let mut args: Arguments = cli::Arguments::parse();
    // mutable because args.input is cleaned up from non-existent files

    let recipe: Recipe = recipe::get_recipe(&args);

    let payloads: Vec<Payload> = video::resolve_input(&mut args, &recipe);
    // probe_input used to return valid video file paths and overwrites args.input

    let _commands: Vec<Command> = cmd::build_commands(args, payloads, recipe);

    /*exec::smoothing(videos); WIP */
}
