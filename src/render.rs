use crate::cmd::SmCommand;
use crate::vapoursynth::output::{output, OutputParameters};
use rustsynth::{
    api::{CoreCreationFlags, API},
    core::CoreRef,
    map::OwnedMap,
    node::Node,
};
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn libav_smashsource(filepath: PathBuf, core: CoreRef, api: API) -> Node {
    let lsmas = core.plugin_by_namespace("lsmas").unwrap();

    let mut in_args = OwnedMap::new(api);
    in_args
        .set_data(
            "source",
            filepath
                .display()
                .to_string()
                .replace("\\\\?\\", "")
                .as_bytes(),
        )
        .expect("Failed setting input source parameter");
    let map = lsmas.invoke("LWLibavSource", &in_args);

    map.get("clip")
        .expect("Failed getting clip from LWLibavSource")
}

pub fn vitamix(commands: Vec<SmCommand>) {
    let api = API::get().unwrap();
    let core = api.create_core(CoreCreationFlags::NONE);

    for cmd in commands {
        let clip = libav_smashsource(cmd.payload.in_path, core, api);

        let mut ffmpeg = Command::new("ffmpeg")
            .args(cmd.ff_args)
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed spawning ffmpeg child");

        let num_frames = clip.video_info().unwrap().num_frames as usize;

        output(
            ffmpeg.stdin.take().expect("Failed taking ffmpeg stdin"),
            None,
            OutputParameters {
                y4m: true,
                node: clip,
                start_frame: 0,
                end_frame: num_frames - 1,
                requests: core.info().num_threads,
            },
        )
        .expect("Failed outputting with output");
    }
}
