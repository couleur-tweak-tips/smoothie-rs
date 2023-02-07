use crate::cmd::SmCommand;
use std::process::{Command, ExitStatus, Stdio};

fn process(cmd: SmCommand) -> ExitStatus {
    // dbg!(&cmd.vs_path);
    // dbg!(&cmd.vs_args);
    // dbg!(&cmd.ff_path);
    // dbg!(&cmd.ff_args);
    // dbg!(&cmd.ff_path);

    let vs = Command::new(cmd.vs_path)
        .args(cmd.vs_args)
        .stdout(Stdio::piped())
        // .stderr(Stdio::piped())
        .spawn()
        .expect("Could not start VapourSynth");

    let ff = Command::new(cmd.ff_path)
        .args(cmd.ff_args)
        .stdin(Stdio::from(vs.stdout.expect("Could not pipe to FFmpeg")))
        .spawn()
        .expect("Could not start FFmpeg");

    ff.wait_with_output()
        .expect("FFmpeg stopped unexpectedly")
        .status
}

pub fn _smoothing(commands: Vec<SmCommand>) {
    for command in commands {
        process(command);
    }
}
