use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct FFProbeOutput {
    pub streams: Vec<Stream>,
    pub format: Format,
}

#[derive(Deserialize, Debug, Default)]
pub struct Stream {
    pub index: u32,
    #[serde(default="String::new")]
    pub codec_name: String,
    #[serde(default="String::new")]
    pub codec_long_name: String,
    pub profile: Option<String>,
    pub codec_type: String,
    pub codec_tag_string: String,
    pub codec_tag: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub coded_width: Option<u32>,
    pub coded_height: Option<u32>,
    pub closed_captions: Option<u32>,
    pub film_grain: Option<u32>,
    pub has_b_frames: Option<u32>,
    pub sample_aspect_ratio: Option<String>,
    pub display_aspect_ratio: Option<String>,
    pub pix_fmt: Option<String>,
    pub level: Option<i32>,
    pub color_range: Option<String>,
    pub color_space: Option<String>,
    pub color_transfer: Option<String>,
    pub color_primaries: Option<String>,
    pub chroma_location: Option<String>,
    pub field_order: Option<String>,
    pub refs: Option<u32>,
    pub is_avc: Option<String>,
    pub nal_length_size: Option<String>,
    pub sample_fmt: Option<String>,
    pub sample_rate: Option<String>,
    pub channels: Option<u32>,
    pub channel_layout: Option<String>,
    pub bits_per_sample: Option<u32>,
    pub initial_padding: Option<u32>,
    #[serde(default="String::new")]
    pub id: String,
    pub r_frame_rate: String,
    pub avg_frame_rate: String,
    pub time_base: String,
    #[serde(default="i64::default")]
    pub start_pts: i64,
    #[serde(default="String::new")]
    pub start_time: String,
    #[serde(default="u64::default")]
    pub duration_ts: u64,
    #[serde(default="String::new")]
    pub duration: String,
    #[serde(default="String::new")]
    pub bit_rate: String,
    pub bits_per_raw_sample: Option<String>,
    pub nb_frames: Option<String>,
    #[serde(default="u32::default")]
    pub extradata_size: u32,
    pub disposition: Disposition,
    #[serde(default="Tags::default")]
    pub tags: Tags,
}

#[derive(Deserialize, Debug, Default)]
pub struct Disposition {
    pub default: u32,
    pub dub: u32,
    pub original: u32,
    pub comment: u32,
    pub lyrics: u32,
    pub karaoke: u32,
    pub forced: u32,
    pub hearing_impaired: u32,
    pub visual_impaired: u32,
    pub clean_effects: u32,
    pub attached_pic: u32,
    pub timed_thumbnails: u32,
    pub non_diegetic: u32,
    pub captions: u32,
    pub descriptions: u32,
    pub metadata: u32,
    pub dependent: u32,
    pub still_image: u32,
}

#[derive(Deserialize, Debug, Default)]
pub struct Tags {
    #[serde(default="String::new")]
    pub language: String,
    #[serde(default="String::new")]
    pub handler_name: String,
    #[serde(default="String::new")]
    pub vendor_id: String,
    pub encoder: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Format {
    pub filename: String,
    pub nb_streams: u32,
    pub nb_programs: u32,
    pub format_name: String,
    pub format_long_name: String,
    #[serde(default="String::new")]
    pub start_time: String,
    #[serde(default="String::new")]
    pub duration: String,
    pub size: String,
    #[serde(default="String::new")]
    pub bit_rate: String,
    pub probe_score: u32,
    #[serde(default="FormatTags::default")]
    pub tags: FormatTags,
}

#[derive(Deserialize, Debug, Default)]
pub struct FormatTags {
    #[serde(default="String::new")]
    pub major_brand: String,
    #[serde(default="String::new")]
    pub minor_version: String,
    #[serde(default="String::new")]
    pub compatible_brands: String,
    #[serde(default="String::new")]
    pub encoder: String,
}