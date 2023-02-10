use rustsynth::{api, node::Node, owned_map, prelude::*};

use crate::{
    cmd::SmCommand,
    vapoursynth::output::{output, OutputParameters},
};
use std::process::{Command, ExitStatus, Stdio};

fn process(cmd: SmCommand, out_params: OutputParameters) -> ExitStatus {
    dbg!(&cmd);

    let mut ff = Command::new(cmd.ff_path)
        .args(cmd.ff_args)
        .stdin(Stdio::piped())
        .stdout(if cmd.ffplay_path.is_some() {
            Stdio::piped()
        } else {
            Stdio::inherit()
        })
        .spawn()
        .expect("Could not start FFmpeg");

    let vapoursynth = output(ff.stdin.take().unwrap(), None, out_params);

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
    let api = API::get().unwrap();
    let core = api.create_core(api::CoreCreationFlags::NONE);
    let lsmash = core.plugin_by_id("lsmas").unwrap();
    for command in commands {
        let args = owned_map!(api, {"source": &command.payload.in_path.to_str().unwrap().to_string()}, {"cache": &1}, {"prefer_hw": &3});
        let clip: Node = lsmash.invoke("LWLibavSource", &args).get("clip").unwrap();
        let end_frame = (clip.video_info().unwrap().num_frames - 1) as usize;
        process(
            command,
            OutputParameters {
                node: clip,
                start_frame: 0,
                end_frame,
                requests: core.info().num_threads,
                y4m: true,
            },
        );
    }
}
