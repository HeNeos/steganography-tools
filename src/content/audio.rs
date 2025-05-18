use crate::traits::{ContentType, Decodable, Encodable};
use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::NamedTempFile;

const TARGET_AUDIO_BITRATE: &str = "64k";
const TARGET_AUDIO_FORMAT_CODEC: &str = "libmp3lame";
const TARGET_AUDIO_EXTENSION: &str = "mp3";

pub struct AudioContent {
    pub compressed_audio_bytes: Vec<u8>,
    pub target_extension: String,
}

impl AudioContent {
    pub fn new(path: &str) -> Result<Self, String> {
        let input_path = Path::new(path);
        if !input_path.exists() {
            return Err(format!("Input audio file not found: {}", path));
        }

        let temp_output_file =
            NamedTempFile::new().map_err(|e| format!("Failed to create temp file: {}", e))?;

        let temp_output_path_with_ext = temp_output_file
            .path()
            .with_extension(TARGET_AUDIO_EXTENSION);
        let temp_output_path_str = temp_output_path_with_ext
            .to_str()
            .ok_or("Invalid temporary file path string")?;

        println!(
            "Compressing audio '{}' to {} at {} using ffmpeg...",
            path, TARGET_AUDIO_EXTENSION, TARGET_AUDIO_BITRATE
        );

        let ffmpeg_output = StdCommand::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(path)
            .arg("-c:a")
            .arg(TARGET_AUDIO_FORMAT_CODEC)
            .arg("-b:a")
            .arg(TARGET_AUDIO_BITRATE)
            .arg(temp_output_path_str)
            .output()
            .map_err(|e| {
                format!(
                    "Failed to execute ffmpeg. Is it installed and in PATH? Error: {}",
                    e
                )
            })?;

        if !ffmpeg_output.status.success() {
            let stderr = String::from_utf8_lossy(&ffmpeg_output.stderr);
            return Err(format!(
                "ffmpeg failed to compress audio (exit code: {:?}). Stderr:\n{}",
                ffmpeg_output.status.code(),
                stderr
            ));
        }

        let compressed_audio_bytes = fs::read(temp_output_path_str)
            .map_err(|e| format!("Failed to read compressed audio from temp file: {}", e))?;

        Ok(Self {
            compressed_audio_bytes,
            target_extension: TARGET_AUDIO_EXTENSION.to_string(),
        })
    }
}

impl Encodable for AudioContent {
    fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Serialize target_extension: length (u32) + string bytes
        let ext_bytes = self.target_extension.as_bytes();
        let ext_len = ext_bytes.len() as u32;
        data.extend_from_slice(&ext_len.to_be_bytes());
        data.extend_from_slice(ext_bytes);

        // Append compressed audio data
        data.extend_from_slice(&self.compressed_audio_bytes);

        data
    }

    fn content_type(&self) -> ContentType {
        ContentType::Audio
    }

    fn metadata(&self) -> Vec<u8> {
        // This metadata (content type ID) is written separately by the LSB encoder.
        vec![self.content_type().to_u8()]
    }
}

impl Decodable for AudioContent {
    fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 4 {
            // Minimum for extension length
            return Err("Not enough data for audio content (extension length)".to_string());
        }

        let mut ext_len_bytes = [0u8; 4];
        ext_len_bytes.copy_from_slice(&data[0..4]);
        let ext_len = u32::from_be_bytes(ext_len_bytes) as usize;

        let header_end_offset = 4 + ext_len;
        if data.len() < header_end_offset {
            return Err(format!(
                "Not enough data for audio content (extension string). Expected {} bytes for ext, data has {}.",
                ext_len, data.len() - 4
            ));
        }

        let target_extension = String::from_utf8(data[4..header_end_offset].to_vec())
            .map_err(|e| format!("Failed to decode audio target extension: {}", e))?;

        let compressed_audio_bytes = data[header_end_offset..].to_vec();

        Ok(Self {
            compressed_audio_bytes,
            target_extension,
        })
    }
}
