use std::{env, path::PathBuf, fs};
use homedir;

const DEFAULT_RECIPE: &str = include_str!("../target/recipe.ini");
const DEFAULT_ENCODING_PRESETS: &str = include_str!("../target/encoding_presets.ini");

fn get_target_path() -> PathBuf {
    let current_exe = env::current_exe().expect("Could not determine exe");
    let target_dir = current_exe.parent()
        .expect("Could not get directory of executable")
        .parent()
        .expect("Could not get directory of directory's executable??");
    return target_dir.to_path_buf();
}

fn is_portable() -> bool {
    let portable = get_target_path().join("linux-portable-enable");
    return portable.exists();
}

pub fn get_config_path() -> PathBuf {
    let config_path: PathBuf;
    
    if cfg!(target_os = "windows") || is_portable() {
        config_path = get_target_path();
    } else {
        let home_dir = homedir::my_home()
            .unwrap()
            .expect("How do you not have a user dir?");
        config_path = home_dir.join(".config/smoothie-rs");
        if !config_path.exists() {
            fs::create_dir_all(&config_path)
                .expect("Failed to create config folder");
        }
    } 

    return config_path;
}

pub fn get_recipe_path_custom(recipe_name : &str) -> PathBuf {
    let recipe_path = get_config_path().join(recipe_name);
    return recipe_path;
}

pub fn get_recipe_path() -> PathBuf {
    let recipe_path = get_recipe_path_custom("recipe.ini");
    if !recipe_path.exists() {
        fs::write(&recipe_path, DEFAULT_RECIPE).unwrap();
    }
    return recipe_path;
}

pub fn get_encoding_presets_path() -> PathBuf {
    let encoding_presets_path = get_config_path().join("encoding_presets.ini");
    if !encoding_presets_path.exists() {
        fs::write(&encoding_presets_path, DEFAULT_ENCODING_PRESETS).unwrap();
    }
    return encoding_presets_path;
}

pub fn get_defaults_path() -> PathBuf {
    return get_target_path().join("defaults.ini");
}

pub fn get_last_args_path() -> PathBuf {
    let last_args: PathBuf;

    if cfg!(target_os = "windows") || is_portable() {
        last_args = get_target_path()
            .join("last_args.txt");
    } else {
        let home_dir = homedir::my_home()
            .unwrap()
            .expect("How do you not have a user dir?");
        last_args = home_dir.join(".local/share/smoothie-rs/last_args.txt");
        if !last_args.exists() {
            fs::create_dir_all(
                last_args
                .parent()
                .unwrap())
                .expect("Failed to create local folder"
                );
        }
    }

    return last_args;
}
