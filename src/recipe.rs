use crate::cli::Arguments;
use std::collections::HashMap;

use std::path::Path;
use std::path::PathBuf;
use std::{env, fs};

use std::io::Read;

pub type Recipe = HashMap<String, HashMap<String, String>>;

pub fn parse_recipe(ini: PathBuf, rc: &mut Recipe) {
    assert!(ini.exists(), "Recipe at path `{ini:?}` does not exist");
    println!("recipe: {ini:?}");

    let mut file = match fs::File::open(&ini) {
        Ok(file) => file,
        Err(e) => panic!("Error opening file: {e}"),
    };

    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(e) => panic!("Error getting file metadata: {e}"),
    };
    if metadata.len() == 0 {
        panic!("Error: File is empty: {file:?}");
    }

    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Ok(_) => (),
        Err(e) => panic!("Error reading file: {e}"),
    };

    let mut cur_category = String::new();
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
                cur_category = category
                    .trim_matches(|c| c == '[' || c == ']')
                    // remove all [ and ] characters
                    .trim()
                    // remove any spaces that would be at the start and end
                    .to_string();

                if !rc.contains_key(&cur_category) {
                    rc.insert(cur_category.clone(), HashMap::new());
                }
            }

            // weighting: gaussian
            setting if cur.contains(':') => {
                if cur_category.is_empty() {
                    panic!("Recipe: Setting {setting:?} has no parent category, line {round}");
                }

                let (key, value) = setting
                    .split_once(':')
                    .expect("Recipe: Failed to split_once a key");

                rc.get_mut(&cur_category)
                    .expect("Failed to get section `{cur_category}`")
                    .insert(key.trim().to_string(), value.trim().to_string());
            }
            // forgot to put val into a comment!
            _ => panic!("Recipe: Failed to parse {cur:?}, line {round}"),
        }
    }
}

pub fn get_recipe(args: &Arguments) -> Recipe {
    let exe = match env::current_exe() {
        Ok(exe) => exe,
        Err(e) => panic!("Could not resolve Smoothie's binary path: {e}"),
    };

    let bin_dir = match exe.parent() {
        Some(bin_dir) => bin_dir,
        None => panic!("Could not resolve Smoothie's binary directory `{exe:?}`"),
    };

    let rc_path = bin_dir.join(&args.recipe);

    let mut rc: Recipe = HashMap::new();

    parse_recipe(Path::join(bin_dir, "defaults.ini"), &mut rc);
    parse_recipe(rc_path, &mut rc);

    if args.r#override.is_some() {
        dbg!(&args.r#override);
        for ov in args
            .r#override
            .clone()
            .expect("Failed unwrapping --override")
        {
            // let (category, key, value) = ov.splitn(3, ";").collect();
            // bad code i know, let me know if you can make line above work ^
            let mut iter = ov.splitn(3, ';');
            let category = iter
                .next()
                .expect("Failed unpacking category of --override");
            let key = iter.next().expect("Failed unpacking key of --override");
            let value = iter.next().expect("Failed unpacking value of --override");

            rc.get_mut(category)
                .expect("Failed to get category from --override")
                .insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    // dbg!(&rc);
    // println!("rc: {:?}", rc);

    rc
}
