use crate::{cli::Arguments, recipe::Recipe};
use color_eyre::owo_colors::OwoColorize;
use ffprobe::FfProbe;
use rand::seq::IndexedRandom;
use rfd::FileDialog;
use std::{collections::HashMap, fs, path::PathBuf};
use which::which;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Payload {
    pub in_path: PathBuf,  // D:\obs stuff\video.mp4
    pub out_path: PathBuf, // D:\obs stuff\video ~ Mango.mp4
    pub basename: String,  // Equivalent to .NET's [IO.Path]::GetFileNameWithoutExtension
    pub probe: FfProbe,    // provided by ffprobe
    pub timecodes: Option<Vec<Timecodes>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Timecodes {
    pub start: String,
    pub fin: String,
}

/// Creates a directory to the given folder path (mainly used for args.outdir)
fn ensure_dir(dir: &PathBuf, silent: bool) {
    if !dir.is_dir() {
        match fs::create_dir(dir) {
            Ok(_) => {
                if !silent {
                    println!("Creating folder `{dir:?}`")
                }
            }
            Err(e) => panic!("Failed creating folder at `{dir:?}`, Error: {}", e),
        }
    }
}

/// Only returns videos that are valid (exists, ffprobe-able)
fn probe_video(input: &PathBuf) -> Option<FfProbe> {
    let path = match input.canonicalize() {
        Ok(path) => path,
        _ => {
            println!(
                "{}",
                format!("{input:?} does not exist or is not a valid filepath, discarding..")
                    .on_red()
            );
            return None;
        }
    };

    // Try to open the file
    let file = match fs::File::open(&path) {
        Ok(file) => file,
        _ => {
            println!("Error opening input file: {path:?}");
            return None;
        }
    };

    // Check if the file is empty (0 bytes)
    let metadata = file.metadata().expect("Error getting input file metadata");
    if metadata.len() == 0 {
        println!(
            "{:?} is an empty file (0 bytes), discarding..",
            path.file_name().expect("Failing getting input filename")
        );
        return None;
    }

    let probe = match ffprobe::ffprobe(&path) {
        Ok(a) => a,
        Err(e) => {
            dbg!(
                "Skipping input file `{:?}` (failed probing): {:?}",
                &input,
                e
            );
            return None;
        }
    };
    Some(probe)
}

/// Generates an output file path
pub fn resolve_outpath(
    args: &mut Arguments,
    recipe: &Recipe,
    in_dir: PathBuf,
    basename: String,
    dont_format: bool,
) -> PathBuf {
    if args.output.is_some() {
        return PathBuf::from(args.output.as_ref().expect("Failed unwrapping --output"));
    }

    #[rustfmt::skip]
    let fruits: Vec<&str> = [
        "Apple",      "Apricot",     "Avocado",     "Banana",     "Blackberry",
        "Blueberry",  "Cantaloupe",  "Cherry",      "Coconut",    "Cranberry",
        "Cucumber",   "Durian",      "Date",        "Eggplant",   "Fig",
        "Grape",      "Guava",       "Honeydew",    "Kiwi",       "Lemon",
        "Lime",       "Lychee",      "Mango",       "Mirabelle",  "Olive",
        "Orange",     "Papaya",      "Passion",     "Peach",      "Pear",
        "Pineapple",  "Pitaya",      "Plum",        "Pomelo",     "Quince",
        "Raspberry",  "Starfruit",   "Strawberry",  "Tomato",     "Watermelon",
    ].to_vec();

    let mut format = if dont_format {
        "%FILENAME%-SM".to_string()
    } else {
        recipe.get("output", "file format").to_uppercase()
        // .get("output")
        // .expect("Failed getting [output] from recipe")
        // .get("file format")
        // .expect("Failed getting `[output] file format:` from recipe")
        // .to_uppercase()
    };

    let out_dir = if args.outdir.is_some() {
        ensure_dir(
            args.outdir
                .as_ref()
                .expect("--outdir: Failed unwrapping value in --outdir"),
            false,
        );
        args.outdir
            .clone()
            .expect("Failed unwrapping string passed in --outdir")
    } else {
        in_dir
    };

    if format.contains("%FRUITS%") || format.contains("%FRUIT") {
        format = format.replace("%FRUIT%", "%FRUITS%").replace(
            "%FRUITS%",
            &format!(
                " {}",
                fruits
                    .choose(&mut rand::rng())
                    .expect("Failed to select a random suffix")
            ),
        );
    }
    // create list of vars with section, key, and placeholder name
    let variables = vec![
        ("interpolation", "fps", "INTERP_FPS"),
        ("interpolation", "speed", "SPEED"),
        ("interpolation", "tuning", "TUNING"),
        ("interpolation", "algorithm", "ALGORITHM"),
        ("frame blending", "fps", "OUTPUT_FPS"),
        ("frame blending", "intensity", "BLUR_AMOUNT"),
        ("frame blending", "weighting", "WEIGHTING"),
        ("flowblur", "intensity", "FLOWBLUR_AMOUNT"),
        ("miscellaneous", "dedup threshold", "DEDUP"),
        ("pre-interp", "factor", "FACTOR"),
    ];
    // loop through each var
    for (section, key, var) in variables {
        // check if file format string contains this var's placeholder
        if format.contains(&format!("%{}%", var)) {
            // get var value from recipe using section and key
            let mut value = recipe.get(section, key);
            // truncate weigting var if too long
            if key == "weighting" && value.len() > 15 {
                value = format!("{}..", &value[..15]);
            }
            // replace filename forbidden characters with underscores
            value = value
                .chars()
                .map(|c| match c {
                    '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
                    _ => c,
                })
                .collect();
            // skip this var if value is empty
            if value.trim().is_empty() {
                continue;
            }
            // replace placeholder with an actual value
            format = format.replace(&format!("%{}%", var), &value);
        }
    }
    if format.contains("%FILENAME") {
        format = format.replace("%FILENAME%", &basename);
    } else {
        panic!("No `%FILENAME%` variable in recipe's `[misc] format:` key");
    }

    let rc_container = recipe.get("output", "container").trim().to_owned();
    // .expect("Failed getting [output] from recipe")
    // .get("container")
    // .expect("Failed getting `[output] container:` from recipe")
    // .trim();

    let container: String = if rc_container.is_empty() {
        println!("Defaulting output extension to .MP4");
        String::from("MP4")
    } else {
        rc_container.replace('.', "")
    };

    let mut out = out_dir.join(format!("{}.{}", &format, &container));
    let mut round = 2;
    while out.exists() {
        out = out_dir
            .clone()
            .join(format!("{} ({round}).{}", &format, &container));
        round += 1;
    }

    out
}

/// Attempts to resolve and structure input structs from CLI arguments
pub fn resolve_input(args: &mut Arguments, recipe: &Recipe) -> Vec<Payload> {
    let mut payloads: Vec<Payload> = vec![];
    let mut videos: Vec<(PathBuf, FfProbe, Option<Vec<Timecodes>>)> = vec![];

    if which("ffmpeg").is_err() {
        let term = if cfg!(target_os = "windows") {
            "ffmpeg.exe"
        } else {
            "ffmpeg"
        };
        panic!("{term} is not installed/in PATH, ensure FFmpeg is installed.");
    }
    if which("ffprobe").is_err() {
        let term = if cfg!(target_os = "windows") {
            "ffprobe.exe"
        } else {
            "ffprobe"
        };
        panic!("{term} is not installed/in PATH, ensure FFmpeg is installed.");
    }
    if recipe.get_bool("preview window", "enabled") && which("ffplay").is_err() {
        let term = if cfg!(target_os = "windows") {
            "ffplay.exe"
        } else {
            "ffplay"
        };
        panic!("You have the preview window enabled, but {term} is not installed/in PATH, ensure FULL FFmpeg is installed (ffmpeg, ffprobe and ffplay.");
    }

    // Option 1: launched a shortcut that had --tui in args
    if args.tui && args.input.is_empty() && args.json.is_none() {
        let input = FileDialog::new()
            .add_filter("Video file", crate::VIDEO_EXTENSIONS)
            .set_title("Select video(s) to queue to Smoothie")
            .set_directory("/")
            .pick_files();

        dbg!(&input);

        args.input = match input {
            Some(paths) => paths,
            None => std::process::exit(0),
        };
    }

    // Option 2: picked files in option 1 / used a shortcut Send to / the CLI
    if !args.input.is_empty() {
        // input is a vector of paths
        for vid in &mut args.input {
            let probe = match probe_video(vid) {
                Some(probe) => probe,
                None => continue, // filtered out
            };

            videos.push((
                vid.canonicalize()
                    .expect("Failed getting full input file path"),
                probe,
                None,
            ));
        }

    // Option 3: suckless-cut / Smoothie Pre-Render
    } else if args.json.is_some() {
        let cuts: HashMap<PathBuf, Vec<Timecodes>> =
            match serde_json::from_str(&args.json.clone().unwrap()) {
                Ok(cut) => cut,
                Err(e) => panic!("Failed parsing JSON: {}", e),
            };

        for vid in Vec::from_iter(cuts.keys()) {
            let probe = match probe_video(vid) {
                Some(probe) => probe,
                None => continue,
            };
            let timecodes: Vec<Timecodes> = cuts.get(vid).expect("Failed").to_owned();

            videos.push((vid.clone(), probe, Some(timecodes)));
        }
    }

    for (vid, probe, timecodes) in videos {
        payloads.push(Payload {
            in_path: vid.clone(),
            out_path: resolve_outpath(
                args,
                recipe,
                vid.parent().unwrap().to_path_buf(),
                vid.file_stem()
                    .expect("Failed getting filename base name (stem) when resolving output")
                    .to_str()
                    .expect("Failed converting")
                    .to_string(),
                false,
            ),
            basename: vid
                .file_stem()
                .expect("Failed getting input filename's base name (stem)")
                .to_str()
                .expect("Failed converting input filename stem to &str")
                .to_string(),
            probe,
            timecodes,
        })
    }

    if payloads.is_empty() {
        panic!("No valid videos were passed to Smoothie")
    }

    payloads
}
