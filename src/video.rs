use crate::cli::Arguments;
use serde_json::Result;
use std::{collections::HashMap, fs, path::PathBuf};

pub struct QueueObject {
    videos: Vec<VideoObject>,
    outpath: String,
}

pub struct VideoObject {
    path: PathBuf,                  // D:\obs stuff\video.mp4
    basename: String,               // video.mp4
    probe: HashMap<String, String>, // provided by ffprobe
    timecodes: Vec<Timecodes>,
}

#[derive(Deserialize, Debug)]
struct Timecodes {
    filename: String,
    fin: String,
    start: String,
}

/// Attempts to resolve and structure input structs from CLI arguments
pub fn resolve_input(mut args: Arguments) {
    if !args.input.is_empty() {
        let mut input: Vec<PathBuf> = Vec::new();
        // create a temporary vector (array)

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
                    path.file_name().unwrap()
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
    }
    // if args.input is not
    else if args.json.is_some() {
        let _cuts: Vec<Timecodes> = match serde_json::from_str(&args.json.clone().unwrap()) {
            Ok(cut) => cut,
            Err(e) => panic!("Failed parsing JSON: {e}"),
        };

        let _cuts: Vec<Timecodes> = serde_json::from_str(&args.json.clone().unwrap()).unwrap();
        println!("{:?}", _cuts);
        for cut in &_cuts {
            println!("filename: {:?}", cut.filename);
            println!("start: {:?}", cut.start);
            println!("fin: {:?}", cut.fin);
        }
    }
    // }else {
    //     println!("ARGS: {:?}", args);
    //     println!("JSON: {:?}, INPUT: {:?}", !args.json.is_none(), !args.input.is_empty());
    //     panic!("Could not resolve input method (nor JSON or INPUT were provided)")
    // }
}
