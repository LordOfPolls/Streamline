use crate::utils;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs;
use std::path::Path;

pub static CONFIG: Lazy<Config> = Lazy::new(|| load_config());

fn load_config() -> Config {
    let spinner = utils::create_spinner(false);
    let config_path = Path::new("config.toml");
    let config = match fs::read_to_string(config_path) {
        Ok(config) => config,
        Err(e) => {
            println!("Error reading config file: {}", e);
            std::process::exit(1);
        }
    };

    match toml::from_str(&config) {
        Ok(config) => {
            spinner.finish_with_message("✅ Config loaded!");
            config
        }
        Err(e) => {
            println!("Error parsing config file: {}", e);
            std::process::exit(1);
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub streamline: StreamLine,
    pub ffmpeg: FFmpeg,
    pub output: Output,
    pub video_targets: VideoTargets,
    pub audio_targets: AudioTargets,
    pub subtitles: Subtitles,
    pub filters: Filters,
}

#[derive(Debug, Deserialize)]
pub struct StreamLine {
    pub source_directory: String,
    pub recursive: bool,
    pub max_depth: u32,
    pub file_extensions: Vec<String>,
    pub dry_run: bool,
    pub debug: bool,
}

#[derive(Debug, Deserialize)]
pub struct FFmpeg {
    pub ffmpeg_path: String,
    pub ffprobe_path: String,
    pub threads: u32,
    pub log_level: String,
}

#[derive(Debug, Deserialize)]
pub struct Output {
    pub temporary_suffix: String,
    pub temp_directory: String,
    pub output_directory: String,
    pub output_extension: String,
    pub output_format: String,
    pub delete_source: bool,
}

#[derive(Debug, Deserialize)]
pub struct VideoTargets {
    pub codec: Vec<String>,
    pub max_bitrate: u32,
    pub crf: i32,
    pub ffmpeg_preset: String,
    pub max_width: u32,
    pub max_height: u32,
    pub max_fps: f64,
    pub filters: String,
    pub force_filter: bool,
    pub pix_fmt: String,
    pub tune: String,
    pub x265_params: String,
}

#[derive(Debug, Deserialize)]
pub struct AudioTargets {
    pub codec: Vec<String>,
    pub language: Vec<String>,
    pub default_language: String,
    pub channel_bitrate: u32,
    pub variable_bitrate: u32,
    pub sample_rate: Vec<u32>,
    pub ffmpeg_profile: String,
    pub filter: String,
    pub force_filter: bool,
    pub channels: u32,
    pub aac_profile: String,
}

#[derive(Debug, Deserialize)]
pub struct Subtitles {
    pub codec: Vec<String>,
    pub language: Vec<String>,
    pub default_language: String,
    pub copy_subtitles: bool,
    pub force_subtitles: bool,
}

#[derive(Debug, Deserialize)]
pub struct Filters {
    pub deinterlace: bool,
    pub deblock: u32,
    pub denoise: u32,
}
