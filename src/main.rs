mod content;
mod decode;
mod encode;
mod input;
mod traits;
mod utils;

use clap::Parser;
use content::audio::AudioContent;
use content::image::ImageContent;
use decode::common::reconstruct_hidden_content;
use decode::to_audio::lsb::decode_lsb_from_audio;
use decode::to_image::lsb::decode_lsb as decode_lsb_from_image;
use encode::to_audio::lsb::encode_lsb_to_audio;
use encode::to_image::lsb::encode_lsb as encode_lsb_to_image;

use input::read::{Args, CarrierType, Command};
use std::path::Path;
use traits::ContentType;
use utils::load::load_image;

fn main() -> Result<(), String> {
    let args = Args::parse();

    match args.command {
        Command::Encode {
            hide_file,
            content_type,
            steg_file,
            output_file,
            carrier_type,
        } => {
            let resolved_content_type = if content_type.to_lowercase() == "auto" {
                ContentType::from_path(&hide_file).ok_or_else(|| {
                    format!(
                        "Could not determine content type for HIDE_FILE '{}'. Please specify with -T",
                        hide_file
                    )
                })?
            } else {
                match content_type.to_lowercase().as_str() {
                    "image" => ContentType::Image,
                    "audio" => ContentType::Audio,
                    _ => {
                        return Err(format!(
                            "Unsupported HIDE_FILE content type: {}",
                            content_type
                        ))
                    }
                }
            };

            println!(
                "Preparing to hide {:?} data from '{}' into {} carrier '{}'",
                resolved_content_type, hide_file, carrier_type, steg_file
            );

            match resolved_content_type {
                ContentType::Image => {
                    let content_to_hide = ImageContent::new(&hide_file);
                    match carrier_type {
                        CarrierType::Image => {
                            encode_lsb_to_image(&content_to_hide, &steg_file, &output_file)?;
                        }
                        CarrierType::Audio => {
                            encode_lsb_to_audio(&content_to_hide, &steg_file, &output_file)?;
                        }
                    }
                }
                ContentType::Audio => {
                    let content_to_hide = AudioContent::new(&hide_file)?;
                    match carrier_type {
                        CarrierType::Image => {
                            encode_lsb_to_image(&content_to_hide, &steg_file, &output_file)?;
                        }
                        CarrierType::Audio => {
                            encode_lsb_to_audio(&content_to_hide, &steg_file, &output_file)?;
                        }
                    }
                }
            }
            println!(
                "Successfully hidden data from '{}' in '{}', output to '{}'",
                hide_file, steg_file, output_file
            );
        }
        Command::Decode {
            steg_file,
            output_file,
            carrier_type,
        } => {
            println!(
                "Analyzing {} steganography carrier '{}'",
                carrier_type, steg_file
            );

            let (data, hidden_content_type) = match carrier_type {
                CarrierType::Image => {
                    let steg_image_carrier = load_image(&steg_file)?;
                    decode_lsb_from_image(&steg_image_carrier)?
                }
                CarrierType::Audio => decode_lsb_from_audio(&steg_file)?,
            };

            reconstruct_hidden_content(&data, hidden_content_type, &output_file)?;
        }
    }
    Ok(())
}
