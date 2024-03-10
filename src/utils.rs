use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use serde::{Deserialize, Deserializer};
use std::fmt::Write;
use std::time::Duration;

pub fn create_spinner(steady_tick: bool) -> ProgressBar {
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(spinner_style);
    if steady_tick {
        spinner.enable_steady_tick(Duration::from_millis(100));
    }
    spinner
}

pub fn create_progress_bar(length: u64, steady_tick: bool, tick_interval: u64) -> ProgressBar {
    let progress = ProgressBar::new(length);
    progress.set_style(
        ProgressStyle::with_template(
            "{spinner} [{elapsed_precise}] [{bar:.cyan/blue}] {pos}/{len} ({eta}) {msg}",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
    );
    if steady_tick {
        progress.enable_steady_tick(Duration::from_millis(tick_interval));
    }
    progress
}

pub fn set_pb_finish_message(pb: &ProgressBar, message: String) {
    pb.set_style(
        ProgressStyle::default_bar()
            .template("   {wide_msg}")
            .unwrap(),
    );
    pb.finish_with_message(message);
}

pub fn parse_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    u32::from_str_radix(&s, 10).map_err(serde::de::Error::custom)
}

// pub fn parse_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
//     where
//         D: Deserializer<'de>,
// {
//     let s = String::deserialize(deserializer)?;
//     u64::from_str_radix(&s, 10).map_err(serde::de::Error::custom)
// }

pub fn parse_frame_rate<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let parts: Vec<&str> = s.split('/').collect();
    let numerator = parts[0].parse::<f64>().unwrap();
    let denominator = parts[1].parse::<f64>().unwrap();
    Ok(numerator / denominator)
}

pub fn get_system_path_separator() -> String {
    if cfg!(windows) {
        "\\".to_string()
    } else {
        "/".to_string()
    }
}
