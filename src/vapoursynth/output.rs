use std::{
    cmp,
    collections::HashMap,
    fs::File,
    io::{Cursor, Write},
    process::{ChildStdin, Stdio},
    sync::{Arc, Condvar, Mutex},
    time::Instant,
};

use anyhow::{anyhow, bail, Context, Error};
use num_rational::Ratio;
use rustsynth::{
    format::{ColorFamily, SampleType},
    frame::{Frame, FrameRef},
    node::{GetFrameError, Node},
};

pub struct OutputParameters<'core> {
    pub node: Node<'core>,
    pub start_frame: usize,
    pub end_frame: usize,
    pub requests: usize,
    pub y4m: bool,
}

struct OutputState<'core, T: Write> {
    output_target: T,
    timecodes_file: Option<File>,
    error: Option<(usize, Error)>,
    reorder_map: HashMap<usize, (Option<FrameRef<'core>>, Option<FrameRef<'core>>)>,
    last_requested_frame: usize,
    next_output_frame: usize,
    current_timecode: Ratio<i64>,
    callbacks_fired: usize,
    callbacks_fired_alpha: usize,
    last_fps_report_time: Instant,
    last_fps_report_frames: usize,
    fps: Option<f64>,
}

struct SharedData<'core, T: Write> {
    output_done_pair: (Mutex<bool>, Condvar),
    output_parameters: OutputParameters<'core>,
    output_state: Mutex<OutputState<'core, T>>,
}

fn print_y4m_header<W: Write>(writer: &mut W, node: &Node) -> Result<(), Error> {
    let info = node.video_info().unwrap();

    let format = info.format;
    write!(writer, "YUV4MPEG2 C")?;

    match format.color_family {
        ColorFamily::Gray => {
            write!(writer, "mono")?;
            if format.bits_per_sample > 8 {
                write!(writer, "{}", format.bits_per_sample)?;
            }
        }
        ColorFamily::YUV => {
            write!(
                writer,
                "{}",
                match (format.sub_sampling_w, format.sub_sampling_h) {
                    (1, 1) => "420",
                    (1, 0) => "422",
                    (0, 0) => "444",
                    (2, 2) => "410",
                    (2, 0) => "411",
                    (0, 1) => "440",
                    _ => bail!("No y4m identifier exists for the current format"),
                }
            )?;

            if format.bits_per_sample > 8 && format.sample_type == SampleType::Integer {
                write!(writer, "p{}", format.bits_per_sample)?;
            } else if format.sample_type == SampleType::Float {
                write!(
                    writer,
                    "p{}",
                    match format.bits_per_sample {
                        16 => "h",
                        32 => "s",
                        64 => "d",
                        _ => unreachable!(),
                    }
                )?;
            }
        }
        _ => bail!("No y4m identifier exists for the current format"),
    }

    write!(writer, " W{} H{}", info.width, info.height)?;

    write!(writer, " F{}:{}", info.fps_num, info.fps_den)?;

    writeln!(writer, " Ip A0:0 XLENGTH={}", info.num_frames)?;

    Ok(())
}

// Checks if the frame is completed, that is, we have the frame and, if needed, its alpha part.
fn is_completed(entry: &(Option<FrameRef>, Option<FrameRef>), have_alpha: bool) -> bool {
    entry.0.is_some() && (!have_alpha || entry.1.is_some())
}

fn print_frame<W: Write>(writer: &mut W, frame: &Frame) -> Result<(), Error> {
    const RGB_REMAP: [usize; 3] = [1, 2, 0];

    let format = frame.video_format().unwrap();
    #[allow(clippy::needless_range_loop)]
    for plane in 0..format.num_planes {
        let plane = if format.color_family == ColorFamily::RGB {
            RGB_REMAP[plane as usize]
        } else {
            plane.try_into().unwrap()
        };

        if let Ok(data) = frame.data(plane.try_into().unwrap()) {
            writer.write_all(data)?;
        } else {
            for row in 0..frame.height(plane.try_into().unwrap()) {
                writer.write_all(frame.data_row(plane.try_into().unwrap(), row))?;
            }
        }
    }

    Ok(())
}

fn print_frames<W: Write>(
    writer: &mut W,
    parameters: &OutputParameters,
    frame: &Frame,
    alpha_frame: Option<&Frame>,
) -> Result<(), Error> {
    if parameters.y4m {
        writeln!(writer, "FRAME").context("Couldn't output the frame header")?;
    }

    print_frame(writer, frame).context("Couldn't output the frame")?;
    if let Some(alpha_frame) = alpha_frame {
        print_frame(writer, alpha_frame).context("Couldn't output the alpha frame")?;
    }

    Ok(())
}

fn update_timecodes<T: Write>(frame: &Frame, state: &mut OutputState<T>) -> Result<(), Error> {
    let props = frame.props();
    let duration_num = props
        .get_int("_DurationNum")
        .context("Couldn't get the duration numerator")?;
    let duration_den = props
        .get_int("_DurationDen")
        .context("Couldn't get the duration denominator")?;

    if duration_den == 0 {
        bail!("The duration denominator is zero");
    }

    state.current_timecode += Ratio::new(duration_num, duration_den);

    Ok(())
}

fn frame_done_callback<'core, T: Write + Send + 'core>(
    frame: Result<FrameRef<'core>, GetFrameError>,
    n: usize,
    _node: &Node<'core>,
    shared_data: &Arc<SharedData<'core, T>>,
    alpha: bool,
) {
    let parameters = &shared_data.output_parameters;
    let mut state = shared_data.output_state.lock().unwrap();

    // Increase the progress counter.
    if !alpha {
        state.callbacks_fired += 1;
        state.callbacks_fired_alpha += 1;
    } else {
        state.callbacks_fired_alpha += 1;
    }

    // Figure out the FPS.
    //if parameters.progress {
    //    let current = Instant::now();
    //    let elapsed = current.duration_since(state.last_fps_report_time);
    //    let elapsed_seconds = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9;

    //    if elapsed.as_secs() > 10 {
    //        state.fps = Some(
    //            (state.callbacks_fired - state.last_fps_report_frames) as f64 / elapsed_seconds,
    //        );
    //        state.last_fps_report_time = current;
    //       state.last_fps_report_frames = state.callbacks_fired;
    //    }
    //}

    match frame {
        Err(error) => {
            if state.error.is_none() {
                state.error = Some((
                    n,
                    anyhow!(error.into_inner().to_string_lossy().into_owned()),
                ))
            }
        }
        Ok(frame) => {
            // Store the frame in the reorder map.
            {
                let entry = state.reorder_map.entry(n).or_insert((None, None));
                if alpha {
                    entry.1 = Some(frame);
                } else {
                    entry.0 = Some(frame);
                }
            }

            // If we got both a frame and its alpha frame, request one more.
            if is_completed(&state.reorder_map[&n], false)
                && state.last_requested_frame < parameters.end_frame
                && state.error.is_none()
            {
                let shared_data_2 = shared_data.clone();
                parameters.node.get_frame_async(
                    state.last_requested_frame + 1,
                    move |frame, n, node| {
                        frame_done_callback(frame, n, &node, &shared_data_2, false)
                    },
                );

                state.last_requested_frame += 1;
            }

            // Output all completed frames.
            while state
                .reorder_map
                .get(&state.next_output_frame)
                .map(|entry| is_completed(entry, false))
                .unwrap_or(false)
            {
                let next_output_frame = state.next_output_frame;
                let (frame, alpha_frame) = state.reorder_map.remove(&next_output_frame).unwrap();

                let frame = frame.unwrap();
                if state.error.is_none() {
                    if let Err(error) = print_frames(
                        &mut state.output_target,
                        parameters,
                        &frame,
                        alpha_frame.as_deref(),
                    ) {
                        state.error = Some((n, error));
                    }
                }

                if state.timecodes_file.is_some() && state.error.is_none() {
                    let timecode = (*state.current_timecode.numer() as f64 * 1000f64)
                        / *state.current_timecode.denom() as f64;
                    match writeln!(state.timecodes_file.as_mut().unwrap(), "{:.6}", timecode)
                        .context("Couldn't output the timecode")
                    {
                        Err(error) => state.error = Some((n, error)),
                        Ok(()) => {
                            if let Err(error) = update_timecodes(&frame, &mut state)
                                .context("Couldn't update the timecodes")
                            {
                                state.error = Some((n, error));
                            }
                        }
                    }
                }

                state.next_output_frame += 1;
            }
        }
    }

    // Output the progress info.
    //if parameters.progress {
    //    eprint!(
    //        "Frame: {}/{}",
    //        state.callbacks_fired,
    //        parameters.end_frame - parameters.start_frame + 1
    //    );

    //    if let Some(fps) = state.fps {
    //        eprint!(" ({:.2} fps)", fps);
    //    }

    //    eprint!("\r");
    //}

    // if state.next_output_frame == parameters.end_frame + 1 {
    // This condition works with error handling:
    let frames_requested = state.last_requested_frame - parameters.start_frame + 1;
    if state.callbacks_fired == frames_requested && state.callbacks_fired_alpha == frames_requested
    {
        *shared_data.output_done_pair.0.lock().unwrap() = true;
        shared_data.output_done_pair.1.notify_one();
    }
}

pub fn output<T: std::io::Write + std::marker::Send>(
    mut output_target: T,
    mut timecodes_file: Option<File>,
    parameters: OutputParameters,
) -> Result<(), Error> {
    // Print the y4m header.
    if parameters.y4m {
        print_y4m_header(&mut output_target, &parameters.node)
            .context("Couldn't write the y4m header")?;
    }

    // Print the timecodes header.
    if let Some(ref mut timecodes_file) = timecodes_file {
        writeln!(timecodes_file, "# timecode format v2")?;
    }

    let initial_requests = cmp::min(
        parameters.requests,
        parameters.end_frame - parameters.start_frame + 1,
    );

    let output_done_pair = (Mutex::new(false), Condvar::new());
    let output_state = Mutex::new(OutputState {
        output_target,
        timecodes_file,
        error: None,
        reorder_map: HashMap::new(),
        last_requested_frame: parameters.start_frame + initial_requests - 1,
        next_output_frame: 0,
        current_timecode: Ratio::from_integer(0),
        callbacks_fired: 0,
        callbacks_fired_alpha: 0,
        last_fps_report_time: Instant::now(),
        last_fps_report_frames: 0,
        fps: None,
    });
    let shared_data = Arc::new(SharedData {
        output_done_pair,
        output_parameters: parameters,
        output_state,
    });

    // Record the start time.
    let start_time = Instant::now();

    // Start off by requesting some frames.
    {
        let parameters = &shared_data.output_parameters;
        for n in 0..initial_requests {
            let shared_data_2 = shared_data.clone();
            parameters.node.get_frame_async(n, move |frame, n, node| {
                frame_done_callback(frame, n, &node, &shared_data_2, false)
            });
        }
    }

    let &(ref lock, ref cvar) = &shared_data.output_done_pair;
    let mut done = lock.lock().unwrap();
    while !*done {
        done = cvar.wait(done).unwrap();
    }

    let elapsed = start_time.elapsed();
    let elapsed_seconds = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9;

    let mut state = shared_data.output_state.lock().unwrap();
    eprintln!(
        "Output {} frames in {:.2} seconds ({:.2} fps)",
        state.next_output_frame,
        elapsed_seconds,
        state.next_output_frame as f64 / elapsed_seconds
    );

    if let Some((n, ref msg)) = state.error {
        bail!("Failed to retrieve frame {} with error: {}", n, msg);
    }

    // Flush the output file.
    state
        .output_target
        .flush()
        .context("Failed to flush the output file")?;

    Ok(())
}
