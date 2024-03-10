use crate::models::media::FFProbeOutput;
use std::fs::DirEntry;

pub struct MediaFile {
    pub path: DirEntry,
    pub info: FFProbeOutput,
}
