use crate::cmd::SmCommand;
use crate::verb;
use std::env;
use std::process::{Command, Stdio};

pub fn vspipe_render(commands: Vec<SmCommand>, mut progress: bool) {
    for cmd in commands {
        let previewing: bool =
            cmd.recipe.get_bool("preview window", "enabled") && cmd.ffplay_args.is_some();

        if previewing && progress {
            progress = false;
            println!(
                "Progress bar is currently not compatible with preview window, disabling progress"
            )
        }

        verb!("FF args: {}", cmd.ff_args.join(" "));

        if previewing {
            verb!(
                "FFplay args: {}",
                &cmd.ffplay_args.clone().unwrap().join(" ")
            );
        }

        let vs = Command::new(cmd.vs_path)
            .args(cmd.vs_args)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed in spawning FFmpeg child");

        let pipe = vs.stdout.expect("Failed piping out of VSPipe");

        let mut ffmpeg = Command::new(cmd.ff_path)
            .args(cmd.ff_args)
            .stdin(pipe)
            .stdout(if previewing {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stderr(if progress {
                Stdio::piped()
            } else {
                Stdio::inherit()
            })
            .spawn()
            .expect("Failed in spawning FFmpeg child");

        if progress {
            let stderr = ffmpeg
                .stderr
                .expect("failed to capture ffmpeg standard error.");

            let fps: i32 = if cmd.recipe.get_bool("frame blending", "enabled") {
                cmd.recipe
                    .get("frame blending", "fps")
                    .parse::<i32>()
                    .unwrap()
            } else {
                for stream in cmd.payload.probe.streams {
                    if stream.codec_type == Some("video".to_owned()) {
                        stream.avg_frame_rate.parse::<i32>().unwrap();
                        break;
                    } else {
                        continue;
                    }
                }
                panic!("Failed finding a probe video stream");
            };

            let duration = cmd
                .payload
                .probe
                .format
                .duration
                .expect("Failed getting probe duration")
                .parse::<f32>()
                .unwrap()
                .round() as usize;

            let _a = crate::ffpb::ffmpeg(stderr, duration, Some(fps));
        } else {
            if previewing {
                let ffplay_pipe = ffmpeg.stdout.take().expect("Failed piping out of FFmpeg");
                let ffplay = Command::new(cmd.ffplay_path.unwrap())
                    .args(cmd.ffplay_args.unwrap())
                    .stdin(ffplay_pipe)
                    .spawn()
                    .expect("Failed in spawning ffplay child");
                ffplay.wait_with_output().unwrap();
            }
            let status = ffmpeg.wait_with_output().unwrap().status;
            if !status.success() {
                panic!("ffmpeg / vapoursynth did not return sucessfully\n\nIF YOU ARE TAKING A SCREENSHOT WHEN ASKING FOR SUPPORT MAKE SURE TO INCLUDE THE TERMINAL's WHICH IS WHERE THE ERROR IS EXPLAINED");
            }
        }
    }
}
