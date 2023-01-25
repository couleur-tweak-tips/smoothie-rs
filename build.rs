// build.rs

extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("./src/smoothie.ico");
        res.compile().expect("Failed compiling exe icon");
    }
}
