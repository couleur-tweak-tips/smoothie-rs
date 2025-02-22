use crate::cli::Arguments;
use crate::portable;
use crate::recipe::{parse_recipe, Recipe};
use crate::verb;
use color_eyre::owo_colors::OwoColorize;
use colored::Colorize;
use serde::Deserialize;
use std::env;
use std::time::Duration;
use ureq::{Agent, Error as uReqError};

pub fn parse_encoding_args(args: &Arguments, rc: &Recipe) -> String {
    let input_enc_args = if args.encargs.is_some() {
        return args.encargs.clone().expect("Failed unwrapping --encargs");
    } else {
        rc.get("output", "enc args")
    };

    let mut enc_arg_presets: Recipe = Recipe::new();
    parse_recipe(
        portable::get_encoding_presets_path(),
        None,
        &mut enc_arg_presets,
        &mut None,
        false,
    );

    let mut codec = String::new(); // e.g H264, H265
    let mut ret = String::new();

    let mut codec_options: Vec<String> = Vec::new();
    for key in enc_arg_presets.keys() {
        for alias in key.split('/') {
            codec_options.push(alias.to_uppercase().to_string());
        }
    }

    // dbg!(&codec_options);

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
            let macros = enc_arg_presets.get_section("MACROS");
            if macros.contains_key(word) {
                verb!("Pushing {word:?} as macro");
                ret.push_str(
                    macros
                        .get(word)
                        .expect("Parsing error: failed getting existing macro"),
                );
                continue;
            }
        }

        if !codec.is_empty() {
            let codec_category = enc_arg_presets.get_section(&codec);
            if enc_arg_presets.clone().contains_key(&codec.to_uppercase()) {
                verb!("Pushing {word:?} as an enc preset");
                ret.push_str(
                    codec_category
                        .get(&*str::to_uppercase(word))
                        .expect("Parsing error: failed getting key from enc preset"),
                )
            }
        } else if codec_options.contains(&word.to_string()) {
            for option in enc_arg_presets.keys() {
                if option.contains(word) {
                    verb!("Found {option:?} for {word}");
                    codec = option.clone();
                }
            }
            // codec = word.to_lowercase().to_string();
        } else {
            verb!("Pushing {word:?} as normal str");
            ret.push_str(word);
        }
    }

    ret
}

// static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);


#[derive(Debug, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub name: String,
}

#[allow(dead_code)]
pub fn ping_github() -> Result<Release, uReqError> {
    let agent: Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();
    let release: Release = agent
        .get("https://api.github.com/repos/couleur-tweak-tips/smoothie/releases/latest")
        .call()?
        .into_json()?;

    Ok(release)
}

#[allow(dead_code)]
pub fn parse_update() {
    match ping_github() {
        Ok(body) => {
            if env!("CARGO_PKG_VERSION") != body.tag_name {
                println!(
                    "{} Current:  {}, Latest: {}",
                    "An update is available!".bright_blue().bold(),
                    env!("CARGO_PKG_VERSION").bright_blue(),
                    body.name.bright_blue()
                );
            }
        }
        Err(e) => {
            println!(
                "{}",
                format!("Update: Failed checking for a new update, discarding.. {e}")
                    .bright_black()
            );
        }
    }
}

