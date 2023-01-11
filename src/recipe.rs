use crate::cli::Arguments;

use std::{fs, env};
// use std::path::PathBuf;
use std::collections::HashMap;

use std::io::Read;

pub enum _Val {
    I32(i32),
    Str(String)
}

pub struct sm_bool {
    boolstr: bool,
}

impl sm_bool {
    fn parse_sm_bool(&self) -> bool {
        
    }
}

struct interpolation {
    enabled: bool,
    fps: i32,
    tuning: str,
    algorithm: str,
    speed: str,
    use_gpu: bool,
}

struct frame_blending {
    enabled: bool,
    fps: i32,
    intensity: f64,
    weighting: str,
}

struct flowblur {
    enabled: bool,
    amount: i32,
    mask: PathBuf
}

struct encoding {
    process: PathBuf,
    loglevel: str,
    args: str
}

struct preview {
    enabled: bool,
    ffmpeg: str,
    process: PathBuf,
    args: str
}

struct misc {
    mpv_bin: PathBuf,
    stay_on_top: bool,
    verbose: bool,
    ding_after: i32,
    folder: str,
    container: str,
    prefix: str,
    suffix: str,
    dedupthreshold: f64,
}

struct time_scale {
    input: f64,
    output: f64,
}

struct pre_interp {
    enabled: bool,
    factor: str,
    model: PathBuf,
}

pub struct smoothie_recipe {
    interpolation: interpolation,
    frame_blending: frame_blending,
    flowblur: flowblur,
    encoding: encoding,
    preview: preview,
    misc: misc,
    time_scale: time_scale,
    pre_interp: pre_interp,
}

pub struct smoothie_recipe2 {
    interp_enabled: bool,
    interp_fps: i32,
    interp_tuning: str,
    interp_algorithm: str,
    interp_speed: str,
    interp_use_gpu: bool,
}

// fn parse_bool (category: str, key: str, boolString: str) -> bool {
//     match boolString {
//         positive if ["on", "yes", "y", "true", "1"].contains(boolString) => true,
//         negative if ["off", "no", "n", "false", "0"].contains(boolString) => false,
//         _ => panic!("Unknown bool (positive/negative) for [{}] {}: with value {:?}", category, key, boolString),
//     }
// }

// Used each time sinces a multiplier needs to know a 
// pub fn parse_fps (category: &str, key: &str, fps: &str, vidfps: i32) -> String  {
//     if fps.ends_with('x') || fps.starts_with('x') {
//         return "Hi".to_string()
//     }else{
//         return "bye".to_string();
//     }
//     let num = match fps.parse::<i32>() {
//         Ok(n) => n,
//         Err(e) => {
//             panic!("Error parsing string: {}", e);
//         },
//     }
// }

/// Returns the formatted key 
// fn format_rcval<'a>(category: &'a str, key: &'a str, value: String) -> String {

//     match category {


//         "interpolation" => {

//             match key {
//                 "fps" => key,

//                 "enabled" => value = parse_bool(category, key, value.trim()),
                
//                 "tuning" => {
//                     if !["weak", "smooth", "film"].contains(key){
//                         panic!("Unknown interpolation tuning key: {:?}", key)
//                     }else {

//                     }
//                 }
//                 "algorithm" => {
//                     if !["1", "13", "23"].contains(key){
//                         panic!("Unknown interpolation algorithm key: {:?}", key)
//                     }else{

//                     }
//                 }
//                 "speed" => {
//                     if !["medium", "fast", "faster", "fastest"].contains(key){
//                         panic!("Unknown interpolation tuning key: {:?}", key)
//                     }
//                 }
//                 _ => panic!("Unknown interpolation key: {:?}", category)
//             }
//         }
        

//         "frame blending" => {
//             match key {
//                 "fps" => {
//                     // parse_fps(category, key, fps, vidfps)
//                     return key.to_string();
//                 },
//                 _ => panic!("")
//             }
//         }


//         // "misc"

//         _ => panic!("Unknown category: {:?}", category),

//     }

// }

fn parse_recipe(content: String) {

    let mut rc: HashMap<String, String> = HashMap::new();

    let mut cur_section = String::new();
    let lines: Vec<&str> = content.split("\n").collect();


    for line in lines {


        let cur = line.trim().to_string();
        
        match cur {


            // e.g [frame interpolation]
            category if cur.starts_with('[') && cur.ends_with(']') => {

                cur_section = category
                                .trim_matches(|c| c == '[' || c == ']')
                                    // remove all [ and ] characters
                                .trim()
                                    // remove any spaces that would be at the start and end
                                .to_string();


                // rc.insert(cur_section, HashMap<String, String>::new());
            },

            // e.g weighting: gaussian
            setting if cur.contains(":") => {

                assert_ne!(cur_section, String::new(), "Setting {:?} has no parent category", setting);

                let parts: Vec<&str> = setting.splitn(2, ':').collect();
                    // split it into an array of key and value*
                
                assert_eq!(parts.len(), 2, "Recipe: key/value has more than two elements: {:?}", setting);
                    // ensure it has been properly parsed

                let (key, value) = (parts[0].trim(), parts[1].trim());
                    // trim and assign

                rc.cur_section.insert(key, value);
            },

            _comment if cur.starts_with("#") => {},
            _ => panic!("Recipe: Don't know what to do with {}", cur)
        }
        println!("{:?}", rc);
    }
}

pub fn get_recipe(args: Arguments){

    let exe = match env::current_exe() {
        Ok(exe) => exe,
        Err(e) => panic!("Could not resolve Smoothie's binary path: {}", e),
    };

    let bindir = match exe.parent() {
        Some(bindir) => bindir,
        None => panic!("Could not resolve Smoothie's binary directory"),
    };

    let rc_path = bindir.join(args.recipe);

    assert!(rc_path.exists(),"Recipe at path `{:?}` does not exist", rc_path);


    println!("recipe: {:?}", rc_path);

    // Open the file at the given path
    let mut file = match fs::File::open(&rc_path) {
        Ok(file) => file,
        Err(e) => panic!("Error opening file: {}", e),
    };

    // Check if the file is empty
    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(e) => panic!("Error getting file metadata: {}", e),
    };
    if metadata.len() == 0 {
        panic!("Error: File is empty");
    }

    // Read the contents of the file into a string
    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Ok(_) => (),
        Err(e) => panic!("Error reading file: {}", e),
    };

    // return parse_recipe(content)
}