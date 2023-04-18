use rustsynth::{
    core::CoreRef, format::SampleType, function::Function, node::Node, owned_map, prelude::Map,
};

pub fn _change_fps<'elem, 'core: 'elem>(
    core: CoreRef<'core>,
    clip: Node<'core>,
    fpsnum: f64,
    fpsden: f64,
) -> Node<'elem> {
    let factor = (fpsnum / fpsden)
        * (clip.video_info().unwrap().fps_den as f64 / clip.video_info().unwrap().fps_num as f64);

    let length = (clip.video_info().unwrap().num_frames as f64 * factor).floor();
    let adjust_frame = move |core: CoreRef<'core>, in_map: &Map<'core>, out: &mut Map<'core>| {
        let n = in_map.get_int("n").unwrap();
        let real_n = (n as f64 / factor).floor();
        let std = core.plugin_by_namespace("std").unwrap();
        let in_args = owned_map!({ "clip": &clip }, { "first": &real_n }, { "last": &real_n });
        let trim = std.invoke("Trim", &in_args);
        let one_frame_clip: Node = trim.get("clip").unwrap();
        let in_args = owned_map!({"clip": &one_frame_clip}, {"factor": &(clip.video_info().unwrap().num_frames as i64 + 100)});
        let one_frame_clip = std.invoke("Loop", &in_args);
        let node = one_frame_clip.get_node("clip").unwrap();
        out.set_node("clip", &node).unwrap();
    };
    let adjust_frame = Function::new(core, adjust_frame);
    let std = core.plugin_by_namespace("std").unwrap();
    let in_args = owned_map!({ "length": &length }, { "fpsnum": &fpsnum }, {
        "fpsden": &fpsden
    });
    let blank = std.invoke("BlankClip", &in_args);
    let in_args = owned_map!({"eval": &adjust_frame}, {"clip": &blank.get_node("clip").unwrap()});
    let eval = std.invoke("FrameEval", &in_args);
    eval.get_node("clip").unwrap()
}

pub fn inter_frame<'elem, 'core: 'elem>(
    core: CoreRef<'core>,
    clip: Node<'core>,
    params: InterFrameParams,
) -> Result<Node<'elem>, &'static str> {
    let info = clip.video_info().unwrap();

    let sw = info.format.sub_sampling_w;
    let sh = info.format.sub_sampling_h;
    let depth = info.format.bits_per_sample;
    if sw != 1 && sh != 1 && !vec![8, 10].contains(&depth) {
        return Err("InterFrame: This is not a clip");
    }
    let oInput = clip;
    // let clip = vsdepth(clip, 8);
    // Validate inputs
    let preset = params.Preset.to_lowercase();
    let tuning = params.Tuning.to_lowercase();
    if !["medium", "fast", "faster", "fastest"].contains(&preset.as_str()) {
        return Err("");
    }
    if !["film", "smooth", "animation", "weak"].contains(&tuning.as_str()) {
        return Err("");
    }
    todo!()
}

pub struct InterFrameParams {
    Preset: String,
    Tuning: String,
    NewNum: Option<i64>,
    NewDen: i64,
    GPU: bool,
    gpuid: i64,
    OverrideAlgo: Option<String>,
    OverrideArea: Option<String>,
    FrameDouble: bool,
}

impl Default for InterFrameParams {
    fn default() -> Self {
        Self {
            Preset: String::from("Medium"),
            Tuning: String::from("Film"),
            NewNum: None,
            NewDen: 1,
            GPU: true,
            gpuid: 1,
            OverrideAlgo: None,
            OverrideArea: None,
            FrameDouble: false,
        }
    }
}

pub fn vsdepth(clip: Node, bitdepth: i64) -> Node {
    let info = clip.video_info().unwrap();
    let curr_depth = info.format.bits_per_sample as i64;
    let sample_type = SampleType::Integer;
    if (curr_depth, info.format.sample_type) == (bitdepth, sample_type) {
        return clip;
    }
    todo!()
}
