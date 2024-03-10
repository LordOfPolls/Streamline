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
    pub video_targets: VideoTargets,
    pub audio_targets: AudioTargets,
    pub subtitles: Subtitles,
    pub filters: Filters,
}

impl Config {
    pub fn sanity_check(&self) -> bool {
        let mut failed: bool = false;
        if self.streamline.always_replace {
            if self.streamline.replace_if_smaller {
                println!("Error: always_replace and replace_if_smaller cannot both be true");
                failed = true;
            }
            if self.streamline.output_directory != "" {
                println!("Error: always_replace and output_directory cannot both be set");
                failed = true;
            }
        }

        if self.streamline.max_depth == 0 {
            println!("Error: max_depth cannot be 0 - It must be a sane positive number");
            failed = true;
        }

        if self.streamline.temporary_suffix == "" {
            println!("Error: temporary_suffix cannot be empty");
            failed = true;
        }

        if !Path::new(&self.streamline.temp_directory).exists() {
            fs::create_dir_all(&self.streamline.temp_directory).unwrap();
        }

        if !Path::new(&self.streamline.output_directory).exists() {
            fs::create_dir_all(&self.streamline.output_directory).unwrap();
        }

        return failed;
    }
}

#[derive(Debug, Deserialize)]
pub struct StreamLine {
    pub source_directory: String,
    pub exclude_directories: Vec<String>,
    pub recursive: bool,
    pub max_depth: u32,
    pub file_extensions: Vec<String>,
    pub dry_run: bool,
    pub debug: bool,
    pub output_extension: String,
    pub replace_if_smaller: bool,
    pub always_replace: bool,
    pub temp_directory: String,
    pub temporary_suffix: String,
    pub output_directory: String,
}

#[derive(Debug, Deserialize)]
pub struct FFmpeg {
    pub ffmpeg_path: String,
    pub ffprobe_path: String,
    pub threads: u32,
    pub log_level: String,
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
    pub filters: String,
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
