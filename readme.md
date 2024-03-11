# Streamline
### WIP project, here be dragons

Streamline is a Rust-based command-line tool designed to process and optimise media libraries.
It levarges the capabilites of the ffmpeg suite to analyze, filter, and transcode your media to ensure they meet
your specified requirements.

# Installation

Ensure you have Rust installed on your system. If not, you can download and install Rust from the official website: https://www.rust-lang.org/
Clone the Streamline repository:

git clone https://github.com/your-username/streamline.git

Navigate to the project directory:

`cd streamline`

Build the project:

`cargo build --release`

The compiled binary will be available in the target/release directory.

# Configuration

Streamline uses a configuration file called `config.toml` to specify various settings and preferences.
You can find a sample configuration file in the project directory. Modify this file to suit your needs.

The configuration file allows you to specifyL
- Source and output directories
- File extensions to process
- Video and audio codecs to use
- Subtitle and audio track preferences
- Filters to apply to your media
- And more

# Usage

To use StreamLine, simply run the compiled binary in the same directory as your `config.toml` file.

Streamline will scan the specified source directory for media files, process them according to your configuration,
and output the results to the specified output directory.

**Please do note**, you are encouraged to run Streamline on a subset of your media library to ensure your configuration is correct.
Do not run Streamline on your entire media library without doing basic due diligence.
**I am not responsible for any data loss or corruption that may occur as a result of using this tool.**

# Dependencies

Streamline is dependent on the following software:
- ffmpeg
- ffprobe

Please ensure these are installed on your system and you have set the configuration file to point to the correct locations.

# Contributing

Contributions to Streamline are welcome! If you find any issues or have suggestions for improvements, please open an issue
or submit a pull request on the GitHub repository.
