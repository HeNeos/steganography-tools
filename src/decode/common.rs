use crate::content::audio::AudioContent;
use crate::content::image::ImageContent;
use crate::traits::{ContentType, Decodable};
use std::fs as std_fs;
use std::path::Path as StdPath;

pub fn reconstruct_hidden_content(
    data: &[u8],
    content_type: ContentType,
    output_file_base: &str,
) -> Result<(), String> {
    match content_type {
        ContentType::Image => {
            let image_content = ImageContent::from_bytes(data)?;
            let output_path = StdPath::new(output_file_base);
            // Ensure correct extension for reconstructed image
            let final_output_path =
                if output_path.extension().and_then(|s| s.to_str()) != Some("png") {
                    format!("{}.png", output_file_base)
                } else {
                    output_file_base.to_string()
                };

            image_content
                .image
                .save(&final_output_path)
                .map_err(|e| format!("Failed to save extracted image: {}", e))?;
            println!("Image successfully extracted to '{}'", final_output_path);
        }
        ContentType::Audio => {
            let audio_content = AudioContent::from_bytes(data)?;
            // Use the extension stored within the audio data
            let final_output_path =
                format!("{}.{}", output_file_base, audio_content.target_extension);
            std_fs::write(&final_output_path, &audio_content.compressed_audio_bytes)
                .map_err(|e| format!("Failed to save extracted audio: {}", e))?;
            println!("Audio successfully extracted to '{}'", final_output_path);
        }
    }
    Ok(())
}
