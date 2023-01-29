use crate::cli::Arguments;
#[allow(unused_imports)]
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

use std::path::Path;
#[allow(unused_imports)]
use std::path::PathBuf;
use std::{env, fs};

use std::io::Read;

pub type Recipe = HashMap<String, HashMap<String, String>>;

fn parse_recipe(ini: PathBuf, rc: &mut Recipe) {
    assert!(ini.exists(), "Recipe at path `{:?}` does not exist", ini);
    println!("recipe: {:?}", ini);

    let mut file = match fs::File::open(&ini) {
        Ok(file) => file,
        Err(e) => panic!("Error opening file: {}", e),
    };

    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(e) => panic!("Error getting file metadata: {}", e),
    };
    if metadata.len() == 0 {
        panic!("Error: File is empty: {:?}", file);
    }

    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Ok(_) => (),
        Err(e) => panic!("Error reading file: {}", e),
    };

    let mut cur_section = String::new();
    let mut round = 1;

    for mut cur in content.split('\n') {
        cur = cur.trim();
        round += 1;

        match cur {
            // a empty line or comment, just like this one
            _comment
                if cur.starts_with('#')
                    || cur.starts_with('/')
                    || cur.starts_with(';')
                    || cur.starts_with(':')
                    || cur.is_empty() => {}

            // [frame interpolation]
            category if cur.starts_with('[') && cur.ends_with(']') => {
                cur_section = category
                    .trim_matches(|c| c == '[' || c == ']')
                    // remove all [ and ] characters
                    .trim()
                    // remove any spaces that would be at the start and end
                    .to_string();

                if !rc.contains_key(&cur_section) {
                    rc.insert(cur_section.clone(), HashMap::new());
                }
            }

            // weighting: gaussian
            setting if cur.contains(':') => {
                if cur_section.is_empty() {
                    panic!(
                        "Recipe: Setting {:?} has no parent category, line {round}",
                        setting
                    );
                }

                let (key, value) = setting
                    .split_once(':')
                    .expect("Recipe: Failed to split {setting}, line {round}");

                rc.get_mut(&cur_section)
                    .expect("Failed to get section `{cur_section}`")
                    .insert(key.trim().to_string(), value.trim().to_string());
            }
            // forgot to put val into a comment!
            _ => panic!("Recipe: Failed to parse {:?}, line {round}", cur),
        }
    }
}

pub fn get_recipe(args: &Arguments) -> Recipe {
    let exe = match env::current_exe() {
        Ok(exe) => exe,
        Err(e) => panic!("Could not resolve Smoothie's binary path: {}", e),
    };

    let bindir = match exe.parent() {
        Some(bindir) => bindir,
        None => panic!("Could not resolve Smoothie's binary directory `{:?}`", exe),
    };

    let rc_path = bindir.join(&args.recipe);

    let mut rc: Recipe = HashMap::new();

    parse_recipe(Path::join(bindir, "defaults.ini"), &mut rc);
    parse_recipe(rc_path, &mut rc);
    println!("rc: {:?}", rc);

    rc
}
