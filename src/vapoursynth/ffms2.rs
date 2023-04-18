use rustsynth::{node::Node, core::CoreRef, OwnedMap};

#[derive(OwnedMap, Clone)]
pub struct SourceArgs {
    pub source: String,
}

pub fn source<'core>(core: &CoreRef<'core>, args: SourceArgs) -> Result<Node<'core>, String> {
    let in_args = args.to_map();
    let ffms2 = core.plugin_by_namespace("ffms2").unwrap();
    let map = ffms2.invoke("Source", &in_args);

    match map.error()  {
        None => Ok(map.get("clip").expect("Failed getting clip from FFMS.2Source")),
        Some(error) => Err(error.into_owned())
    }
}