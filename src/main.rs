#[macro_use] // to parse --json in video.rs
extern crate serde_derive;
extern crate clap;
extern crate serde;
extern crate serde_json;


use std::env;
use clap::Parser; // cli.rs

// use color_eyre::owo_colors::OwoColorize;
extern crate colored;
extern crate ffprobe; // cli wrapper

// rustsynth output
extern crate anyhow;
extern crate num_rational;

// in order of use
mod cli;
mod cmd;
mod parse;
mod recipe;
mod video;
// mod exec;
// mod output;

use crate::{cli::Arguments, recipe::Recipe, video::Payload};
use which::which;
fn main() {

    if enable_ansi_support::enable_ansi_support().is_err() {
        println!("Failed enabling ANSI color support, expect broken colors!")
    }

    parse::parse_update();


    let mut args: Arguments = cli::setup_args();;
    // args.input is the only one being mutated in video.rs

    let recipe: Recipe = recipe::get_recipe(&args);
    // loads defaults.ini, then overrides recipe.ini over it

    let payloads: Vec<Payload> = video::resolve_input(&mut args, &recipe);

    let _commands: Vec<(String, String)> = cmd::build_commands(args, payloads, recipe);

    // exec::_smoothing(payloads);
}
