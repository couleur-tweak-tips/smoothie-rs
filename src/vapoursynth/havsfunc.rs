use rustsynth::{
    core::CoreRef,
    function::Function,
    node::Node,
    owned_map,
    prelude::{Map, API},
};

pub fn change_fps<'elem, 'core: 'elem>(
    api: API,
    core: CoreRef<'core>,
    clip: Node<'core>,
    fpsnum: f64,
    fpsden: f64,
) -> Node<'elem> {
    let factor = (fpsnum / fpsden)
        * (clip.video_info().unwrap().fps_den as f64 / clip.video_info().unwrap().fps_num as f64);
    let adjust_frame = Function::new(core, adjust_frame);
    let std = core.plugin_by_namespace("std").unwrap();
    let in_args = owned_map!(api, {"length": &(clip.video_info().unwrap().num_frames as f64 * factor).floor()}, {"fpsnum": &fpsnum}, {"fpsden": &fpsden});
    let blank = std.invoke("BlankClip", &in_args);
    let in_args =
        owned_map!(api, {"eval": &adjust_frame}, {"clip": &blank.get_node("clip").unwrap()});
    let eval = std.invoke("FrameEval", &in_args);
    eval.get_node("clip").unwrap()
}

fn adjust_frame<'core>(api: API, core: CoreRef<'core>, in_map: &Map<'core>, out: &mut Map<'core>) {
    let n = in_map.get_int("n").unwrap();
    let clip = in_map.get_node("clip").unwrap();
    let factor = in_map.get_float("factor").unwrap();
    let real_n = (n as f64 / factor).floor();
    let std = core.plugin_by_namespace("std").unwrap();
    let in_args = owned_map!(api, { "clip": &clip }, { "first": &real_n }, {
        "last": &real_n
    });
    let trim = std.invoke("Trim", &in_args);
    let one_frame_clip: Node = trim.get("clip").unwrap();
    let in_args = owned_map!(api, {"clip": &one_frame_clip}, {"factor": &(clip.video_info().unwrap().num_frames as i64 + 100)});
    let one_frame_clip = std.invoke("Loop", &in_args);
    let node = one_frame_clip.get_node("clip").unwrap();
    out.set_node("clip", &node).unwrap();
}
