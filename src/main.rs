// trait Formatting {
//     fn rant_about_c(&self);
// }
//  
// impl Formatting for &str {
//     fn rant_about_c (&self){
//         println!("{self} rant_about_c);
//     }
// }
//  
// fn main(){
//     "aetopia".rant_about_c();
//     Formatting::rant_about_c(&"atzur");
//  
//     // let facts: Vec<Box<dyn Formatting>> = vec![
//     //     Box::new("hi")
//     // ];
//     // for fact in facts {
//     //     fact.Endeve();
//     // }
// }

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;


use std::collections::HashMap;
use clap::Parser;

mod cli;
// // mod recipe;
mod video;
// // mod exec;

fn main() {
    color_eyre::install().unwrap();
    
    let _args = cli::Arguments::parse();
    let rc: HashMap <String, String> = HashMap::new();
    // let _rc = recipe::get_recipe(args); 
    let _videos = video::resolve_input(rc, _args);
    // exec::smoothing(videos);
}