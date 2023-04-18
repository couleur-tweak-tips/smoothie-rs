use std::path::Path;

use rustsynth::{core::CoreRef, map::OwnedMap, node::Node};

pub fn libav_smashsource<'a>(filepath: &Path, core: CoreRef<'a>) -> Node<'a> {
    let lsmas = core.plugin_by_namespace("lsmas").unwrap();

    let mut in_args = OwnedMap::new();
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
