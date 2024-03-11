use crate::models::media::FFProbeOutput;
use std::path::PathBuf;

pub struct MediaFile {
    pub path: PathBuf,
    pub info: FFProbeOutput,
}
