use crate::cli::Arguments;
use crate::verb;
use crate::{NO, YES};
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

    pub fn _entry(&mut self, key: String) -> Entry<String, IndexMap<String, String>> {
        return self.data.entry(key);
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
        let bool_str = match self.data.get(section) {
            Some(section) => match section.get(key) {
                Some(value) => value,
                None => panic!("Recipe: [{section:?}] {key:?}:"),
            },
            None => panic!("Recipe section not found: `{section}`"),
        };

        match bool_str {
            _ if crate::YES.contains(&bool_str.to_lowercase().as_str()) => true,
            _ => false,
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

    pub fn _get_mut(&mut self, section: &str, key: &str) -> String {
        match self.data.get_mut(section) {
            Some(section) => match section.get_mut(key) {
                Some(value) => value.to_string(),
                None => panic!("Recipe get_mut:  failed to get {key:?}"),
            },
            None => panic!("Recipe section not found: `{section}`"),
        }
    }

    pub fn get_section(&self, section: &str) -> &IndexMap<String, String> {
        match self.data.get(section) {
            Some(ret) => &ret,
            None => panic!("Recipe section not found: `{section}`"),
        }
    }

    pub fn get_section_mut(&mut self, section: &str) -> &mut IndexMap<String, String> {
        match self.data.get_mut(section) {
            Some(ret) => ret,
            None => panic!("Recipe section not found: `{section}`"),
        }
    }
}

pub fn parse_recipe(
    ini: PathBuf,
    recipe_str: Option<String>,
    rc: &mut Recipe,
    meta: &mut Option<WidgetMetadata>,
    first_run: bool,
) {
    let content = if let Some(rc_str) = recipe_str {
        rc_str
    } else {
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

        content
    };

    let lines: Vec<String> = content
        .split('\n')
        .map(|s| s.trim().replace("\u{feff}", ""))
        .collect();
    let mut cur_category = String::new();
    // let mut round = 1;

    for i in 0..lines.len() {
        let cur = &lines[i];
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

                let (key, value) = if let Some((key, value)) = setting.split_once(':') {
                    (key.trim(), value.trim())
                } else {
                    panic!("Recipe: Failed to split_once a key")
                };

                // let (key, value) = setting
                //     .split_once(':')
                //     .expect("Recipe: Failed to split_once a key");

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
                let previous_line = &lines[i - 1];

                let is_value_bool = if let Some(mut inner_meta) = meta.take() {
                    if first_run && previous_line.starts_with("#{") {
                        let meta_defs: Vec<String> = previous_line
                            .strip_prefix("#{")
                            .unwrap()
                            .strip_suffix('}')
                            .unwrap()
                            .split(';')
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

                        inner_meta
                            .entry(cur_category.clone())
                            .or_insert_with(IndexMap::new)
                            .entry(key.to_string())
                            .or_insert_with(IndexMap::new)
                            .insert("default".to_string(), value.trim().to_string());

                        inner_meta
                            .entry(cur_category.clone())
                            .or_insert_with(IndexMap::new)
                            .entry("_sm_category".to_string())
                            .or_insert_with(IndexMap::new)
                            .insert("display".to_string(), "no".to_string());

                        inner_meta
                            .entry(cur_category.clone())
                            .or_insert_with(IndexMap::new)
                            .entry(key.to_owned())
                            .or_insert_with(IndexMap::new)
                            .insert("display".to_string(), "no".to_string());
                    } else if !first_run {
                        // inner_meta[cur_category][key][enabled] = true
                        inner_meta
                            .entry(cur_category.clone())
                            .or_insert_with(IndexMap::new)
                            .entry(key.to_owned())
                            .or_insert_with(IndexMap::new)
                            .insert("display".to_string(), "yes".to_string());

                        inner_meta
                            .entry(cur_category.clone())
                            .or_insert_with(IndexMap::new)
                            .entry("_sm_category".to_string())
                            .or_insert_with(IndexMap::new)
                            .insert("display".to_string(), "yes".to_string());
                    } else {
                        panic!("WHAT.")
                    }

                    *meta = Some(inner_meta.clone());

                    inner_meta
                        .get(&cur_category)
                        .unwrap()
                        .get(key)
                        .unwrap()
                        .get("type")
                        .expect(format!("Failed to get 'type' metadata from '[{}] {}:', is there #{{}} metadata for it in defaults.ini?", cur_category, key).as_str())
                        == "bool"
                } else {
                    // metadata is not passed, there is no need to assume
                    false
                };

                if is_value_bool {
                    if YES.contains(&value) {
                        rc.insert_value(&cur_category, key.trim().to_string(), "yes".to_string());
                    } else if NO.contains(&value) {
                        rc.insert_value(&cur_category, key.trim().to_string(), "no".to_string());
                    } else {
                        dbg!(&YES);
                        dbg!(&NO);
                        panic!("{}", format!("Invalid boolean value '{}' for '[{}] {}:', replace with any of the two lists above", value, cur_category, key));
                    }
                }
            }
            // forgot to put val into a comment!
            _ => panic!("Recipe: Failed to parse {:?}, line {}", cur, i + 1),
        }
    }
}

// converts a Recipe object to a serialized String to be copied to copied to the clipboard / wrote to a file
pub fn export_recipe(
    recipe: Recipe,
    meta: &WidgetMetadata,
    omit_removed_categories: bool,
    omit_removed_keys: bool,
    convert_aliases_to_bools: bool,
) -> String {
    // dbg!(&meta);
    let mut buffer = String::new();
    let mut inner_recipe = recipe.clone();

    for cat in inner_recipe.keys() {
        if meta
            .get(cat)
            .unwrap()
            .get("_sm_category")
            .unwrap()
            .get("display")
            .unwrap()
            == "false"
        {
            continue;
        }

        if omit_removed_categories {
            if let Some(enabled) = recipe.data.get(cat).unwrap().get("enabled") {
                if !YES.contains(&enabled.to_lowercase().as_str()) {
                    continue;
                }
            }
        }

        buffer.push_str(("[".to_owned() + cat + "]\n").as_str());

        for (key, value) in recipe.get_section(cat) {
            if omit_removed_keys
                && meta
                    .get(cat)
                    .unwrap()
                    .get(key)
                    .unwrap()
                    .get("display")
                    .unwrap()
                    == "no"
            {
                continue;
            }

            // overwrite as "true" / "false" if needed
            let value = if convert_aliases_to_bools
                && meta
                    .get(cat)
                    .unwrap()
                    .get(key)
                    .unwrap()
                    .get("type")
                    .unwrap()
                    == "bool"
            {
                if YES.contains(&value.as_str()) {
                    "yes".to_owned()
                } else if NO.contains(&value.as_str()) {
                    "no".to_string()
                } else {
                    panic!("Unknown ");
                }
            } else {
                value.to_string()
            };
            buffer.push_str((key.to_owned() + ": " + value.as_str() + "\n").as_str());
        }
        buffer.push('\n');
    }
    buffer
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

    let home_dir: PathBuf;
    let config_path: PathBuf = Default::default();
    let local_path: PathBuf = Default::default();
    let cur_dir_sc: PathBuf = Default::default();

    let mut config_path: PathBuf = Default::default();
    let mut local_path: PathBuf = Default::default();

    #[cfg(not(target_os = "windows"))]
    {
        home_dir = env::home_dir().expect("How do you not have a user dir?");
        config_path = home_dir.join(".config/smoothie-rs");
        if !config_path.exists() {
            panic!("ERROR: expected folder not found @ {}", config_path.display());
        }

        local_path = home_dir.join(".local/share/smoothie-rs");
        if !local_path.exists() {
            panic!("ERROR: expected folder not found @ {}", local_path.display());
        }
    }

    let rc_path = if PathBuf::from(&args.recipe).exists() {
        PathBuf::from(&args.recipe)
    } else {
        let cur_dir_rc = if cfg!(target_os = "windows") {
            bin_dir.join(&args.recipe)
        } else {
            config_path.join(&args.recipe)
        };
        if !cur_dir_rc.exists() {
            panic!(
                "Recipe filepath does not exist (expected at {})",
                cur_dir_rc.display()
            )
        }
        cur_dir_rc
    };
    args.recipe = rc_path.display().to_string();

    let mut rc: Recipe = Recipe::new();
    let mut metadata = Some(WidgetMetadata::new());

    parse_recipe(
        Path::join(bin_dir, "defaults.ini"),
        None,
        &mut rc,
        &mut metadata,
        true,
    );
    parse_recipe(
        rc_path,
        args.recipe_str.clone(),
        &mut rc,
        &mut metadata,
        false,
    );

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
