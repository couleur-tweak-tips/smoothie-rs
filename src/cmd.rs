use std::env::current_exe;
use which::which;

use crate::cli::Arguments;
use crate::recipe::Recipe;
use crate::video::Payload;

pub struct Command {
    ff: String,
    vs: String,
}

pub fn build_commands(_args: Arguments, payloads: Vec<Payload>, recipe: Recipe) -> Vec<Command> {
    let ff_path = which("ffmpeg").expect("FFmpeg is not installed and/or is not in PATH");

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

    let mut ret: Vec<Command> = Vec::new();

    for payload in payloads {
        for src in payload.videos {
            ret.push(Command {
                ff: format!(
                    "{:?} {} {:?} ",
                    ff_path.display(),
                    recipe
                        .get("miscellaneous")
                        .expect("build_command: failed getting miscellaneous")
                        .get("ffmpeg options")
                        .expect("build_command: failed getting [miscellaneous] ffmpeg options"),
                    src.path.display(),
                ),
                vs: format!("{}", vs_path.display()),
            });
        }
    }

    ret
}
