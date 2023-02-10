#[macro_use] // to parse --json in video.rs
extern crate serde_derive;
extern crate clap;
extern crate serde;
extern crate serde_json;

// use color_eyre::owo_colors::OwoColorize;
extern crate colored;
extern crate ffprobe; // cli wrapper

// rustsynth output
extern crate anyhow;
extern crate num_rational;

// rust2c binder
// extern crate cty;

mod cli;
mod cmd;
mod exec;
mod parse;
mod recipe;
mod vapoursynth;
mod video;
// mod exec;
// mod output;

use crate::{cli::Arguments, cmd::SmCommand, recipe::Recipe, video::Payload};

use std::os::raw::c_int;

extern "C" {
    pub fn SetConsoleParams(
        borderless: c_int,
        always_on_top: c_int,
        wnd_position: c_int,
        wnd_cx: c_int,
        wnd_cy: c_int,
    );
}

fn main() {
    if enable_ansi_support::enable_ansi_support().is_err() {
        println!("Failed enabling ANSI color support, expect broken colors!")
    }

    parse::parse_update();

    let mut args: Arguments = cli::setup_args();
    // args.input is the only one being mutated in video.rs

    let recipe: Recipe = recipe::get_recipe(&args);
    // loads defaults.ini, then overrides recipe.ini over it

    if cfg!(target_os = "windows") && !recipe.get_bool("miscellaneous", "always verbose") {
        #[rustfmt::skip]
        let pos = {

            let pos = recipe.get("console", "position");

            match pos.as_str() {
                "top left"     | "top_left"     | "top-left"     | "topleft"    |"tl" => 0 as c_int,
                "bottom left"  | "bottom_left"  | "bottom-left"  | "bottomleft" |"bl" => 1 as c_int,
                "top right"    | "top_right"    | "top-right"    | "topright"   |"tr" => 2 as c_int,
                "bottom right" | "bottom_right" | "bottom-right" | "bottomright"|"br" => 3 as c_int,
                _ => {
                    println!("Unknown position `{:?}`, defaulting to `top left`", pos);
                    0 as c_int
                }
            }
        };
        unsafe {
            SetConsoleParams(
                recipe.get_bool("console", "borderless") as c_int,
                recipe.get_bool("console", "stay on top") as c_int,
                pos,
                {
                    match recipe.get("console", "width").parse::<c_int>() {
                        Ok(height) => height,
                        Err(e) => {
                            println!("Failed parsing `[console] width:` to an integer, defaulting to 800");
                            800 as c_int
                        }
                    }
                },
                {
                    match recipe.get("console", "height").parse::<c_int>() {
                        Ok(height) => height,
                        Err(e) => {
                            println!("Failed parsing `[console] height:` to an integer, defaulting to 600");
                            600 as c_int
                        }
                    }
                },
            );
        }
    }

    let payloads: Vec<Payload> = video::resolve_input(&mut args, &recipe);

    let _commands: Vec<SmCommand> = cmd::build_commands(args, payloads, recipe);

    exec::_smoothing(_commands);
}
