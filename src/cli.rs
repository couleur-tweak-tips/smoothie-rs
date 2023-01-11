use std::path::PathBuf;
use clap::Parser;

/// Smoothen up your gameplay footage with Smoothie, yum! 
#[derive(Parser, Debug)]
#[clap(version, about, long_about = "h")]
pub struct Arguments {
    /// Input video file paths
    pub input:      Option<Vec<PathBuf>>,

    /// Payload containing video timecodes, used by NLE scripts
    pub json:       Option<String>,

    /// Output video filepath (one only)
    #[clap(short, long)]
    pub output:     Option<String>,

    /// Display extra "advanced" information, what I personally use
    #[clap(short, long, default_value_t = false)]
    pub verb:       bool,

    /// Join all cuts to a file, used with -json
    #[clap(short, long, default_value_t = false)]
    pub trim:       bool,

    /// Split all cuts to separate files, used with -json
    #[clap(short, long, default_value_t = false)]
    pub split:      bool,

    /// 
    #[clap(short, long, default_value_t = false)]
    pub debug:      bool,

    /// Specify a recipe path (defaults to recipe.yaml)
    #[clap(short, long, default_value = "recipe.ini")]
    pub recipe:     String,
}