// #![windows_subsystem = "windows"]

#[macro_use] // to parse --json in video.rs
extern crate serde_derive;

use recipe::Recipe;
use winapi::um::{wincon::GetConsoleWindow, winuser::ShowWindow};

mod cli;
mod cmd;
mod smgui;
// mod ffpb;
// mod ffpb2;
mod parse;
mod recipe;
mod render;
mod utils;
//mod vapoursynth;
mod video;

use crate::{cli::Arguments, cmd::SmCommand, video::Payload};
use std::{env, sync::mpsc::channel};
use utils::verbosity_init;

const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "webm", "mov", "avi", "wmv", "flv", "ts", "m3u8", "qt", "m4v",
];

const YES: &[&str] = &[
    "on", "True", "true", "yes", "y", "1", "yeah", "yea", "yep", "sure", "positive",
];

const NO: &[&str] = &[
    "off", "False", "false", "no", "n", "nah", "nope", "negative", "negatory", "0", "0.0", "null",
    "", " ", "  ", "\t", "none",
];

fn main() {
    if enable_ansi_support::enable_ansi_support().is_err() {
        println!("Failed enabling ANSI color support, expect broken colors!")
    }

    // unused for now as it spams the API each time you launch it :/...
    // parse::parse_update();

    let mut args: Arguments = cli::setup_args();
    // args.input is the only one being mutated in video.rs

    // Recipe and WidgetMetadata
    let (recipe, _metadata) = recipe::get_recipe(&mut args);
    // mutable because args.verbose sets `[miscellaneous] always verbose:` to true
    // loads defaults.ini, then overrides recipe.ini over it

    verbosity_init(
        args.verbose,
        recipe.get_bool("miscellaneous", "always verbose"),
    );

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

    let payloads: Vec<Payload>;
    let (recipe, mut args) = if args.input.is_empty() && !args.tui {
        let hwnd: Option<*mut winapi::shared::windef::HWND__> = if cfg!(windows) {
            unsafe {
                let hwnd = GetConsoleWindow();
                if !hwnd.is_null() {
                    Some(hwnd)
                } else {
                    None
                }
            }
        } else {
            None
        };

        if let Some(hwnd) = hwnd {
            unsafe {
                ShowWindow(hwnd, winapi::um::winuser::SW_MINIMIZE);
            }
        }

        let (sender, receiver) = channel::<(Recipe, Arguments)>();

        if let Err(e) = smgui::sm_gui(recipe.clone(), _metadata, args.recipe.clone(), args, sender)
        {
            println!("smoothie egui failed:\n\n{}", e);
        };

        // for video in receiver.recv().ok().unwrap() {
        //     dbg!(&video.payload.basename);
        // }

        if let Some(hwnd) = hwnd {
            unsafe {
                ShowWindow(hwnd, winapi::um::winuser::SW_RESTORE);
            }
        }

        if let Ok((args, recipe)) = receiver.recv() {
            (args, recipe)
        } else {
            panic!("Failed retrieving data from GUI");
            // this also 
        }
    } else {
        // data was already retrieved from CLI, just pass them back
        (recipe, args)
    };

    payloads = video::resolve_input(&mut args, &recipe);
    let commands: Vec<SmCommand> = cmd::build_commands(args, payloads, recipe);
    render::vspipe_render(commands);
}
