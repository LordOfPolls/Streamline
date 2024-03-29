use crate::models::config::CONFIG;
use crate::models::file::MediaFile;
use crate::models::media::Stream;
use crate::utils;
use std::path::Path;
use std::process::Command;

pub fn check_ffmpeg() -> Result<(), String> {
    match Command::new(&CONFIG.ffmpeg.ffmpeg_path)
        .arg("-version")
        .output()
    {
        Ok(_) => Ok(()),
        Err(_) => Err("FFmpeg not found!".to_string()),
    }
}

pub fn get_output_path(input_file: &Path) -> String {
    let mut working_path = input_file
        .with_extension(&CONFIG.streamline.output_extension)
        .to_str()
        .unwrap()
        .to_string();
    if !CONFIG.streamline.output_directory.is_empty() {
        working_path = working_path
            .split(&utils::get_system_path_separator())
            .last()
            .unwrap()
            .to_string();
        working_path = format!(
            "{}{}{}",
            &CONFIG.streamline.output_directory,
            utils::get_system_path_separator(),
            working_path
        );
    }
    format!("{}.{}", working_path, &CONFIG.streamline.temporary_suffix)
}

fn apply_aspect_ratio_corrections(stream: &Stream, filters: &mut Vec<String>) {
    if stream.width.is_none() || stream.height.is_none() {
        // No width or height, nothing to do
        return;
    }
    if !CONFIG.video_targets.max_width != 0 && !CONFIG.video_targets.max_height != 0 {
        // No max width or height, nothing to do
        return;
    }

    let width = stream.width.unwrap();
    let height = stream.height.unwrap();
    if width <= CONFIG.video_targets.max_width && height <= CONFIG.video_targets.max_height {
        // Video is already within target dimensions, nothing to do
        return;
    }

    let aspect_ratio = width as f32 / height as f32;
    let target_aspect_ratio =
        CONFIG.video_targets.max_width as f32 / CONFIG.video_targets.max_height as f32;

    let mut pad_width = 0;
    let mut pad_height = 0;

    if aspect_ratio > target_aspect_ratio {
        pad_height = CONFIG.video_targets.max_height - height;
    } else {
        pad_width = CONFIG.video_targets.max_width - width;
    }

    filters.push(format!(
        "pad={}:{}:{}:{}",
        CONFIG.video_targets.max_width,
        CONFIG.video_targets.max_height,
        pad_width / 2,
        pad_height / 2
    ));
}

fn apply_video_arguments(info: &Stream, command: &mut Command) {
    if !CONFIG.video_targets.codec.contains(&info.codec_name) {
        command.arg("-c:v").arg(&CONFIG.video_targets.codec[0]);
    }

    if CONFIG.video_targets.max_fps != 0.0 {
        command
            .arg("-r")
            .arg(&CONFIG.video_targets.max_fps.to_string());
    }

    if CONFIG.video_targets.max_bitrate != 0 {
        command
            .arg("-b:v")
            .arg(&CONFIG.video_targets.max_bitrate.to_string());
    }

    if CONFIG.video_targets.crf != -1 {
        command
            .arg("-crf")
            .arg(&CONFIG.video_targets.crf.to_string());
    }

    if !CONFIG.video_targets.ffmpeg_preset.is_empty() {
        command
            .arg("-preset")
            .arg(&CONFIG.video_targets.ffmpeg_preset);
    }

    if !CONFIG.video_targets.pix_fmt.is_empty() {
        command.arg("-pix_fmt").arg(&CONFIG.video_targets.pix_fmt);
    }

    if !CONFIG.video_targets.tune.is_empty() {
        command.arg("-tune").arg(&CONFIG.video_targets.tune);
    }

    if !CONFIG.video_targets.x265_params.is_empty() {
        command
            .arg("-x265-params")
            .arg(&CONFIG.video_targets.x265_params);
    }
}

fn apply_video_filters(filters: &mut Vec<String>) {
    if CONFIG.filters.deinterlace {
        filters.push("yadif".to_string());
    }

    if CONFIG.filters.deblock > 0 {
        filters.push(format!("deblock={}", CONFIG.filters.deblock));
    }

    if CONFIG.filters.denoise > 0 {
        filters.push(format!("hqdn3d={}", CONFIG.filters.denoise));
    }
}

fn apply_audio_arguments(stream: &Stream, command: &mut Command, default_set: &mut bool) {
    if !CONFIG.audio_targets.codec.is_empty()
        && !CONFIG.audio_targets.codec.contains(&stream.codec_name)
    {
        command
            .arg(format!("-c:a:{}", stream.index,))
            .arg(&CONFIG.audio_targets.codec[0]);
    }

    if !CONFIG.audio_targets.sample_rate.is_empty() {
        let source_rate = stream.sample_rate;
        if !CONFIG.audio_targets.sample_rate.contains(&source_rate) {
            command
                .arg(format!("-ar:{}", stream.index,))
                .arg(&CONFIG.audio_targets.sample_rate[0].to_string());
        }
    }

    if !CONFIG.audio_targets.language.is_empty()
        && !stream.tags.language.is_empty()
        && !CONFIG
            .audio_targets
            .language
            .contains(&stream.tags.language)
    {
        command.arg("-map").arg(format!("-0:{}", stream.index));
    }

    if !*default_set
        && !CONFIG.audio_targets.default_language.is_empty()
        && stream.tags.language == CONFIG.audio_targets.default_language
    {
        command
            .arg(format!("-disposition:a:{}", stream.index))
            .arg("default");
        *default_set = true;
    } else if stream.disposition.default == 1 {
        command
            .arg(format!("-disposition:a:{}", stream.index))
            .arg("0");
    }
}

fn apply_subtitle_arguments(stream: &Stream, command: &mut Command, default_set: &mut bool) {
    if !CONFIG.subtitles.language.is_empty()
        && !stream.tags.language.is_empty()
        && !CONFIG.subtitles.language.contains(&stream.tags.language)
    {
        command.arg("-map").arg(format!("-0:s:{}?", stream.index));
    }

    if !*default_set
        && !CONFIG.subtitles.default_language.is_empty()
        && stream.tags.language == CONFIG.subtitles.default_language
    {
        command.arg("-disposition").arg("default");
        *default_set = true;
    } else if stream.disposition.default == 1 {
        command.arg(format!("-disposition:s:{}", stream.index));
    }
}

fn handle_completed_file(input_file: &MediaFile, output_file: &str) -> Result<(), String> {
    if CONFIG.streamline.always_replace {
        match std::fs::rename(output_file, input_file.path.as_path()) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    } else if CONFIG.streamline.replace_if_smaller {
        let input_size = input_file.path.metadata().unwrap().len();
        let output_size = std::fs::metadata(output_file).unwrap().len();
        if output_size < input_size {
            match std::fs::rename(output_file, input_file.path.as_path()) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
        } else {
            match std::fs::remove_file(output_file) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
        }
    } else {
        let desired_name = output_file
            .strip_suffix(&CONFIG.streamline.temporary_suffix)
            .unwrap();
        if Path::new(desired_name).exists() {
            return Err(format!(
                "File already exists and would be overwritten: {}",
                desired_name
            ));
        }
        match std::fs::rename(output_file, desired_name) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn process_file(input_file: &MediaFile) -> Result<(), String> {
    let output_file = get_output_path(&input_file.path);

    if Path::new(&output_file).exists() {
        std::fs::remove_file(&output_file).unwrap();
    }

    let mut command = Command::new(&CONFIG.ffmpeg.ffmpeg_path);
    let mut filters = Vec::new();

    command.arg("-i").arg(input_file.path.as_path());
    command.arg("-xerror");
    command.arg("-v").arg("error");
    command.arg("-f").arg(&CONFIG.streamline.output_format);

    if CONFIG.get_threads() != 0 {
        command
            .arg("-threads")
            .arg(&CONFIG.get_threads().to_string());
    }

    let video_streams = input_file.info.get_streams_of_type("video");
    let audio_streams = input_file.info.get_streams_of_type("audio");
    let subtitle_streams = input_file.info.get_streams_of_type("subtitle");

    if video_streams.is_empty() {
        return Err("No video streams found!".to_string());
    }
    apply_video_arguments(video_streams[0], &mut command);
    apply_video_filters(&mut filters);
    apply_aspect_ratio_corrections(video_streams[0], &mut filters);

    let default_audio = input_file.info.get_default_stream_of_type("audio");
    let mut audio_default_set = false;
    if default_audio.is_some()
        && !CONFIG.audio_targets.default_language.is_empty()
        && default_audio.unwrap().tags.language == CONFIG.audio_targets.default_language
    {
        audio_default_set = true;
    }

    for stream in audio_streams {
        apply_audio_arguments(stream, &mut command, &mut audio_default_set);
    }

    if !subtitle_streams.is_empty() {
        let mut subtitle_default_set = false;
        let default_subtitle = input_file.info.get_default_stream_of_type("subtitle");
        if default_subtitle.is_some()
            && !CONFIG.subtitles.default_language.is_empty()
            && default_subtitle.unwrap().tags.language == CONFIG.subtitles.default_language
        {
            subtitle_default_set = true;
        }
        for stream in subtitle_streams {
            apply_subtitle_arguments(stream, &mut command, &mut subtitle_default_set);
        }
    }

    let user_video_filters = CONFIG.video_targets.filters.clone();
    let user_audio_filters = CONFIG.audio_targets.filters.clone();

    if !user_video_filters.is_empty() {
        filters.push(
            user_video_filters
                .split(',')
                .collect::<Vec<&str>>()
                .join(","),
        );
    }
    if !user_audio_filters.is_empty() {
        filters.push(
            user_audio_filters
                .split(',')
                .collect::<Vec<&str>>()
                .join(","),
        );
    }

    if !filters.is_empty() {
        command.arg("-vf").arg(filters.join(","));
    }
    command.arg(&output_file);

    if CONFIG.streamline.dry_run {
        println!("Would run command: {:?}", command);
        Ok(())
    } else {
        match command.output() {
            Ok(cmd_out) => {
                handle_completed_file(input_file, &output_file)?;
                if !cmd_out.status.success() {
                    return Err(format!(
                        "Error running ffmpeg: {} -- {:?}",
                        String::from_utf8_lossy(&cmd_out.stderr),
                        command
                    ));
                }
                if CONFIG.streamline.debug {
                    println!(
                        "CMD: {:?}\nOutput: {}",
                        command,
                        String::from_utf8_lossy(&cmd_out.stdout)
                    );
                }

                Ok(())
            }
            Err(e) => Err(format!("Error running ffmpeg: {} -- {:?}", e, command)),
        }
    }
}
