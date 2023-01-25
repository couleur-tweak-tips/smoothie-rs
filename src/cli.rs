use clap::Parser;
use open_file;
use std::{env, path::PathBuf, process::Command};
/// Smoothen up your gameplay footage with Smoothie, yum!
#[derive(Parser, Debug)]
#[clap(about, long_about = "", arg_required_else_help = true)]
pub struct Arguments {
    // io
    /// Input video file paths, quoted and separated by strings
    #[clap(short, long, conflicts_with="json", num_args=1..)]
    pub input: Vec<PathBuf>,

    /// Single output video file path
    #[clap(short, long, visible_alias = "out")]
    pub output: Option<String>,

    /// Makes sm behave like an app instead of a CLI tool (e.g pause before exitting on an error)
    #[clap(short, long, default_value_t = false)]
    pub tui: bool,

    // /// Override the output directory for all files
    // #[clap(short, long, visible_alias="outd")]
    // pub outdir: PathBuf,

    // /// Overrides output to an image of frame number passed
    // #[clap(short, long, conflicts_with="encargs")]
    // pub peek: i32,

    // misc io
    /// Discard any audio tracks that'd pass to output
    #[clap(visible_alias = "an", long, default_value_t = false)]
    pub stripaudio: bool,

    /// Redirect VS' Y4M output to null
    #[clap(
        visible_alias = "tn",
        long,
        default_value_t = false,
        conflicts_with = "tompv"
    )]
    pub tonull: bool,

    /// Redirect VS' Y4M output to MPV (video player)
    #[clap(
        visible_alias = "tm",
        long,
        default_value_t = false,
        conflicts_with = "tonull"
    )]
    pub tompv: bool,

    // external script/extensions
    /// Payload containing video timecodes, used by NLE scripts
    #[clap(long, conflicts_with = "input")]
    pub json: Option<String>,

    /// Split all cuts to separate files, used with -json
    #[clap(
        long,
        default_value_t = false,
        requires = "json",
        conflicts_with = "input"
    )]
    pub split: bool,

    /// Join all cuts to a file, used with -json
    #[clap(
        long,
        default_value_t = false,
        requires = "json",
        conflicts_with = "input"
    )]
    pub trim: bool,

    /// Keep same length (black cuts in between), used with -json
    #[clap(
        long,
        default_value_t = false,
        requires = "json",
        conflicts_with = "input"
    )]
    pub padding: bool,

    // debugging
    /// Display details about recipe, what I personally use
    #[clap(short, long, default_value_t = false)]
    pub verb: bool,

    /// Makes sm behave like an app instead of a CLI
    #[clap(visible_alias = "apm", long, default_value_t = false)]
    pub appmode: bool,

    /// Prints all the nerdy stuff to find bugs
    #[clap(visible_alias = "db", long, default_value_t = false)]
    pub debug: bool,

    /// Specify a recipe path
    #[clap(visible_alias = "!!", long, default_value_t = false)]
    pub rerun: bool,

    // customization
    /// Override FFmpeg encoding arguments
    #[clap(
        visible_alias = "enc",
        long,
        default_value_t = false,
        conflicts_with = "tompv",
        conflicts_with = "tonull"
    )]
    pub encargs: bool,

    /// Specify a recipe path
    #[clap(short, long, default_value = "recipe.ini")]
    pub recipe: String,

    /// Override recipe setting(s), e.g: --ov "flowblur;amount;40" "misc;container;MKV"
    #[clap(visible_alias="ov", visible_alias="overide", long, default_value = "", num_args=1..)]
    pub r#override: String,
}

pub fn void_args() {
    let first_arg = match std::env::args().nth(1) {
        Some(arg) => arg,
        None => return,
    };

    let current_exe = env::current_exe().expect("Could not determine exe");
    let current_exe_path = current_exe.parent().unwrap();

    match first_arg.as_ref() {
        "rc" | "recipe" | "conf" | "config" => {
            let ini_path = current_exe_path.join("..").join("recipe.ini");

            if !ini_path.exists() {
                panic!("Could not find recipe at {:?}", ini_path)
            }

            println!("Opening recipe: {}", ini_path.to_str().unwrap());
            open_file::open(ini_path.display().to_string(), None);
        }
        "root" | "dir" | "folder" => {
            println!("Opening directory");
            if cfg!(target_os = "windows") {
                println!("Hi Windows!");
                Command::new("explorer.exe")
                    .args([current_exe_path.parent().expect(
                        "Failed to get smoothie's parent directory, is it in a drive's root folder?",
                    )])
                    .output()
                    .expect("failed to execute process");
            } else {
                println!(
                    "The smoothie binary is located at {}",
                    current_exe_path.display()
                );
            }
        }
        _ => return,
    }

    std::process::exit(0);
}
