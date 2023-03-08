#[macro_use] // to parse --json in video.rs
extern crate serde_derive;
extern crate clap;
extern crate serde;
extern crate serde_json;

extern crate colored;
extern crate ffprobe; // cli wrapper

// rustsynth output
extern crate anyhow;
extern crate num_rational;

mod cli;
mod cmd;
mod parse;
mod recipe;
mod render;
mod utils;
mod vapoursynth;
mod video;

use crate::{cli::Arguments, cmd::SmCommand, recipe::Recipe, video::Payload};
use std::env;

fn main() {
    if enable_ansi_support::enable_ansi_support().is_err() {
        println!("Failed enabling ANSI color support, expect broken colors!")
    }

    // parse::parse_update();

    let mut args: Arguments = cli::setup_args();
    // args.input is the only one being mutated in video.rs

    let recipe: Recipe = recipe::get_recipe(&args);
    // loads defaults.ini, then overrides recipe.ini over it

    let is_conhost: bool = (env::var("WT_SESSION").is_err() && env::var("ALACRITY_LOG").is_err())
        || env::var("NO_SMOOTHIE_WIN32").is_ok();
    // user is neither running Windows Terminal and alacritty, OR has NO_SMOOTHIE_WIN32 defined

    if args.tui
        && is_conhost
        && cfg!(target_os = "windows")
        && !recipe.get_bool("miscellaneous", "always verbose")
        && !args.verbose
    {
        utils::set_window_position(&recipe);
    }

    let payloads: Vec<Payload> = video::resolve_input(&mut args, &recipe);

    let commands: Vec<SmCommand> = cmd::build_commands(args, payloads, recipe);

    render::vspipe_render(commands);
}
