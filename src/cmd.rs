use std::env::current_exe;
use which::which;

use crate::cli::Arguments;
use crate::parse::parse_encoding_args;
use crate::recipe::Recipe;
use crate::video::Payload;

#[derive(Debug)]
pub struct SmCommand {
    pub payload: Payload,
    pub ff_path: String,
    pub ff_args: Vec<String>,
    pub vs_path: String,
    pub vs_args: Vec<String>,
}

pub fn build_commands(
    args: Arguments,
    payloads: Vec<Payload>,
    recipe: Recipe,
) -> Vec<SmCommand> {
    let executable: String = if args.tompv {
        which("mpv")
            .expect("MPV is not installed and/or it's directory is not in PATH")
            .display()
            .to_string()
    } else {
        let ff_path = recipe.get("output", "process");
        if ff_path == "ffmpeg" {
            which(ff_path)
                .expect("FFmpeg is not installed and/or it's directory is not in PATH")
                .display()
                .to_string()
        } else {
            ff_path
        }
    };

    let vs_path = if cfg!(target_os = "windows") {
        let included = current_exe()
            .expect("Failed getting current exe path")
            .parent()
            .expect("Failed getting exe path parent")
            .join("VapourSynth\\vspipe.exe");

        if included.exists() {
            included
        } else {
            println!(
                "Embedded VapourSynth environment not found, falling back to VSPipe.exe in path.."
            );
            which("vspipe").expect("VapourSynth's vspipe is not installed and/or is not in PATH")
        }
    } else {
        which("vspipe").expect("VapourSynth's vspipe is not installed and/or is not in PATH")
    };

    let mut cmd_arguments: Vec<String> = vec![];
    if args.tompv {
        cmd_arguments.push("-".to_string());
    }else{
        cmd_arguments.append(
            &mut recipe.get("miscellaneous", "ffmpeg options").split(" ").map(String::from).collect()
        );
    }

    let vitamix = current_exe()
        .expect("Failed getting current exe path")
        .parent()
        .expect("Failed getting exe path parent")
        .parent()
        .expect("Failed getting exe path parent's path parent")
        .join("sm-py\\vitamix.vpy");

    let rc_string =
        serde_json::to_string(&recipe.data).expect("Failed serializing recipe (to pass to VSPipe)");

    let mut vs_args: Vec<String> = vec![
        vitamix.display().to_string(),
        "--container".to_owned(),
        "y4m".to_owned(),
        "-".to_owned(),
        "--arg".to_owned(),
        format!("rc={rc_string}"),
    ];

    if args.peek.is_some() {
        vs_args.append(&mut vec![
            "--start".to_owned(),
            format!("{}", args.peek.unwrap()),
            "--end".to_owned(),
            format!("{}", args.peek.unwrap()),
        ]);
    }

    let mut enc_args: Vec<String> = parse_encoding_args(&args, &recipe).split(" ").map(String::from).collect();

    // let mut ret: Vec<(Payload, PathBuf, String)> = vec![];
    let mut ret: Vec<SmCommand> = vec![];

    for payload in payloads {
        if payload.in_path == payload.out_path {
            panic!("Output path has same path as input")
        }

        let mut cur_cmd_arguments = cmd_arguments.clone();
        let mut cur_vs_args = vs_args.clone();

        cur_vs_args.append(&mut vec![
            "--arg".to_owned(), format!("input_video={}", payload.in_path.display().to_string())
        ]);

        if args.tompv {
        } else {
            if args.tonull {
                cur_cmd_arguments.append(
                    &mut vec![
                        "-i".to_owned(), payload.in_path.display().to_string(), "-f".to_owned(), "null".to_owned(), "NUL".to_owned()
                    ]
                );
                // cur_cmd_arguments.push(format!(" -i {:?} -f null NUL ", payload.in_path));
            } else if args.peek.is_some() {} else {
                if args.stripaudio {
                    cur_cmd_arguments.append(&mut enc_args);
                } else {
                    cur_cmd_arguments.append(&mut vec![
                        "-i".to_owned(), format!("{}", payload.in_path.display().to_string()),
                        "-map".to_owned(), "0:v".to_owned(), "-map".to_owned(), "1:a".to_owned()
                    ]);
                }
                cur_cmd_arguments.push(payload.out_path.display().to_string());

                /* fuck that.
                 if recipe.get_bool("preview window", "enabled") {
                     let mut ffplay_path = recipe.get("preview window", "process");
                     if ffplay_path == "ffplay" {
                         ffplay_path = which(ffplay_path)
                             .expect("FFplay (previewer) is not installed and/or it's directory is not in PATH")
                             .display()
                             .to_string()
                     };
                     cur_cmd_arguments.push(format!(
                         " | {ffplay_path:?} {}",
                         recipe.get("miscellaneous", "ffplay options")
                     ));
                */
            }

        }
        ret.push(SmCommand {
            payload,
            ff_path: executable.clone(),
            ff_args: cur_cmd_arguments.iter().map(|n| n.to_string()).collect(),
            vs_path: vs_path.clone().display().to_string(),
            vs_args: cur_vs_args.iter().map(|n| n.to_string()).collect(),
        });
        // ret.push((payload, PathBuf::from(executable.clone()), cmd_arguments.clone()));
        // println!("{cmd_arguments}");
    }

    ret
}
