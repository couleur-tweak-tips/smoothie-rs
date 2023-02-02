use crate::{cli::Arguments, recipe::Recipe};
use ffprobe::FfProbe;
use rand::seq::SliceRandom;
use rfd::FileDialog;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
};

#[derive(Debug, Clone)]
pub struct Payload {
    pub videos: Vec<Source>,
    pub outpath: String,
}
#[derive(Debug, Clone)]
pub struct Source {
    pub path: PathBuf,    // D:\obs stuff\video.mp4
    pub basename: String, // video
    pub probe: FfProbe,   // provided by ffprobe
    pub timecodes: Option<Vec<Timecodes>>,
    // bettertimecodes: Option<Vec<(String, String)>>
}

#[derive(Deserialize, Debug, Clone, PartialEq, Hash)]
pub struct Timecodes {
    pub filename: String,
    pub fin: String,
    pub start: String,
}

pub struct _QueueObject {
    pub video: Option<PathBuf>, // D:\obs stuff\video.mp4
    pub videos: Option<Vec<PathBuf>>,
    pub basename: String, // video
    pub probe: FfProbe,   // provided by ffprobe
    pub timecodes: Option<Vec<HashMap<PathBuf, _NewTimeCodes>>>,
    // bettertimecodes: Option<Vec<(String, String)>>
}

pub struct _NewTimeCodes {
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
            Err(e) => panic!("Failed creating folder at `{dir:?}`, Error: {e}"),
        }
    }
}

/// Returns and filters the valid videos amongst args.input / args.json
fn probe_input(input: Vec<PathBuf>) -> HashMap<PathBuf, FfProbe> {
    let mut ret: HashMap<PathBuf, FfProbe> = HashMap::new();
    for vid in &input {
        let path = match vid.canonicalize() {
            Ok(path) => path,
            _ => {
                println!("{vid:?} does not exist or is not a valid filepath, discarding..");
                continue;
            }
        };

        // Try to open the file
        let file = match fs::File::open(&path) {
            Ok(file) => file,
            _ => {
                println!("Error opening file: {path:?}");
                continue;
            }
        };

        // Check if the file is empty (0 bytes)
        let metadata = file.metadata().expect("Error getting file metadata");
        if metadata.len() == 0 {
            println!(
                "{:?} is an empty file (0 bytes), discarding..",
                path.file_name().expect("Failing getting input filename")
            );
            continue;
        }

        let probe = match ffprobe::ffprobe(path) {
            Ok(a) => a,
            Err(e) => {
                println!("Skipping input file `{:?}` (failed probing): {:?}", &vid, e);
                continue;
            }
        };
        ret.insert(vid.to_path_buf(), probe);
    }
    ret
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
        "Berry",      "Cherry",   "Cranberry",   "Coconut",   "Kiwi",
        "Avocado",    "Durian",   "Lemon",       "Fig",       "Lime",
        "Mirabelle",  "Banana",   "Pineapple",   "Pitaya",    "Blueberry",
        "Raspberry",  "Apricot",  "Strawberry",  "Melon",     "Papaya",
        "Apple",      "Pear",     "Orange",      "Mango",     "Plum",
        "Peach",      "Grape",    "Tomato",      "Cucumber",  "Eggplant",
        "Guava",      "Honeydew", "Lychee",      "Nut",       "Quince",
        "Olive",      "Passion",  "Plum",        "Pomelo",    "Raisin",
    ].to_vec();

    let mut format = if dont_format {
        "%FILENAME%-SM".to_string()
    } else {
        recipe
            .get("output")
            .expect("Failed getting [output] from recipe")
            .get("file format")
            .expect("Failed getting `[output] file format:` from recipe")
            .to_uppercase()
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
                    .choose(&mut rand::thread_rng())
                    .expect("Failed to select a random suffix")
            ),
        );
    }
    if format.contains("%FILENAME") {
        format = format.replace("%FILENAME%", &basename);
    } else {
        panic!("No `%FILENAME%` variable in recipe's `[misc] format:` key");
    }

    let rc_container = recipe
        .get("output")
        .expect("Failed getting [output] from recipe")
        .get("container")
        .expect("Failed getting `[output] container:` from recipe")
        .trim();

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
    // println!("wtf");

    if args.input.is_empty() && args.json.is_none() && args.tui {
        let input = FileDialog::new()
            .add_filter(
                "Video file",
                &[
                    "mp4", "mkv", "webm", "mov", "avi", "wmv", "flv", "ts", "m3u8",
                ],
            )
            .set_title("Select video(s) to queue to Smoothie")
            .set_directory("/")
            .pick_files();

        println!("Added:");
        dbg!(&input);

        args.input = match input {
            Some(paths) => paths,
            None => std::process::exit(0),
        };
    }

    if !args.input.is_empty() {
        let probe_map: HashMap<PathBuf, FfProbe> = probe_input(args.input.clone());
        args.input = Vec::from_iter(probe_map.keys().cloned());
        // replace input with clean output

        for vid in args.input.clone() {
            // println!("{:?}", probe);
            // dbg!(&probe);

            payloads.push(Payload {
                videos: vec![Source {
                    path: vid.clone(),
                    basename: vid
                        .file_stem()
                        .expect("Failed getting input filename's base name (stem)")
                        .to_str()
                        .expect("Failed converting input filename stem to &str")
                        .to_string(),
                    probe: probe_map
                        .get(&*vid)
                        .expect("Failed getting probe map key")
                        .clone(),
                    timecodes: None,
                }],

                outpath: resolve_outpath(
                    args,
                    recipe,
                    vid.parent().unwrap().to_path_buf(),
                    vid.file_stem()
                        .expect("Failed getting filename base name (stem) when resolving output")
                        .to_str()
                        .expect("Failed converting")
                        .to_string(),
                    false,
                )
                .display()
                .to_string(),
            })
        }
        // dbg!(&payloads);
    }
    // if args.input is not
    else if args.json.is_some() {
        let _cuts: Vec<Timecodes> = match serde_json::from_str(&args.json.clone().unwrap()) {
            Ok(cut) => cut,
            Err(e) => panic!("Failed parsing JSON: {e}"),
        };

        let _cuts: Vec<Timecodes> = serde_json::from_str(&args.json.clone().unwrap()).unwrap();

        let mut aggregated_cuts: HashMap<String, Vec<(String, String)>> = HashMap::new();

        for cut in _cuts.clone() {
            if !aggregated_cuts.contains_key(&cut.filename) {
                aggregated_cuts.insert(cut.filename.clone(), vec![(cut.start, cut.fin)]);
            } else {
                aggregated_cuts
                    .get_mut(&cut.filename)
                    .unwrap()
                    .append(&mut vec![(cut.start, cut.fin)]);
            }
        }

        dbg!(&aggregated_cuts);
        let mut _sources: Vec<Source> = vec![];
        // for cut in aggregated_cuts.keys() {
        //     sources.push(
        //         Source {
        //             path: cut.into(),
        //             basename: cut.
        //             probe: (),
        //             timecodes: () }
        //     )
        //     // payloads.push(
        //     //     Payload {

        //     //     }
        //     // );
        // }

        // let mut _payload = Payload {
        //     outpath: String::from("C:\\yay~SM.mp4"),
        //     videos: vec![Source {
        //         path: PathBuf::from("C:\\yay.mp4"),
        //         basename: String::from("yay"),
        //         probe: ffprobe::ffprobe("C:\\yay.mp4").unwrap(),
        //         timecodes: None,
        //     }],
        // };
        // // payload.outpath = String::from("hi");
        // println!("{:?}", _cuts);
        // for cut in &_cuts {
        //     println!("filename: {:?}", cut.filename);
        //     println!("start: {:?}", cut.start);
        //     println!("fin: {:?}", cut.fin);
        // }
    };

    payloads

    // }else {
    //     println!("ARGS: {:?}", args);
    //     println!("JSON: {:?}, INPUT: {:?}", !args.json.is_none(), !args.input.is_empty());
    //     panic!("Could not resolve input method (nor JSON or INPUT were provided)")
    // }
}
