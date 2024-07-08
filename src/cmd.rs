use std::env::current_exe;
use which::which;

use crate::cli::Arguments;
use crate::parse::parse_encoding_args;
use crate::recipe::Recipe;
use crate::video::Payload;

use crate::verb;
use std::env;

#[derive(Debug)]
pub struct SmCommand {
    pub vs_path: String,
    pub vs_args: Vec<String>,
    pub payload: Payload,
    pub ff_path: String,
    pub recipe: Recipe,
    pub ff_args: Vec<String>,
    pub ffplay_path: Option<String>,
    pub ffplay_args: Option<Vec<String>>,
}

pub fn build_commands(args: Arguments, payloads: Vec<Payload>, recipe: Recipe) -> Vec<SmCommand> {
    let executable: String = if args.tompv {
        which("mpv")
            .expect("mpv has not been installed or has not been added to PATH")
            .display()
            .to_string()
    } else {
        let ff_path = recipe.get("output", "process");
        if ff_path == "ffmpeg" {
            which(ff_path)
                .expect("FFmpeg has not been installed or has not been added to PATH")
                .display()
                .to_string()
        } else {
            let is_ffmpeg: bool = ff_path.ends_with("ffmpeg") || ff_path.ends_with("ffmpeg.exe");
            let r#override: bool = env::var("SM_ALLOW_MISC_OUTPUT") == Ok("1".to_owned());

            if !is_ffmpeg && !r#override {
                panic!("You specified an output process which does not have the filename 'ffmpeg', to override this error message please set the environment variable SM_ALLOW_MISC_OUTPUT to 1");
            } else {
                ff_path
            }
        }
    };

    let mut cmd_arguments: Vec<String> = vec![];
    if args.tompv {
        cmd_arguments.push("-".to_string());
    } else {
        cmd_arguments.append(
            &mut recipe
                .get("miscellaneous", "ffmpeg options")
                .split(" ")
                .map(String::from)
                .collect(),
        );
    }

    let enc_args: Vec<String> = parse_encoding_args(&args, &recipe)
        .split(" ")
        .map(String::from)
        .filter(|s| !s.is_empty())
        .collect();

    let cur_exe = current_exe().unwrap();
    let cur_exe_dir = cur_exe.parent().unwrap();
    let vs_bin = if cfg!(target_os = "windows") {
        "vspipe.exe"
    } else {
        "vspipe"
    };
    let bin_dir_vspipe = cur_exe_dir.join(vs_bin);
    let vspipe_in_path = which("vspipe");
    let vs_path = (
        if args.vspipe_path.is_some(){
        args.vspipe_path.unwrap()}
    else if bin_dir_vspipe.exists() {
        verb!("Using vspipe that's in same directory as binary");
        bin_dir_vspipe
    } else if vspipe_in_path.is_ok() {
        verb!("Using VSPipe from PATH");
        vspipe_in_path.unwrap()
    } else {
        panic!("vspipe binary in path/bin dir not found");
    })
    .display()
    .to_string();

    let vpy_path = if args.vpy.exists() {
        args.vpy
    } else if cur_exe_dir.parent().unwrap().join(&args.vpy).exists() {
        cur_exe_dir.parent().unwrap().join(&args.vpy)
    } else {
        panic!(
            "jamba.vpy not found, expected {:?}",
            cur_exe_dir.parent().unwrap().join(&args.vpy)
        );
    };

    /*
        scuffed, but works

        https://github.com/indexmap-rs/indexmap/issues/325

        old one : let rc_string = serde_json::to_string(&recipe).expect("Failed serializing recipe to JSON");
    */
    let rc_string = (format!("{:?}", &recipe)).replace("Recipe { data: {", "{ \"data\": {");


    let vs_args = vec![
        // "--progress".to_owned(),
        "--container".to_owned(),
        "y4m".to_owned(),
        "-".to_owned(),
        vpy_path.display().to_string(),
        "--arg".to_owned(),
        format!("recipe={rc_string:?}"),
    ];

    let mut ret: Vec<SmCommand> = vec![];

    for payload in payloads {
        let mut cur_vs_args = vs_args.clone();

        cur_vs_args.append(&mut vec![
            "--arg".to_owned(),
            format!("input_video={}", payload.in_path.display()),
        ]);
        if let Some(timecodes) = payload.timecodes.clone() {
            let json_timecodes =
                serde_json::to_string(&timecodes).expect("Failed serializing timecodes to JSON");

            cur_vs_args.append(&mut vec![
                "--arg".to_owned(),
                format!("timecodes={json_timecodes:?}"),
            ]);
        }

        if payload.in_path == payload.out_path {
            panic!("Output path has same path as input")
        }

        let mut cur_cmd_arguments = cmd_arguments.clone();

        if args.tompv {
            // nothing to do, but this still needs to step in to break out the if chain
            if let Some(p) = args.peek {
                // duplicate sowwy :33
                cur_vs_args.append(&mut vec![
                    "--start".to_owned(),
                    p.to_string(),
                    "--end".to_owned(),
                    p.to_string()
                ]);
            }
        } else if args.tonull {
            cur_cmd_arguments.append(&mut vec![
                "-i".to_owned(),
                payload.in_path.display().to_string(),
                "-f".to_owned(),
                "null".to_owned(),
                "NUL".to_owned(),
            ]);
            // cur_cmd_arguments.push(format!(" -i {:?} -f null NUL ", payload.in_path));
        } else {
            if let Some(p) = args.peek {
                cur_vs_args.append(&mut vec![
                    "--start".to_owned(),
                    p.to_string(),
                    "--end".to_owned(),
                    p.to_string()
                ]);
            } else if args.stripaudio {
                cur_cmd_arguments.append(&mut enc_args.clone());
            } else {
                cur_cmd_arguments.append(&mut vec![
                    "-i".to_owned(),
                    format!("{}", payload.in_path.display().to_string()),
                    "-map".to_owned(),
                    "0:v".to_owned(),
                    "-map".to_owned(),
                    "1:a?".to_owned(),
                ]);
            }
            cur_cmd_arguments.append(&mut enc_args.clone());
            cur_cmd_arguments.push(payload.out_path.display().to_string());

            if recipe.get_bool("preview window", "enabled") {
                let mut ffmpeg_preview_output: Vec<String> = recipe
                    .get("preview window", "output args")
                    .split(" ")
                    .map(String::from)
                    .collect();

                cur_cmd_arguments.append(&mut ffmpeg_preview_output);
            }
        }

        let (ffplay_path, ffplay_args) =
            if recipe.get_bool("preview window", "enabled") && !args.tompv && args.peek.is_none() {
                let mut ffplay_path = recipe.get("preview window", "process");
                if ffplay_path == "ffplay" {
                    ffplay_path = which(ffplay_path)
                    .expect(
                        "FFplay (previewer) has not been installed or has not been added to PATH",
                    )
                    .display()
                    .to_string()
                };
                let ffplay_args: Vec<String> = recipe
                    .get("miscellaneous", "ffplay options")
                    .split(" ")
                    .map(String::from)
                    .collect();

                (Some(ffplay_path), Some(ffplay_args))
            } else {
                (None, None)
            };
        // dbg!(&cur_cmd_arguments);
        ret.push(SmCommand {
            payload,
            ff_path: executable.clone(),
            ff_args: cur_cmd_arguments,
            recipe: recipe.clone(),
            ffplay_path,
            ffplay_args,
            vs_path: vs_path.clone(),
            vs_args: cur_vs_args.clone(),
        });
    }

    ret
}
