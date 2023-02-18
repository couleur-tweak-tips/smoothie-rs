use crate::cmd::SmCommand;

use std::process::{Command, Stdio};

pub fn vspipe_render(commands: Vec<SmCommand>) {
    for cmd in commands {
        println!("{}", cmd.ff_args.join(" "));

        let mut vs = Command::new(cmd.vs_path)
            .args(cmd.vs_args)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed spawning ffmpeg child");

        let pipe = vs.stdout.take().expect("Failed piping out of VSPipe");

        let ffmpeg = Command::new(cmd.ff_path)
            .args(cmd.ff_args)
            .stdin(pipe)
            .spawn()
            .expect("Failed spawning ffmpeg child");

        vs.wait_with_output().unwrap();
        ffmpeg.wait_with_output().unwrap();
    }
}
