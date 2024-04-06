use crate::cli::Arguments;
use crate::verb;
use indexmap::map::Entry;
use indexmap::map::IndexMap;
use indexmap::map::Keys;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Default)]
pub struct Recipe {
    #[serde(with = "indexmap::map::serde_seq")]
    pub data: IndexMap<String, IndexMap<String, String>>,
}

pub type WidgetMetadata = IndexMap<String, IndexMap<String, IndexMap<String, String>>>;

impl Recipe {
    pub fn new() -> Recipe {
        Recipe {
            data: IndexMap::new(),
        }
    }

    pub fn contains_key(&mut self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn entry(&mut self, key: String) -> Entry<String, IndexMap<String, String>>{
        return self.data.entry(key)
    }

    pub fn keys(&mut self) -> Keys<'_, String, IndexMap<String, String>> {
        self.data.keys()
    }

    pub fn insert_section(&mut self, section: String, data: IndexMap<String, String>) {
        self.data.insert(section, data);
    }
    // that lil boilerplate worth the error handling
    pub fn insert_value(&mut self, section: &str, key: String, value: String) {
        if !self.data.contains_key(section) {
            self.data.insert(section.parse().unwrap(), IndexMap::new());
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
    
    pub fn get_mut(&mut self, section: &str, key: &str) -> &mut String {
        match self.data.get_mut(section) {
            Some(section) => match section.get_mut(key) {
                Some(value) => value,
                None => panic!("Recipe get_mut:  failed to get {key:?}"),
            },
            None => panic!("Recipe section not found: `{section}`"),
        }
    }

    pub fn get_section(&self, section: &str) -> &IndexMap<String, String> {
        match self.data.get(section) {
            Some(section) => section,
            None => panic!("Recipe section not found: `{section}`"),
        }
    }
}

pub fn parse_recipe(
    ini: PathBuf,
    rc: &mut Recipe,
    meta: &mut Option<WidgetMetadata>,
    first_run: bool,
) {
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

    let lines: Vec<&str> = content.split('\n').map(|s| s.trim()).collect();
    let mut cur_category = String::new();
    // let mut round = 1;

    for i in 0..lines.len() {
        let cur = lines[i];
        // round += 1;

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
                    rc.insert_section(cur_category.clone(), IndexMap::new());
                }
            }

            // weighting: gaussian
            setting if cur.contains(':') => {
                // rc
                if cur_category.is_empty() {
                    panic!(
                        "Recipe: Setting {:?} has no parent category, line {}",
                        setting,
                        i + 1
                    );
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

                // meta
                let previous_line = lines[i - 1];

                if let Some(mut inner_meta) = meta.take() {

                    if first_run && previous_line.starts_with("#{") {
                        let meta_defs: Vec<String> = previous_line
                            .strip_prefix("#{")
                            .unwrap()
                            .strip_suffix("}")
                            .unwrap()
                            .split(";")
                            .map(|s| s.trim().to_string())
                            .collect();

                        for meta_definition in meta_defs {

                            let (meta_key, meta_value) = meta_definition
                                .split_once(':')
                                .expect("Recipe: Failed to split_once a key");

                            

                            inner_meta
                                .entry(cur_category.clone())
                                .or_insert_with(IndexMap::new)
                                .entry(key.to_string())
                                .or_insert_with(IndexMap::new)
                                .insert(meta_key.trim().to_string(), meta_value.trim().to_string());
                        }
                    } else if !first_run {
                        // inner_meta[cur_category][key][enabled] = true

                        inner_meta
                            .entry(cur_category.clone())
                            .or_insert_with(IndexMap::new)
                            .entry(key.to_string())
                            .or_insert_with(IndexMap::new)
                            .insert("display".to_string(), "true".to_string());
                    }

                    *meta = Some(inner_meta);
                }
            }
            // forgot to put val into a comment!
            _ => panic!("Recipe: Failed to parse {:?}, line {}", cur, i + 1),
        }
    }
}

pub fn get_recipe(args: &mut Arguments) -> (Recipe, WidgetMetadata) {
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
    let mut metadata = Some(WidgetMetadata::new());

    parse_recipe(
        Path::join(bin_dir, "defaults.ini"),
        &mut rc,
        &mut metadata,
        true,
    );
    parse_recipe(rc_path, &mut rc, &mut metadata, false);

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

    (rc, metadata.unwrap())
}
