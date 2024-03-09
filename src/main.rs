use std::fs;
use std::fs::DirEntry;
use std::io;
use std::path::Path;
use std::process::Command;

use indicatif::ProgressBar;

use models::config::CONFIG;

mod ffprobe;
mod models;
mod ffmpeg;
mod utils;

fn main() {
    let extensions = &CONFIG.streamline.file_extensions;
    let path = Path::new(&CONFIG.streamline.source_directory);

    sanity_check(path);

    let collection_spinner = utils::create_spinner(true);
    let files = collect_files_with_extensions(path, extensions, CONFIG.streamline.recursive, 0, CONFIG.streamline.max_depth, &collection_spinner).unwrap();
    collection_spinner.finish_with_message(format!("✅ Files collected! Found: {}", files.len()));

    check_files(files);
}


fn check_files(files: Vec<DirEntry>) {
    let pb = utils::create_progress_bar(files.len() as u64);


    let mut needs_processing = Vec::new();

    for file in files {
        pb.inc(1);

        let info = ffprobe::get_file_info(&file);

        let info = match info {
            Ok(info) => {info}
            Err(e) => {println!("Error processing file: {}", e); continue;}
        };

        let video_streams = info.streams.iter().filter(|s| s.codec_type == "video").collect::<Vec<_>>();
        let audio_streams = info.streams.iter().filter(|s| s.codec_type == "audio").collect::<Vec<_>>();

        for stream in video_streams {
            if stream.codec_name != "h264" {
                needs_processing.push(file.path());
                break;
            }
        }

        for stream in audio_streams {
            if stream.codec_name == "aac" {
                needs_processing.push(file.path());
                break;
            }
        }


    };
    utils::set_pb_finish_message(&pb, format!("✅ Files checked! Found: {} that need processing", needs_processing.len()));
}

fn sanity_check(path: &Path) {
    let spinner = utils::create_spinner(true);
    spinner.set_message(format!("Sanity checking"));

    let mut failed = false;

    match fs::read_dir(path) {
        Ok(_) => {spinner.tick()}
        Err(e) => {
            println!("Error reading directory: {}", e);
            failed = true;
        }
    }

    match Command::new(&CONFIG.ffmpeg.ffmpeg_path).arg("-version").output() {
        Ok(_) => {spinner.tick()}
        Err(_) => {
            println!("FFmpeg not found!");
            failed = true;
        }
    }

    match ffprobe::check_ffprobe() {
        Ok(_) => {spinner.tick()}
        Err(e) => {
            println!("{}", e);
            failed = true;
        }
    }

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
) -> io::Result<Vec<DirEntry>>
{
    let mut files: Vec<DirEntry> = Vec::new();

    if depth <= max_depth {
        let objs = fs::read_dir(path)?;
        spinner.set_message(format!("Collecting files... Searching {}", path.display()));

        for obj in objs {
            let obj = obj?;
            let path = obj.path();

            if path.is_file() {
                match path.extension() {
                    Some(ext) => {
                        if extensions.contains(&ext.to_string_lossy().into_owned()) {
                            files.push(obj);

                        }
                    }
                    None => {}
                }
            }
            else if recursive && path.is_dir() {
                let sub_files = collect_files_with_extensions(&path, &extensions, recursive, depth + 1, max_depth, &spinner)?;
                files.extend(sub_files);
            }
        }
    }
    Ok(files)
}
