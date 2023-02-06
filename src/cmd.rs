use std::path::PathBuf;
use which::which;

use crate::cli::Arguments;
use crate::parse::parse_encoding_args;
use crate::recipe::Recipe;
use crate::video::Payload;

// pub struct Command {
//     ff: String,
//     vs: String,
// }

pub fn build_commands(
    args: Arguments,
    payloads: Vec<Payload>,
    mut recipe: Recipe,
) -> Vec<(String, String)> {
    let executable: String = if args.tompv {
        which("mpv")
            .expect("MPV is not installed and/or it's directory is not in PATH")
            .display()
            .to_string()
    } else if args.tonull {
        "".to_owned()
    } else {
        let ff_path = recipe.get("output", "process");
        if ff_path == "ffmpeg" {
            which(recipe.get("output", "process"))
                .expect("FFmpeg is not installed and/or it's directory is not in PATH")
                .display()
                .to_string()
        } else {
            ff_path
        }
    };

    let enc_args = parse_encoding_args(&args, &recipe);

    let mut cmd_arguments = if args.tompv {
        "-".to_string()
    } else {
        recipe.get("miscellaneous", "ffmpeg options")
    };

    for payload in payloads {
        if args.tompv {
        } else {
            if args.tonull {
                cmd_arguments.push_str(&format!(" -i {:?} -f null NUL ", payload.in_path));
            } else if args.peek.is_some() {
            } else {
                if args.stripaudio {
                    cmd_arguments.push_str(&format!(
                        " -i - -map 0:v -map 1:a {enc_args} {:?} ",
                        payload.out_path
                    ));
                }
                cmd_arguments.push_str(&format!(
                    " -i - -i {:?} -map 0:v -map 1:a {enc_args} {:?} ",
                    payload.in_path, payload.out_path
                ));

                if recipe.get_bool("preview window", "enabled") {
                    let mut ffplay_path = recipe.get("preview window", "process");
                    if ffplay_path == "ffplay" {
                        ffplay_path = which(recipe.get("output", "process"))
                            .expect("FFmpeg is not installed and/or it's directory is not in PATH")
                            .display()
                            .to_string()
                    };
                    cmd_arguments.push_str(&format!(
                        " | {ffplay_path:?} {}",
                        recipe.get("miscellaneous", "ffplay options")
                    ));
                }
            }
        }
    }

    // let vs_path = if cfg!(target_os = "windows") {
    //     let included = current_exe()
    //         .expect("Failed getting current exe path")
    //         .parent()
    //         .expect("Failed getting exe path parent")
    //         .join("VapourSynth\\vspipe.exe");
    //
    //     if included.exists() {
    //         included
    //     } else {
    //         println!(
    //             "Embedded VapourSynth environment not found, falling back to VSPipe.exe in path.."
    //         );
    //         which("vspipe").expect("VapourSynth's vspipe is not installed and/or is not in PATH")
    //     }
    // } else {
    //     which("vspipe").expect("VapourSynth's vspipe is not installed and/or is not in PATH")
    // };
    //
    // let mut ret: Vec<Command> = Vec::new();
    //
    // for payload in payloads {
    //     for src in payload.videos {
    //         ret.push(Command {
    //             ff: format!(
    //                 "{:?} {} {:?} ",
    //                 ff_path.display(),
    //                 recipe
    //                     .get("miscellaneous")
    //                     .expect("build_command: failed getting miscellaneous")
    //                     .get("ffmpeg options")
    //                     .expect("build_command: failed getting [miscellaneous] ffmpeg options"),
    //                 src.path.display(),
    //             ),
    //             vs: format!("{}", vs_path.display()),
    //         });
    //     }
    // }
    // for cmd in &ret {
    //     println!("{:?}, {:?}", cmd.ff, cmd.vs);
    // }

    vec![(executable, "".to_string())]
}
