use crate::portable;
use clap::Parser;
use std::fs::File;
use std::io::{Read, Write};
use std::{env, path::PathBuf, process::Command};

/// Smoothen up your gameplay footage with Smoothie, yum!
#[derive(Parser, Debug, Clone)]
#[clap(about, long_about = "", arg_required_else_help = false)]
pub struct Arguments {
    // io
    /// Input video file paths, quoted and separated by strings
    #[clap(short, long, conflicts_with="json", num_args=1..)]
    pub input: Vec<PathBuf>,

    /// Single output video file path
    #[clap(
        short,
        long,
        visible_alias = "out",
        conflicts_with = "tompv",
        conflicts_with = "tonull"
    )]
    pub output: Option<String>,

    /// Makes sm behave like an app instead of a CLI tool (e.g pause before exiting on an error)
    #[clap(short, long, default_value_t = false)]
    pub tui: bool,

    /// Override the output directory for all files
    #[clap(long, visible_alias = "outd")]
    pub outdir: Option<PathBuf>,

    /// Overrides output to an image of frame number passed
    #[clap(long)]
    pub peek: Option<u64>,

    /// Pass a .vpy script to evaluate nodes from
    #[clap(long, default_value = "jamba.vpy")]
    pub vpy: PathBuf,

    /// Pass a VSPipe executable to run scripts from
    #[clap(long)]
    pub vspipe_path: Option<PathBuf>,

    // misc io
    /// Discard any audio tracks that'd pass to output
    #[clap(long, visible_alias = "an", default_value_t = false)]
    pub stripaudio: bool,

    /// Redirect VS' Y4M output to null
    #[clap(
        visible_alias = "tn",
        long,
        default_value_t = false,
        conflicts_with = "tompv",
        conflicts_with = "output"
    )]
    pub tonull: bool,

    /// Redirect VS' Y4M output to mpv (video player)
    #[clap(
        visible_alias = "tm",
        long,
        default_value_t = false,
        conflicts_with = "tonull",
        conflicts_with = "output"
    )]
    pub tompv: bool,

    /// return recipe string, formatted for jamba.vpy
    #[clap(long)]
    pub return_recipe: bool,

    // external script/extensions
    /// Payload containing video timecodes, used by NLE scripts
    #[clap(long, conflicts_with = "input")]
    pub json: Option<String>,

    /// New json is HashMap<Path, (start,fin)>, old is Vec<(PathBuf, start, fin)>
    #[clap(long, default_value_t = false)]
    pub old_json: bool,

    /// Join all cuts to a file, used with -json
    #[clap(
        long,
        default_value_t = false,
        requires = "json",
        conflicts_with = "input",
        conflicts_with = "padding"
    )]
    pub trim: bool,

    /// Keep same length (black cuts in between), used with -json
    #[clap(
        long,
        default_value_t = false,
        requires = "json",
        conflicts_with = "input",
        conflicts_with = "trim"
    )]
    pub padding: bool,

    // debugging
    /// Display progress bar
    #[clap(long, default_value_t = false)]
    pub progress: bool,

    /// Display details about recipe, what I personally use
    #[clap(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Prints all the nerdy stuff to find bugs
    #[clap(visible_alias = "db", long, default_value_t = false)]
    pub debug: bool,

    /// Specify a recipe path
    #[clap(visible_alias = "!!", long, default_value_t = false)]
    pub rerun: bool,

    /// Override FFmpeg encoding arguments (prefer using --override)
    #[clap(
        alias = "enc-args",
        visible_alias = "enc",
        long,
        conflicts_with = "tompv",
        conflicts_with = "tonull"
    )]
    pub encargs: Option<String>,

    /// Specify a recipe path
    #[clap(
        short,
        long,
        default_value = "recipe.ini",
        conflicts_with = "recipe_str"
    )]
    pub recipe: String,

    /// Specify a recipe string
    #[clap(long, conflicts_with = "recipe")]
    pub recipe_str: Option<String>,

    /// Override recipe setting(s), e.g: --ov "flowblur;amount;40" "misc;container;MKV"
    #[clap(visible_alias="ov", alias="overide", long, num_args=1..)]
    pub r#override: Option<Vec<String>>,
}

pub fn setup_args() -> Arguments {
    if cfg!(debug_assertions) {
        color_eyre::install().expect("Failed setting up error handler");
    } else {
        std::panic::set_hook(Box::new(|panic_info| {
            let payload = panic_info.payload();
            let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                s
            } else if let Some(s) = payload.downcast_ref::<String>() {
                s
            } else {
                "Unknown panic payload"
            };
            let (file, line, column) = match panic_info.location() {
                Some(loc) => (loc.file(), loc.line().to_string(), loc.column().to_string()),
                None => ("?", "?".into(), "?".into()),
            };
            let args: Vec<_> = env::args().collect();
            rfd::MessageDialog::new()
                .set_title("Smoothie crashed!")
                .set_description(&format!(
r#"Error message:
{msg}

Location in source:
{file}:{line}:{column}

Arguments passed:
{args:?}

Note: If your PC is still going BRRR the video might still be rendering :)

Common errors are listed on the troubleshooting page at ctt.cx/smoothie  

If you'd like help, take a screenshot of this message and your recipe and come over to discord.gg/CTT and make a post in #support
                    "#))
                .set_level(rfd::MessageLevel::Error)
                .show();
        }));
    }

    let first_arg = match env::args().nth(1) {
        Some(arg) => arg,
        None => "".to_string(),
    };

    let current_exe = env::current_exe().expect("Could not determine exe");
    let current_exe_path = current_exe
        .parent()
        .expect("Could not get directory of executable");

    let mut last_args = portable::get_last_args_path();

    if !last_args.exists() {
        if File::create(&last_args).is_err() {
            panic!("Failed to create last_args.txt at {last_args:?}")
        };
    }

    match first_arg.as_ref() {
        "enc" | "encoding" | "presets" | "encpresets" | "macros" => {
            let presets_path = portable::get_encoding_presets_path();
            if !presets_path.exists() {
                panic!(
                    "Could not find encoding presets (expected at {})",
                    presets_path.display()
                )
            }

            let ini_path = presets_path.canonicalize().unwrap().display().to_string();

            match opener::open(&ini_path) {
                Ok(()) => {
                    std::process::exit(0);
                }
                Err(e) => {
                    panic!("Error {e}\n\nFailed opening file {:?}", ini_path);
                }
            }
        }
        "def" | "default" | "defaults" => {
            let presets_path = portable::get_defaults_path();

            if !presets_path.exists() {
                panic!(
                    "Could not find defaults.ini (expected at {})",
                    presets_path.display()
                )
            }

            let ini_path = presets_path.canonicalize().unwrap().display().to_string();

            match opener::open(&ini_path) {
                Ok(()) => {
                    std::process::exit(0);
                }
                Err(e) => {
                    panic!("Error {e}\n\nFailed opening file {:?}", ini_path);
                }
            }
        }
        "rc" | "recipe" | "conf" | "config" => {
            let ini_path = portable::get_recipe_path();
            if !ini_path.exists() {
                panic!("Could not find recipe at {}", ini_path.display())
            }

            let ini_path = ini_path.canonicalize().unwrap().display().to_string();

            match opener::open(&ini_path) {
                Ok(()) => {
                    std::process::exit(0);
                }
                Err(e) => {
                    panic!("Error {e}\n\nFailed opening file {:?}", ini_path);
                }
            }
        }
        "root" | "dir" | "folder" => {
            if cfg!(target_os = "windows") {
                Command::new("explorer.exe")
                    .args([current_exe_path.parent().expect(
                        "Failed to get smoothie's parent directory, is it in a drive's root folder?",
                    )])
                    .output()
                    .expect("Failed to execute explorer process for dir");
            } else {
                println!(
                    "The smoothie binary is located at {}",
                    current_exe_path.display()
                );
            }
            std::process::exit(0);
        }
        "!!" | "-!!" | "--!!" | "-rerun" | "--rerun" => {
            let mut file = match File::open(&last_args) {
                Ok(file) => file,
                Err(e) => panic!("Error opening last_args.txt: {}", e),
            };
            let mut content = String::new();
            match file.read_to_string(&mut content) {
                Ok(_) => (),
                Err(e) => panic!("Error reading last_args.txt: {}", e),
            };
            let last_args_lines: Vec<&str> = content.lines().collect();
            dbg!(&last_args_lines);
            match Arguments::try_parse_from(last_args_lines) {
                Ok(args) => args,
                Err(e) => panic!("{}", e),
            }
        }
        _ => {
            let mut file = match File::create(&mut last_args) {
                Ok(file) => file,
                Err(e) => panic!("Error opening last_args.txt: {}", e),
            };

            for arg in env::args() {
                write!(file, "{arg}\n").expect("Failed writing to last_args.txt");
            }

            Arguments::parse()
        }
    }
}
