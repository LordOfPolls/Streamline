use std::fs;
use std::fs::DirEntry;
use std::io;
use std::path::Path;

use indicatif::ProgressBar;

use crate::models::file::MediaFile;
use models::config::CONFIG;

mod ffmpeg;
mod ffprobe;
mod models;
mod utils;

fn main() {
    let extensions = &CONFIG.streamline.file_extensions;
    let path = Path::new(&CONFIG.streamline.source_directory);

    sanity_check(path);
    CONFIG.display();

    let collection_spinner = utils::create_spinner(false);
    let files = collect_files_with_extensions(
        path,
        extensions,
        CONFIG.streamline.recursive,
        0,
        CONFIG.streamline.max_depth,
        &collection_spinner,
    )
    .unwrap();
    collection_spinner.finish_with_message(format!("✅ Files collected! Found: {}", files.len()));

    let to_process = check_files(files);

    let processing_pb = utils::create_progress_bar(to_process.len() as u64, true, 500);
    processing_pb.set_message("Processing files...");
    for file in to_process {
        processing_pb.set_message(format!("Processing: {}", file.path.path().display()));
        processing_pb.tick();
        match ffmpeg::process_file(&file) {
            Ok(_) => processing_pb.inc(1),
            Err(e) => {
                println!("Error processing file: {}", e)
            }
        }
    }
    utils::set_pb_finish_message(&processing_pb, "✅ Files processed!".to_string());
}

fn debug_print_ln(s: &str) {
    if CONFIG.streamline.debug {
        println!("{}", s);
    }
}

fn _check_file(file: MediaFile, to_process: &mut Vec<MediaFile>) {
    if CONFIG.video_targets.force_filter || CONFIG.audio_targets.force_filter {
        debug_print_ln("Force filter enabled - processing all files");
        to_process.push(file);
        return;
    }

    let video_streams = file
        .info
        .streams
        .iter()
        .filter(|s| s.codec_type == "video")
        .collect::<Vec<_>>();
    let audio_streams = file
        .info
        .streams
        .iter()
        .filter(|s| s.codec_type == "audio")
        .collect::<Vec<_>>();

    for stream in video_streams {
        if !CONFIG.video_targets.codec.is_empty()
            && !CONFIG.video_targets.codec.contains(&stream.codec_name)
        {
            debug_print_ln(&format!("Codec: {} not in target list", stream.codec_name));
            to_process.push(file);
            return;
        }

        if (CONFIG.video_targets.max_height != 0
            && stream.height.unwrap() > CONFIG.video_targets.max_height)
            || (CONFIG.video_targets.max_width != 0
                && stream.width.unwrap() > CONFIG.video_targets.max_width)
        {
            debug_print_ln(&format!(
                "Resolution {}x{} exceeds target {}x{}",
                stream.width.unwrap(),
                stream.height.unwrap(),
                CONFIG.video_targets.max_width,
                CONFIG.video_targets.max_height
            ));
            to_process.push(file);
            return;
        }

        if CONFIG.video_targets.max_fps != 0.0
            && stream.avg_frame_rate > CONFIG.video_targets.max_fps
        {
            debug_print_ln(&format!(
                "FPS: {} exceeds target {}",
                stream.avg_frame_rate, CONFIG.video_targets.max_fps
            ));
            to_process.push(file);
            return;
        }

        if CONFIG.video_targets.max_bitrate != 0
            && stream.bit_rate > CONFIG.video_targets.max_bitrate
        {
            debug_print_ln(&format!(
                "Bitrate: {} exceeds target {}",
                stream.bit_rate, CONFIG.video_targets.max_bitrate
            ));
            to_process.push(file);
            return;
        }
    }

    for stream in audio_streams {
        if !CONFIG.audio_targets.codec.is_empty()
            && !CONFIG.audio_targets.codec.contains(&stream.codec_name)
        {
            debug_print_ln(&format!("Codec: {} not in target list", stream.codec_name));
            to_process.push(file);
            return;
        }
    }
}

fn check_files(files: Vec<DirEntry>) -> Vec<MediaFile> {
    let pb = utils::create_progress_bar(files.len() as u64, true, 500);
    pb.set_message("Analyzing files...");
    let mut needs_processing = Vec::new();

    let total_files = files.len();

    for file in files {
        pb.inc(1);
        pb.tick();

        let info = ffprobe::get_file_info(&file);

        let media_file = match info {
            Ok(info) => MediaFile { path: file, info },
            Err(e) => {
                println!("Error processing file: {}", e);
                continue;
            }
        };

        _check_file(media_file, &mut needs_processing);
    }
    utils::set_pb_finish_message(
        &pb,
        format!(
            "✅ Files Analyzed! Found: {}/{} that need processing",
            needs_processing.len(),
            total_files
        ),
    );

    needs_processing
}

fn sanity_check(path: &Path) {
    let spinner = utils::create_spinner(true);
    spinner.set_message("Sanity checking".to_string());

    let mut failed = false;

    match fs::read_dir(path) {
        Ok(_) => spinner.tick(),
        Err(e) => {
            println!("Error reading directory: {}", e);
            failed = true;
        }
    }

    match ffmpeg::check_ffmpeg() {
        Ok(_) => spinner.tick(),
        Err(e) => {
            println!("{}", e);
            failed = true;
        }
    }

    match ffprobe::check_ffprobe() {
        Ok(_) => spinner.tick(),
        Err(e) => {
            println!("{}", e);
            failed = true;
        }
    }

    failed = CONFIG.sanity_check() || failed;

    if failed {
        println!("❌ Sanity check failed!");
        std::process::exit(1);
    }
    spinner.finish_with_message("✅ Sanity check passed!");
}

fn collect_files_with_extensions(
    path: &Path,
    extensions: &Vec<String>,
    recursive: bool,
    depth: u32,
    max_depth: u32,
    spinner: &ProgressBar,
) -> io::Result<Vec<DirEntry>> {
    let mut files: Vec<DirEntry> = Vec::new();

    if !CONFIG.streamline.exclude_directories.is_empty() {
        for dir in &CONFIG.streamline.exclude_directories {
            if path.ends_with(dir) {
                return Ok(files);
            }
        }
    }

    if depth <= max_depth {
        let objs = fs::read_dir(path)?;
        spinner.set_message(format!("Collecting files... Searching {}", path.display()));

        for obj in objs {
            let obj = obj?;
            let path = obj.path();
            spinner.tick();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if extensions.contains(&ext.to_string_lossy().into_owned()) {
                        files.push(obj);
                    }
                }
            } else if recursive && path.is_dir() {
                let sub_files = collect_files_with_extensions(
                    &path,
                    extensions,
                    recursive,
                    depth + 1,
                    max_depth,
                    spinner,
                )?;
                files.extend(sub_files);
            }
        }
    }
    Ok(files)
}
