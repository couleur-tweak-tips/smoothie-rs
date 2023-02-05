#[macro_use] // to parse --json in video.rs
extern crate serde_derive;
extern crate clap;
extern crate ffprobe;
extern crate serde;
extern crate serde_json;
// extern crate tokio;

extern crate anyhow;
extern crate num_rational;

use clap::Parser; // cli.rs
                  // use color_eyre::owo_colors::OwoColorize;
extern crate colored; // not needed in Rust 2018

// structs, in order of use
use crate::cli::Arguments;
// use crate::cmd::Command;
use crate::recipe::Recipe;
use crate::video::Payload;

mod cli;
// mod output;
// mod cmd;
// mod exec;
mod parse;
mod recipe;
mod video;

fn main() {
    match enable_ansi_support::enable_ansi_support() {
        Ok(()) => {}
        Err(_) => {
            // The operation was unsuccessful, typically because it's running on an older
            // version of Windows. The program may choose to disable ANSI color code output in
            // this case.
            println!("Failed enabling ANSI color support, expect broken colors!")
        }
    }

    parse::parse_update();
    cli::void_args();

    let mut args: Arguments = cli::Arguments::parse();
    // mutable because args.input is cleaned up from non-existent files

    let recipe: Recipe = recipe::get_recipe(&args);

    let payloads: Vec<Payload> = video::resolve_input(&mut args, &recipe);
    // probe_input used to return valid video file paths and overwrites args.input

    // let _commands: Vec<Command> = cmd::build_commands(args, payloads, recipe);

    // exec::_smoothing(payloads);
}
