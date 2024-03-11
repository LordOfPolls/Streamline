use indicatif::ProgressBar;
use std::collections::VecDeque;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::models::config::CONFIG;
use crate::models::file::MediaFile;
use crate::models::media::FFProbeOutput;
use crate::utils;
use std::fs::DirEntry;

pub fn check_ffprobe() -> Result<(), String> {
    match Command::new(&CONFIG.ffmpeg.ffprobe_path)
        .arg("-version")
        .output()
    {
        Ok(_) => Ok(()),
        Err(_) => Err("FFprobe not found!".to_string()),
    }
}

pub fn call_ffprobe(file: &DirEntry) -> Result<String, String> {
    let output = Command::new(&CONFIG.ffmpeg.ffprobe_path)
        .arg("-v")
        .arg("quiet")
        .arg("-show_format")
        .arg("-show_streams")
        .arg("-show_entries")
        .arg("stream_tags:format_tags")
        .arg("-print_format")
        .arg("json")
        .arg(&file.path())
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let output = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        Ok(output)
    } else {
        let error = String::from_utf8(output.stderr).map_err(|e| e.to_string())?;
        Err(error)
    }
}

pub fn bulk_get_file_info(files: Vec<DirEntry>) -> Vec<MediaFile> {
    let allowed_workers = CONFIG.ffmpeg.ffprobe_workers;

    let files = Arc::new(Mutex::new(VecDeque::from(files)));
    let pb = utils::create_progress_bar(files.lock().unwrap().len() as u64, true, 500);
    pb.set_message("Probing files...");
    let shared_pb = Arc::new(Mutex::new(pb));

    let results = Arc::new(Mutex::new(Vec::new()));
    let mut workers = vec![];

    for _ in 0..allowed_workers {
        let files = files.clone();
        let results = results.clone();
        let prg = shared_pb.clone();
        workers.push(thread::spawn(move || {
            probe_file_worker(files, results, prg);
        }));
    }

    for worker in workers {
        worker.join().unwrap();
    }

    let output = results.lock().unwrap().drain(..).flatten().collect();
    utils::set_pb_finish_message(&shared_pb.lock().unwrap(), "✅ Files probed!".to_string());
    output
}

fn probe_file_worker(
    files: Arc<Mutex<VecDeque<DirEntry>>>,
    results: Arc<Mutex<Vec<Result<MediaFile, String>>>>,
    pb: Arc<Mutex<ProgressBar>>,
) {
    loop {
        let file = match files.lock().unwrap().pop_front() {
            Some(file) => file,
            None => break,
        };

        let info = match get_file_info(&file) {
            Ok(info) => info,
            Err(e) => {
                results.lock().unwrap().push(Err(e));
                pb.lock().unwrap().inc(1);
                continue;
            }
        };

        results.lock().unwrap().push(Ok(MediaFile {
            path: file.path(),
            info,
        }));
        pb.lock().unwrap().inc(1);
    }
}

pub fn get_file_info(file: &DirEntry) -> Result<FFProbeOutput, String> {
    parse_ffprobe_output(&call_ffprobe(file)?)
}
fn parse_ffprobe_output(output: &str) -> Result<FFProbeOutput, String> {
    serde_json::from_str(output)
        .map_err(|e| format!("Error parsing ffprobe output: {}\nOutput: {}", e, output))
}
