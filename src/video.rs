use crate::{cli::Arguments, recipe::Recipe};
use ffprobe::FfProbe;
use rand::seq::SliceRandom;
// use serde_json::Result;
// select random fruit suffix
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Payload {
    videos: Vec<Source>,
    outpath: String,
}
#[derive(Debug, Clone)]
pub struct Source {
    path: PathBuf,    // D:\obs stuff\video.mp4
    basename: String, // video
    probe: FfProbe,   // provided by ffprobe
    timecodes: Option<Vec<Timecodes>>,
    // bettertimecodes: Option<Vec<(String, String)>>
}

#[derive(Deserialize, Debug, Clone, PartialEq, Hash)]
struct Timecodes {
    filename: String,
    fin: String,
    start: String,
}

fn ensure_dir(dir: &PathBuf, silent: bool) {
    if !dir.is_dir() {
        match fs::create_dir(dir) {
            Ok(_) => {
                if !silent {
                    println!("Creating folder `{:?}`", dir)
                }
            }
            Err(e) => panic!("Failed creating folder at `{:?}`, Error: {e}", dir),
        }
    }
}

fn probe_input(input: Vec<PathBuf>) -> HashMap<PathBuf, FfProbe> {
    let mut ret: HashMap<PathBuf, FfProbe> = HashMap::new();
    for vid in &input {
        let probe = match ffprobe::ffprobe(vid) {
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

pub fn _resolve_outpath(
    args: &mut Arguments,
    rc: &mut Recipe,
    indir: PathBuf,
    basename: String,
) -> PathBuf {
    if args.output.is_some() {}

    let outdir = if let Some(outdir_arg) = args.outdir.clone() {
        println!("Using output directory `{:?}`", outdir_arg);
        ensure_dir(&outdir_arg, false);
        outdir_arg
    } else if rc["misc"]["folder"].trim() != String::new() {
        let outdir_recipe = PathBuf::from(rc["misc"]["folder"].clone());
        ensure_dir(&outdir_recipe, false);
        outdir_recipe
    } else {
        indir
    };

    let container: String = if rc["misc"]["container"].trim() == String::new() {
        println!("Defaulting output extension to .MP4");
        String::from("MP4")
    } else {
        rc["misc"]["container"].replace(".", "").to_string()
    };

    let mut filename = String::from("");

    if !rc["misc"]["prefix"].is_empty() {
        filename.push_str(&rc["misc"]["prefix"].clone());
    }

    filename.push_str(&basename);

    #[rustfmt::skip]
    let fruits: Vec<&str> = [
        " Berry",      " Cherry",   " Cranberry",   " Coconut",   " Kiwi",
        " Avocado",    " Durian",   " Lemon",       " Fig",       " Lime",
        " Mirabelle",  " Banana",   " Pineapple",   " Pitaya",    " Blueberry",
        " Raspberry",  " Apricot",  " Strawberry",  " Melon",     " Papaya",
        " Apple",      " Pear",     " Orange",      " Mango",     " Plum",
        " Peach",      " Grape",    " Tomato",      " Cucumber",  " Eggplant",
        " Guava",      " Honeydew", " Lychee",      " Nut",       " Quince",
        " Olive",      " Passion",  " Plum",        " Pomelo",    " Raisin",
    ]
    .to_vec();

    let suffix: &str = &rc["misc"]["suffix"].clone();
    // lmk if u manage to make it work without converting to a &str
    let suffix: &str = match suffix {
        "fruits" => fruits
            .choose(&mut rand::thread_rng())
            .expect("Failed to select a random suffix"),
        _ => " ~ Smoothie",
    };

    filename.push_str(suffix);
    filename.push_str(&format!(".{container}"));

    return outdir.join(filename);
}

/// Attempts to resolve and structure input structs from CLI arguments
pub fn resolve_input(args: &mut Arguments, _rc: &mut Recipe) -> Vec<Payload> {
    let mut payloads: Vec<Payload> = vec![];

    if args.input.is_empty() && args.json.is_none() && args.tui == true {
        use rfd::FileDialog;

        let _input = FileDialog::new()
            .add_filter(
                "Video file",
                &[
                    "mp4", "mkv", "webm", "mov", "avi", "wmv", "flv", "ts", "m3u8",
                ],
            )
            .set_title("Select video(s) to queue to Smoothie")
            .set_directory("/")
            .pick_files();

        println!("{:?}", _input);

        args.input = match _input {
            Some(paths) => paths,
            None => std::process::exit(0),
        };
    }

    if !args.input.is_empty() {
        let mut input: Vec<PathBuf> = Vec::new();

        for vid in &args.input {
            // Get absolute form of the path (e.g .\video.mp4 -> C:\obs\videos.mp4)
            let path = match vid.canonicalize() {
                Ok(path) => path,
                _ => {
                    println!(
                        "{:?} does not exist or is not a valid filepath, discarding..",
                        vid
                    );
                    continue;
                }
            };

            // Try to open the file
            let file = match fs::File::open(&path) {
                Ok(file) => file,
                _ => {
                    println!("Error opening file: {:?}", path);
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

            // None of the checks hooked up a "continue" statement, safe to add
            input.push(path);
        }

        if input.is_empty() {
            panic!("No input files could be resolved")
        }

        println!("\ninput: {:?}", args.input);
        println!("clean: {:?}", input);

        args.input = input;
        // replace input with clean output

        for vid in args.input.clone() {
            let probe: FfProbe = match ffprobe::ffprobe(&vid) {
                Ok(info) => info,
                Err(err) => {
                    panic!("Could not analyze file with ffprobe: {:?}", err);
                }
            };

            dbg!(&probe);

            payloads.push(Payload {
                videos: vec![Source {
                    path: vid.clone(),
                    basename: vid
                        .file_stem()
                        .expect("Failed getting input filename's base name (stem)")
                        .to_str()
                        .expect("Failed converting input filename stem to &str")
                        .to_string(),
                    probe,
                    timecodes: None,
                }],

                outpath: _resolve_outpath(
                    args,
                    _rc,
                    vid.parent().unwrap().to_path_buf(),
                    vid.file_stem()
                        .expect("Failed getting filename base name (stem) when resolving output")
                        .to_str()
                        .expect("Failed converting")
                        .to_string(),
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
