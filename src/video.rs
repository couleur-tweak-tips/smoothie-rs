
use crate::cli::Arguments;

use std::{
    collections::HashMap,
    path::PathBuf,
    fs
};

enum _RecipeValues {
    SmBool,
    Int, int32,

}

pub struct QueueObject {
    videos:     Vec<VideoObject>,
    outpath:    String,

}

pub struct VideoObject {
    path:       PathBuf, // D:\obs stuff\video.mp4
    basename:   String, // video.mp4
    probe:      HashMap<String, String>, // provided by ffprobe
    timecodes:  Vec<Trims>,
}



#[derive(Deserialize, Debug)]
struct Trims {
    trims: Vec<Timecodes>
}


#[derive(Deserialize, Debug)]
struct Timecodes {
    filename: String,
    fin: i32,
    start: i32,
}

/// Attempts to resolve and structure input structs from CLI arguments
pub fn resolve_input (_rc: HashMap<String, String>, mut args: Arguments){

    args.input.take();
    /*
    if args.json.is_empty() && !args.input.is_empty() {

        let mut input: Vec<PathBuf> = Vec::new();
        for vid in &args.input {

            // Get absolute form of the path (e.g .\video.mp4 -> C:\obs\videos.mp4)
            let path = match vid.canonicalize() {
                Ok(path) => path,
                _ => {
                    println!("{:?} does not exist or is not a valid filepath, discarding..", vid);
                    continue;
                },
            };
            // Try to open the file
            let file = match fs::File::open(&path) {
                Ok(file) => file,
                _ => {
                    println!("Error opening file: {:?}", path);
                    continue;
                },
            };
            // Check if the file is empty (0 bytes)
            let metadata = file.metadata().expect("Error getting file metadata");
            if metadata.len() == 0 {
                println!("{:?} is an empty file (0 bytes), discarding..", path.file_name().unwrap());
                continue;
            }

            // None of the checks hooked up a "continue" statement, safe to add 
            input.push(path);

        }
        if input.len() == 0 {
            panic!("No input files could be resolved")
        }
        println!("\ninput: {:?}", args.input);
        println!("clean: {:?}", input);
        args.input = input;
        


    } else if !args.json.is_empty() && args.input.is_empty() {


        for _videopath in &args.input {

            if args.input.len() == 1 && args.trim || args.split {
    
                if !args.json.is_empty() {
                    panic!("Provided split/trim switch but no JSON payload")
                }
    
                let _person: Trims = serde_json::from_str(&args.json).unwrap();
            }
        }
    
    } else {
        println!("ARGS: {:?}", args);
        println!("JSON: {:?}, INPUT: {:?}", args.json.is_empty(), args.input.is_empty());
        panic!("Could not resolve input method (nor JSON or INPUT were provided)")
    }
    */
}