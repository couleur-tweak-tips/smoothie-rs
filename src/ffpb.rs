use kdam::{tqdm, BarExt, Column, RichProgress};
use regex::{Captures, Regex};
use std::{
    io::{stderr, BufRead, BufReader, Error, ErrorKind, IsTerminal},
    num::ParseIntError,
    process::ChildStderr,
    thread,
    time::Duration,
};

fn new_error(msg: &str) -> Error {
    Error::new(ErrorKind::Other, msg)
}

fn time_to_secs(x: &Captures) -> Result<usize, ParseIntError> {
    let hours = x.get(1).unwrap().as_str().parse::<usize>()?;
    let minutes = x.get(2).unwrap().as_str().parse::<usize>()?;
    let seconds = x.get(3).unwrap().as_str().parse::<usize>()?;
    Ok((((hours * 60) + minutes) * 60) + seconds)
}

pub fn ffmpeg(ffmpeg: ChildStderr, duration: usize, fps: Option<i32>) -> Result<(), Error> {
    kdam::term::init(stderr().is_terminal());

    let mut reader = BufReader::new(ffmpeg);
    let mut pb = RichProgress::new(
        tqdm!(unit = "s".to_owned(), dynamic_ncols = true),
        vec![
            Column::Animation,
            Column::Percentage(1),
            Column::Text("Rendered:".to_owned()),
            Column::CountTotal,
            Column::Text("Time:".to_owned()),
            Column::ElapsedTime,
            Column::Text("ETA:".to_owned()),
            Column::RemainingTime,
            Column::Text("Speed:".to_owned()),
            Column::Text("0 FPS".to_owned()),
        ],
    );

    pb.pb.total = duration;

    let read_byte = b'\r';
    let progress_rx = Regex::new(r"time=(\d{2}):(\d{2}):(\d{2})\.\d{2}").unwrap();
    let mut last_update = std::time::Instant::now();

    loop {
        thread::sleep(Duration::from_millis(100));

        let mut buf = vec![];
        reader.read_until(read_byte, &mut buf)?;

        if let Ok(line) = String::from_utf8(buf) {
            if line.is_empty() {
                pb.refresh()?;
                println!();
                break;
            }

            if let Some(x) = progress_rx.captures_iter(&line).next() {
                let mut current =
                    time_to_secs(&x).map_err(|_| new_error("couldn't parse current duration."))?;

                if let Some(frames) = fps {
                    current *= frames as usize;
                    if pb.pb.total == duration {
                        pb.pb.total *= frames as usize;
                    }
                }

                pb.replace(9, Column::Text(format!("{:.0} FPS", pb.pb.rate())));
                if last_update.elapsed() > Duration::from_millis(200) {
                    pb.update_to(current)?;
                    last_update = std::time::Instant::now();
                }
            }
        } else {
            break;
        }
    }

    Ok(())
}
