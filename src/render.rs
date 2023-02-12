use std::any::Any;
use std::path::PathBuf;
use crate::cmd::SmCommand;
use crate::vapoursynth::output::{output, OutputParameters};
use rustsynth::{
    map::OwnedMap,
    node::Node,
    api::{API, CoreCreationFlags},
    core::CoreRef
};
use std::process::{Command, Stdio};

fn libav_smashsource (filepath: PathBuf, core: CoreRef, api: API) -> Node {

    let lsmas = core.plugin_by_namespace("lsmas").unwrap();

    let mut in_args = OwnedMap::new(api);
    in_args
        .set_data("source", filepath.display().to_string().replace("\\\\?\\","").as_bytes())
        .expect("Failed setting input source parameter");
    let map = lsmas.invoke("LWLibavSource", &in_args);

    map.get("clip").expect("Failed getting clip from LWLibavSource")
}

pub fn vitamix(commands: Vec<SmCommand>) {
    let api = API::get().unwrap();
    let core = api.create_core(CoreCreationFlags::NONE);


    for cmd in commands {

        let clip = libav_smashsource(cmd.payload.in_path, core, api);

        let mut ffmpeg = Command::new("ffmpeg")
            .args(["-", "-loglevel", "trace"])
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed spawning ffmpeg child");

        output(
            ffmpeg.stdin.take().expect("Failed taking ffmpeg stdin"),
            None,
            OutputParameters {
                y4m: true,
                node: clip,
                start_frame: 0,
                end_frame: 0,
                requests: 0,
            },
        )
        .expect("Failed outputting with output");
    }
}
