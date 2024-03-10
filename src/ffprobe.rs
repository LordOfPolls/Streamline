use std::process::Command;


use std::fs::DirEntry;
use serde_json;
use crate::models::config::{CONFIG};

use crate::models::media::{FFProbeOutput};

pub fn check_ffprobe() -> Result<(), String> {
    match Command::new(&CONFIG.ffmpeg.ffprobe_path).arg("-version").output() {
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

pub fn get_file_info(file: &DirEntry) -> Result<FFProbeOutput, String> {
    // ffprobe -v quiet -show_format -show_streams -show_entries stream_tags:format_tags -print_format json
    parse_ffprobe_output(&call_ffprobe(file)?)
}

fn parse_ffprobe_output(output: &str) -> Result<FFProbeOutput, String> {
    serde_json::from_str(output).map_err(|e| format!("Error parsing ffprobe output: {}\nOutput: {}", e, output))
}