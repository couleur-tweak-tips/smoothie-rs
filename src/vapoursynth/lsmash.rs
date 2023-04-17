use std::path::PathBuf;

use rustsynth::{core::CoreRef, map::OwnedMap, node::Node, prelude::API};

pub fn libav_smashsource(filepath: PathBuf, core: CoreRef, api: API) -> Node {
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
