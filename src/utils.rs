use std::fmt::Write;
use std::time::Duration;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

pub fn create_spinner(steady_tick: bool) -> ProgressBar {
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(spinner_style);
    if steady_tick{
        spinner.enable_steady_tick(Duration::from_millis(100));
    }
    spinner
}

pub fn create_progress_bar(length: u64) -> ProgressBar {
    let progress = ProgressBar::new(length);
    progress.set_style(ProgressStyle::with_template("{spinner} [{elapsed_precise}] [{bar:.cyan/blue}] {pos}/{len} ({eta}) {msg}")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()));
    progress
}

pub fn set_pb_finish_message(pb: &ProgressBar, message: String) {
    pb.set_style(ProgressStyle::default_bar().template("   {wide_msg}").unwrap());
    pb.finish_with_message(message);
}
