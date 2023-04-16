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

// progress bar, used in ffpb.rs
extern crate kdam;
extern crate regex;

mod cli;
mod cmd;
mod ffpb;
mod ffpb2;
mod parse;
mod recipe;
mod render;
mod utils;
mod vapoursynth;
mod video;

use crate::{cli::Arguments, cmd::SmCommand, recipe::Recipe, video::Payload};
use std::env;
use utils::verbosity_init;

fn main() {
    if enable_ansi_support::enable_ansi_support().is_err() {
        println!("Failed enabling ANSI color support, expect broken colors!")
    }

    // unused for now as it spams the API each time you launch it :/...
    // parse::parse_update();

    let mut args: Arguments = cli::setup_args();
    // args.input is the only one being mutated in video.rs

    let recipe: Recipe = recipe::get_recipe(&mut args);
    // mutable because args.verbose sets `[miscellaneous] always verbose:` to true
    // loads defaults.ini, then overrides recipe.ini over it

    verbosity_init(
        args.verbose,
        recipe.get_bool("miscellaneous", "always verbose"),
    );

    let is_conhost: bool = (env::var("WT_SESSION").is_err() && env::var("ALACRITY_LOG").is_err())
        || env::var("NO_SMOOTHIE_WIN32").is_ok();
    // user is neither running Windows Terminal and alacritty, OR has NO_SMOOTHIE_WIN32 defined

    #[cfg(target_os = "windows")]
    if args.tui
        && is_conhost
        && !recipe.get_bool("miscellaneous", "always verbose")
        && !args.verbose
    {
        utils::set_window_position(&recipe);
    }

    let payloads: Vec<Payload> = video::resolve_input(&mut args, &recipe);

    let commands: Vec<SmCommand> = cmd::build_commands(args, payloads, recipe);

    render::_vpipe_render2(commands);
}
