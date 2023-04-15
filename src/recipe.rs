use crate::cli::Arguments;
use crate::verb;
use std::collections::hash_map::Keys;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct Recipe {
    pub data: HashMap<String, HashMap<String, String>>,
}

impl Recipe {
    pub fn new() -> Recipe {
        Recipe {
            data: HashMap::new(),
        }
    }

    pub fn contains_key(&mut self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn keys(&mut self) -> Keys<'_, String, HashMap<String, String>> {
        self.data.keys()
    }

    pub fn insert_section(&mut self, section: String, data: HashMap<String, String>) {
        self.data.insert(section, data);
    }
    // that lil boilerplate worth the error handling
    pub fn insert_value(&mut self, section: &str, key: String, value: String) {
        if !self.data.contains_key(section) {
            self.data.insert(section.parse().unwrap(), HashMap::new());
        }

        match self.data.get_mut(section) {
            Some(section) => {
                section.insert(key, value);
            }
            None => panic!("Recipe section not found: {section}"),
        }
    }

    pub fn get_bool(&self, section: &str, key: &str) -> bool {
        let pos = vec!["yes", "ye", "y", "on", "enabled", "1"];
        let neg = vec!["no", "na", "n", "off", "disabled", "0"];

        let bool_str = match self.data.get(section) {
            Some(section) => match section.get(key) {
                Some(value) => value,
                None => panic!("Recipe: [{section:?}] {key:?}:"),
            },
            None => panic!("Recipe section not found: `{section}`"),
        };

        match bool_str {
            _ if pos.contains(&bool_str.to_lowercase().as_str()) => true,
            _ if neg.contains(&bool_str.to_lowercase().as_str()) => false,
            _ => panic!("Unknown boolean (true/false value): {bool_str:?}"),
        }
    }

    pub fn get(&self, section: &str, key: &str) -> String {
        match self.data.get(section) {
            Some(section) => match section.get(key) {
                Some(value) => value.to_owned(),
                None => panic!("Recipe: [{section:?}] {key:?}:"),
            },
            None => panic!("Recipe section not found: `{section}`"),
        }
    }

    pub fn get_section(&self, section: &str) -> &HashMap<String, String> {
        match self.data.get(section) {
            Some(section) => section,
            None => panic!("Recipe section not found: `{section}`"),
        }
    }
}

pub fn parse_recipe(ini: PathBuf, rc: &mut Recipe) {
    assert!(ini.exists(), "Recipe at path `{ini:?}` does not exist");
    verb!(
        "Parsing: {}",
        ini.display().to_string().replace("\\\\?\\", "")
    );

    let mut file = match File::open(&ini) {
        Ok(file) => file,
        Err(e) => panic!("Error opening file: {}", e),
    };

    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(e) => panic!("Error getting file metadata: {}", e),
    };
    if metadata.len() == 0 {
        panic!("Error: File is empty: {file:?}");
    }

    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Ok(_) => (),
        Err(e) => panic!("Error reading file: {}", e),
    };

    let mut cur_category = String::new();
    let mut round = 1;

    for mut cur in content.split('\n') {
        cur = cur.trim();
        round += 1;

        match cur {
            // an empty line or comment, just like this one
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
                    rc.insert_section(cur_category.clone(), HashMap::new());
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

                if key.trim() == "∞" {
                    println!("You buffoon.");
                    std::process::exit(0);
                }

                rc.insert_value(
                    &cur_category,
                    key.trim().to_string(),
                    value.trim().to_string(),
                );
            }
            // forgot to put val into a comment!
            _ => panic!("Recipe: Failed to parse {cur:?}, line {round}"),
        }
    }
}

pub fn get_recipe(args: &mut Arguments) -> Recipe {
    let exe = match env::current_exe() {
        Ok(exe) => exe,
        Err(e) => panic!("Could not resolve Smoothie's binary path: {}", e),
    };

    let bin_dir = match exe.parent() {
        Some(bin_dir) => bin_dir.parent().unwrap(),
        None => panic!("Could not resolve Smoothie's binary directory `{exe:?}`"),
    };

    let rc_path = if PathBuf::from(&args.recipe).exists() {
        PathBuf::from(&args.recipe)
    } else {
        let cur_dir_rc = bin_dir.join(&args.recipe);
        if !cur_dir_rc.exists() {
            panic!(
                "Recipe filepath does not exist (expected at {})",
                cur_dir_rc.display()
            )
        }
        cur_dir_rc
    };

    let mut rc: Recipe = Recipe::new();

    parse_recipe(Path::join(bin_dir, "defaults.ini"), &mut rc);
    parse_recipe(rc_path, &mut rc);

    if args.r#override.is_some() {
        // dbg!(&args.r#override);
        for ov in args
            .r#override
            .clone()
            .expect("Failed unwrapping --override")
        {
            // let (category, key, value) = ov.splitn(3, ";").collect();
            // bad code I know, let me know if you can make the line above work ^
            let mut iter = ov.splitn(3, ';');
            let category = iter
                .next()
                .expect("Failed unpacking category of --override");
            let key = iter.next().expect("Failed unpacking key of --override");
            let value = iter.next().expect("Failed unpacking value of --override");

            rc.insert_value(category, key.trim().to_string(), value.trim().to_string());
        }
    }

    if args.verbose || rc.get_bool("miscellaneous", "always verbose") {
        args.verbose = true;
        rc.insert_value(
            "miscellaneous",
            "always verbose".to_owned(),
            "yes".to_owned(),
        );
    }

    rc
}
