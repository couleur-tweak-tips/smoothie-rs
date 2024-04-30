

use crate::cmd::SmCommand;
use std::process::{Command, Stdio};

use crate::verb;
use std::env;

pub fn vspipe_render(commands: Vec<SmCommand>) {
    for cmd in commands {
        let previewing: bool =
            cmd.recipe.get_bool("preview window", "enabled") && cmd.ffplay_args.is_some();

        verb!("FF args: {}", cmd.ff_args.join(" "));

        if previewing {
            verb!(
                "FFplay args: {}",
                &cmd.ffplay_args.clone().unwrap().join(" ")
            );
        }

        let mut vs = Command::new(cmd.vs_path)
            .args(cmd.vs_args)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed in spawning FFmpeg child");

        let pipe = vs.stdout.take().expect("Failed piping out of VSPipe");

        let mut ffmpeg = Command::new(cmd.ff_path)
            .args(cmd.ff_args)
            .stdin(pipe)
            .stdout(if previewing {
                Stdio::piped()
            } else {
                Stdio::inherit()
            })
            .spawn()
            .expect("Failed in spawning FFmpeg child");

        if previewing {
            let ffplay_pipe = ffmpeg.stdout.take().expect("Failed piping out of FFmpeg");
            let ffplay = Command::new(cmd.ffplay_path.unwrap())
                .args(cmd.ffplay_args.unwrap())
                .stdin(ffplay_pipe)
                .spawn()
                .expect("Failed in spawning ffplay child");
            ffplay.wait_with_output().unwrap();
        }

        vs.wait_with_output().unwrap();
        let status = ffmpeg.wait_with_output().unwrap().status;
        if !status.success() {
            panic!("ffmpeg / vapoursynth did not return sucessfully\n\nIF YOU ARE TAKING A SCREENSHOT WHEN ASKING FOR SUPPORT MAKE SURE TO INCLUDE THE TERMINAL's WHICH IS WHERE THE ERROR IS EXPLAINED");
        }
    }
}
/*
use std::io::prelude::*;
use std::io::BufReader;
use regex::Regex;
use std::process::{ChildStderr};
use indicatif::{ProgressBar, ProgressStyle};
use crate::ffpb::ffmpeg;

pub fn _teres_render(commands: Vec<SmCommand>) {
    for cmd in commands {
        let vspipe = Command::new(cmd.vs_path)
            .args(cmd.vs_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start vspipe process");

        let ffmpeg = Command::new(cmd.ff_path)
            .args(cmd.ff_args)
            .stdin(Stdio::from(
                vspipe.stdout.expect("Failed to open vspipe stdout"),
            ))
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start ffmpeg process");

        // dbg!("Spawned subprocesses");

        let progress = ProgressBar::new(100);
        progress.set_style(
            ProgressStyle::default_bar()
                .template(
                    format!(
                        " [{}] {{wide_bar:.cyan/blue}} {{percent}}% | ETA: {{eta_precise}}",
                        cmd.payload.basename
                    )
                        .as_str(),
                )
                .unwrap(),
        );

        _teres_progress(vspipe.stderr.unwrap(), progress);

        dbg!(ffmpeg.wait_with_output().unwrap().status);
    }
}

fn _teres_progress(stderr: ChildStderr, progress: ProgressBar) {
    let mut read_frames = false;
    let frame_regex = Regex::new(r"Frame: (?P<current>\d+)/(?P<total>\d+)").unwrap();
    let output_regex = Regex::new(r"Output").unwrap();
    let mut buf = BufReader::new(stderr);

    loop {
        let mut byte_vec = vec![];
        buf.read_until(b'\r', &mut byte_vec).expect("stderr Error");
        let string = String::from_utf8_lossy(&byte_vec);
        if output_regex.is_match(&string) {
            break;
        }
        let caps;
        if frame_regex.is_match(&string) {
            caps = frame_regex.captures(&string).unwrap();
            if !read_frames {
                progress.set_length(caps["total"].parse::<u64>().unwrap());
                read_frames = true
            }
            progress.set_position(caps["current"].parse::<u64>().unwrap())
        }
    }
}

pub fn _vspipe_render(commands: Vec<SmCommand>) {
    for cmd in commands {
        let previewing: bool =
            cmd.recipe.get_bool("preview window", "enabled") && cmd.ffplay_args.is_some();

        verb!("FF args: {}", cmd.ff_args.join(" "));

        if previewing {
            verb!(
                "FFplay args: {}",
                &cmd.ffplay_args.clone().unwrap().join(" ")
            );
        }

        let mut vs = Command::new(cmd.vs_path)
            .args(cmd.vs_args)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed in spawning FFmpeg child");

        let pipe = vs.stdout.take().expect("Failed piping out of VSPipe");

        // if ffmpeg(cmd.ff_args, pipe).is_ok() {
        //     println!("okie we good");
        // } else {
        //     panic!("Failed rendering");
        // }

        let mut ffmpeg = Command::new(cmd.ff_path)
            .args(cmd.ff_args)
            .stdin(pipe)
            .stdout(if previewing {
                Stdio::piped()
            } else {
                Stdio::inherit()
            })
            .spawn()
            .expect("Failed in spawning FFmpeg child");

        if previewing {
            let ffplay_pipe = ffmpeg.stdout.take().expect("Failed piping out of FFmpeg");

            let ffplay = Command::new(cmd.ffplay_path.unwrap())
                .args(cmd.ffplay_args.unwrap())
                .stdin(ffplay_pipe)
                .spawn()
                .expect("Failed in spawning ffplay child");

            ffplay.wait_with_output().unwrap();
        }

        vs.wait_with_output().unwrap();
        ffmpeg.wait_with_output().unwrap();
    }
}


fn _old_ffpb(cmd: SmCommand, previewing: bool) {
    let mut vs = Command::new(cmd.vs_path)
        .args(cmd.vs_args)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed in spawning FFmpeg child");

    let pipe = vs.stdout.take().expect("Failed piping out of VSPipe");

    let mut ffmpeg = Command::new(cmd.ff_path)
        .args(cmd.ff_args)
        .stdin(pipe)
        .stdout(if previewing {
            Stdio::piped()
        } else {
            Stdio::inherit()
        })
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed in spawning FFmpeg child");

    let _ff_stats = ffmpeg.stderr.take().expect("Failed capturing FFmpeg");

    // ffpb2::ffmpeg2(ff_stats).expect("Failed rendering ffmpeg");

    vs.wait_with_output().expect("failed waiting VapourSynth");
    ffmpeg.wait_with_output().expect("failed waiting ffmpeg");

    if previewing {
        // let ffplay_pipe = ffmpeg.stdout.take().expect("Failed piping out of FFmpeg");
        // let ffplay = Command::new(cmd.ffplay_path.unwrap())
        //     .args(cmd.ffplay_args.unwrap())
        //     .stdin(ffplay_pipe)
        //     .spawn()
        //     .expect("Failed in spawning ffplay child");
        // ffplay.wait_with_output().unwrap();
    }
}
*/