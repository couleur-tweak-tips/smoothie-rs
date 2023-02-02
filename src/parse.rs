use crate::cli::Arguments;
use crate::recipe::{parse_recipe, Recipe};
use colored::Colorize;
use std::collections::HashMap;
use std::env::current_exe;

pub fn _parse_bool(bool_str: &str) -> bool {
    let pos = vec!["yes", "ye", "y", "on", "enabled", "1"];
    let neg = vec!["no", "na", "n", "off", "disabled", "0"];

    match bool_str {
        _ if pos.contains(&&*bool_str.to_lowercase()) => true,
        _ if neg.contains(&&*bool_str.to_lowercase()) => false,
        _ => panic!("Unknown boolean (true/false value): {:?}", bool_str),
    }
}

pub fn _parse_encoding_args(args: &Arguments, rc: &Recipe) -> String {
    let input_enc_args = if args.encargs.is_some() {
        args.encargs.clone().expect("Failed unwrapping --encargs")
    } else {
        dbg!(&rc);
        rc.get("encoding")
            .expect("Failed getting [encoding] category from recipe")
            .get("args")
            .expect("Failed getting key `[encoding] args:` from recipe")
            .clone()
    };

    let mut enc_arg_presets: Recipe = HashMap::new();
    parse_recipe(
        current_exe()
            .expect("Failed getting exe path")
            .parent()
            .expect("Failed getting exe parent path")
            .join("encoding_presets.ini"),
        &mut enc_arg_presets,
    );
    dbg!(&enc_arg_presets);

    let mut codec = String::new(); // e.g H264, H265
    let mut ret = String::new();

    let mut codec_options: Vec<String> = Vec::new();
    for key in enc_arg_presets.keys() {
        for alias in key.split('/') {
            codec_options.push(alias.to_uppercase().to_string());
        }
    }

    dbg!(&codec_options);

    for word in input_enc_args.split(' ') {
        if ret.chars().last().is_some()
            && ret
                .chars()
                .last()
                .expect("Parsing error: failed getting last ret char for enc arg preset building")
                != ' '
        {
            ret.push(' ');
        }

        if enc_arg_presets.contains_key("MACROS") {
            let macros = enc_arg_presets
                .get("MACROS")
                .expect("Parsing error: failed getting MACROS in encoding presets");
            if macros.contains_key(word) {
                println!("Pushing {:?} as macro", word);
                ret.push_str(
                    macros
                        .get(word)
                        .expect("Parsing error: failed getting existing macro"),
                );
                continue;
            }
        }

        if !codec.is_empty() {
            let codec_category = enc_arg_presets
                .get(&*codec)
                .expect("Parsing error: failed getting category from enc arg presets");
            if enc_arg_presets.contains_key(&codec.to_uppercase()) {
                println!("Pushing {:?} as an enc preset", word);
                ret.push_str(
                    codec_category
                        .get(word)
                        .expect("Parsing error: failed getting key from enc preset"),
                )
            }
        } else if codec_options.contains(&word.to_string()) {
            for option in enc_arg_presets.keys() {
                if option.contains(word) {
                    println!("Found {:?} for {word}", option);
                    codec = option.clone();
                }
            }
            // codec = word.to_uppercase().to_string();
        } else {
            println!("Pushing {:?} as normal str", word);
            ret.push_str(word);
        }
    }

    ret
}

// use crate::exec::_smoothing;
use serde::{Deserialize};

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Debug, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub name: String,
}

// use reqwest::Error;
// use colored::*;

#[tokio::main]
pub async fn ping_github() -> Result<Release, reqwest::Error> {
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;

    let release: Release = client
        .get("https://api.github.com/repos/couleur-tweak-tips/smoothie/releases/latest")
        .send()
        .await?
        .json()
        .await?;

    Ok(release)
}

pub fn parse_update() {
    let result: Result<Release, reqwest::Error> = ping_github();

    match result {
        Ok(body) => {
            if env!("CARGO_PKG_VERSION") != body.tag_name {
                println!(
                    "{} Current:  {}, Latest: {}",
                    "An update is available!".bright_blue().bold(),
                    env!("CARGO_PKG_VERSION").blue(),
                    body.name.blue()
                );
            }
        }
        Err(e) => {
            println!("Update: Failed checking for a new update, discarding.. {e:?}");
        }
    }
}
