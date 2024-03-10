use std::fs::DirEntry;
use std::process::Command;
use std::thread::sleep;
use crate::models::config::{CONFIG};
use crate::models::file::MediaFile;
use crate::models::media::{Stream};

pub fn check_ffmpeg() -> Result<(), String> {
    match Command::new(&CONFIG.ffmpeg.ffmpeg_path).arg("-version").output() {
        Ok(_) => Ok(()),
        Err(_) => Err("FFmpeg not found!".to_string()),
    }
}

pub fn get_output_path(input_file: &DirEntry) -> String {
    let mut working_path = input_file.path().with_extension(&CONFIG.output.output_extension).to_str().unwrap().to_string();
    if !CONFIG.output.output_directory.is_empty() {
        working_path = working_path.split("/").last().unwrap().to_string();
        working_path = format!("{}/{}.{}", &CONFIG.output.output_directory, working_path, &CONFIG.output.output_extension);
    }
    return working_path;
}

fn apply_aspect_ratio_corrections(stream: &Stream, filters: &mut Vec<String>) {
    if !stream.width.is_some() || !stream.height.is_some() {
        // No width or height, nothing to do
        return;
    }
    if !CONFIG.video_targets.max_width != 0 &&
        !CONFIG.video_targets.max_height != 0{
        // No max width or height, nothing to do
            return;
    }

    let width = stream.width.unwrap();
    let height = stream.height.unwrap();
    if width <= CONFIG.video_targets.max_width &&
        height <= CONFIG.video_targets.max_height {
        // Video is already within target dimensions, nothing to do
            return;
    }

    let aspect_ratio = width as f32 / height as f32;
    let target_aspect_ratio = CONFIG.video_targets.max_width as f32 / CONFIG.video_targets.max_height as f32;

    let mut pad_width = 0;
    let mut pad_height = 0;

    if aspect_ratio > target_aspect_ratio {
        pad_height = CONFIG.video_targets.max_height - height;
    } else {
        pad_width = CONFIG.video_targets.max_width - width;
    }

    filters.push(format!("pad={}:{}:{}:{}",
        CONFIG.video_targets.max_width,
        CONFIG.video_targets.max_height,
        pad_width / 2,
        pad_height / 2));
}

fn apply_video_arguments(info: &Stream, command: &mut Command) {
    if !CONFIG.video_targets.codec.contains(&info.codec_name) {
        command.arg("-c:v").arg(&CONFIG.video_targets.codec[0]);
    }

    if CONFIG.video_targets.max_fps != 0.0 {
        command.arg("-r").arg(&CONFIG.video_targets.max_fps.to_string());
    }

    if CONFIG.video_targets.max_bitrate != 0 {
        command.arg("-b:v").arg(&CONFIG.video_targets.max_bitrate.to_string());
    }

    if CONFIG.video_targets.crf != -1 {
        command.arg("-crf").arg(&CONFIG.video_targets.crf.to_string());
    }

    if CONFIG.video_targets.ffmpeg_preset != "" {
        command.arg("-preset").arg(&CONFIG.video_targets.ffmpeg_preset);
    }

    if CONFIG.video_targets.pix_fmt != "" {
        command.arg("-pix_fmt").arg(&CONFIG.video_targets.pix_fmt);
    }

    if CONFIG.video_targets.tune != "" {
        command.arg("-tune").arg(&CONFIG.video_targets.tune);
    }

    if CONFIG.video_targets.x265_params != ""{
        command.arg("-x265-params").arg(&CONFIG.video_targets.x265_params);
    }
}

fn apply_audio_arguments(stream: &Stream, command: &mut Command) {
    if !CONFIG.audio_targets.sample_rate.is_empty() {
        let source_rate = stream.sample_rate;
        if !CONFIG.audio_targets.sample_rate.contains(&source_rate) {
            command.arg(format!("-ar:{} {}", stream.index, &CONFIG.audio_targets.sample_rate[0]));
        }
    }

    if CONFIG.audio_targets.channels != 0{
        command.arg(format!("-ac:{} {}", stream.index, &CONFIG.audio_targets.channels.to_string()));
    }
}

pub fn process_file(input_file: &MediaFile) -> Result<(), String> {
    let output_file = get_output_path(&input_file.path);

    let mut command = Command::new(&CONFIG.ffmpeg.ffmpeg_path);
    let mut filters = Vec::new();

    command.arg(format!("-i {}", input_file.path.path().to_str().unwrap()));


    let video_streams = input_file.info.streams.iter().filter(|s| s.codec_type == "video").collect::<Vec<_>>();
    let audio_streams = input_file.info.streams.iter().filter(|s| s.codec_type == "audio").collect::<Vec<_>>();


    if video_streams.len() == 0 {
        return Err("No video streams found!".to_string());
    }
    apply_video_arguments(&video_streams[0], &mut command);
    apply_aspect_ratio_corrections(&video_streams[0], &mut filters);

    apply_audio_arguments(audio_streams[0], &mut command);

    if filters.len() > 0 {
        command.arg("-vf").arg(filters.join(","));
    }
    command.arg(output_file);
    sleep(std::time::Duration::from_millis(500));

    // println!("Running command: {:?}", command);

    Ok(())
}