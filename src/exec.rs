use crate::cmd::SmCommand;
use std::process::{Command, ExitStatus, Stdio};

fn process(cmd: SmCommand) -> ExitStatus {
    dbg!(&cmd);

    let vs = Command::new(cmd.vs_path)
        .args(cmd.vs_args)
        .stdout(Stdio::piped())
        // .stderr(Stdio::piped())
        .spawn()
        .expect("Could not start VapourSynth");

    let ff = Command::new(cmd.ff_path)
        .args(cmd.ff_args)
        .stdin(Stdio::from(vs.stdout.expect("Could not pipe to FFmpeg")))
        .stdout(if cmd.ffplay_path.is_some() {
            Stdio::piped()
        } else {
            Stdio::inherit()
        })
        .spawn()
        .expect("Could not start FFmpeg");

    if cmd.ffplay_path.is_some() {
        let ffplay = Command::new(cmd.ffplay_path.unwrap())
            .args(cmd.ffplay_args.unwrap())
            .stdin(Stdio::from(ff.stdout.expect("Could not pipe to FFplay")))
            .spawn()
            .expect("Could not start FFplay (previewer)");

        ffplay
            .wait_with_output()
            .expect("FFplay stopped unexpectedly")
            .status
    } else {
        ff.wait_with_output()
            .expect("FFmpeg stopped unexpectedly")
            .status
    }
}

pub fn _smoothing(commands: Vec<SmCommand>) {
    for command in commands {
        process(command);
    }
}
