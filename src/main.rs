mod input;
mod utils;

use clap::Parser;
use image::{DynamicImage, GenericImageView};
use input::read::{Args, Command};
use std::fs::{self, File};
use std::io::Write;
use utils::load::{load_image, load_image_and_resize};

fn decode_lsb(steg_image: &DynamicImage) -> Result<Vec<u8>, String> {
    let steg_rgba = steg_image.to_rgba8();
    let steg_flat_samples = steg_rgba.as_flat_samples();
    let steg_bytes = steg_flat_samples.samples;

    let mut current_steg_byte_index = 0;
    let bits_in_u8 = 8;

    if steg_bytes.len() < 4 * bits_in_u8 {
        return Err("Steg image too small to contain length information.".to_string());
    }

    let mut len_bytes_arr = [0u8; 4];
    for i in 0..4 {
        let mut current_byte_of_len = 0u8;
        for bit_pos in 0..bits_in_u8 {
            if current_steg_byte_index >= steg_bytes.len() {
                return Err("Ran out of steg image bytes while reading length.".to_string());
            }
            let lsb = steg_bytes[current_steg_byte_index] & 1;
            current_byte_of_len |= lsb << (7 - bit_pos);
            current_steg_byte_index += 1;
        }
        len_bytes_arr[i] = current_byte_of_len;
    }
    let secret_data_len = u32::from_be_bytes(len_bytes_arr) as usize;

    let required_lsbs_for_data = secret_data_len * bits_in_u8;
    let remaining_lsbs_in_steg = steg_bytes.len() - current_steg_byte_index;

    if required_lsbs_for_data > remaining_lsbs_in_steg {
        return Err(format!(
            "Steg image does not contain enough data. Declared secret data length is {} bytes (requires {} LSBs), but only {} LSBs remain after length header.",
            secret_data_len,
            required_lsbs_for_data,
            remaining_lsbs_in_steg
        ));
    }
    if secret_data_len == 0 {
        println!("Warning: Extracted data length is 0. Output file will be empty.");
        return Ok(Vec::new());
    }

    let mut secret_data = Vec::with_capacity(secret_data_len);
    for _ in 0..secret_data_len {
        let mut current_secret_byte = 0u8;
        for bit_pos in 0..bits_in_u8 {
            if current_steg_byte_index >= steg_bytes.len() {
                return Err(format!(
                    "Ran out of steg image bytes unexpectedly while reading secret data. Expected {} bytes, read up to byte index {}.",
                    secret_data_len,
                    secret_data.len()
                ));
            }
            let lsb = steg_bytes[current_steg_byte_index] & 1;
            current_secret_byte |= lsb << (7 - bit_pos);
            current_steg_byte_index += 1;
        }
        secret_data.push(current_secret_byte);
    }

    Ok(secret_data)
}

fn main() {
    let args = Args::parse();
    match args.command {
        Command::Encode {
            hide_file,
            steg_file,
            output_file,
        } => {
            let hide_image = load_image_and_resize(&hide_file).to_rgba8();
            let (hide_image_width, hide_image_height) = hide_image.dimensions();
            let hide_image_pixels = hide_image.into_raw();
            let mut hide_data = Vec::new();
            hide_data.extend_from_slice(&hide_image_width.to_be_bytes());
            hide_data.extend_from_slice(&hide_image_height.to_be_bytes());
            hide_data.extend_from_slice(&hide_image_pixels);

            let steg_image: DynamicImage = load_image(&steg_file);
            let mut steg_image_rgba = steg_image.to_rgba8();
            let steg_image_bytes = steg_image_rgba.as_flat_samples_mut().samples;

            let hide_data_len = hide_data.len();
            let len_bytes = (hide_data_len as u32).to_be_bytes();
            let total_bits_to_hide = (4 + hide_data_len) * 8;

            let mut current_steg_byte_index = 0;
            for byte in len_bytes.iter() {
                for bit_index in 0..8 {
                    let bit_to_hide = (byte >> (7 - bit_index)) & 1;
                    // Clear last bit in steg byte
                    steg_image_bytes[current_steg_byte_index] &= 0xFE;
                    // Set last bit in steg byte
                    steg_image_bytes[current_steg_byte_index] |= bit_to_hide;
                    current_steg_byte_index += 1;
                }
            }

            for byte in hide_data.iter() {
                for bit_index in 0..8 {
                    let bit_to_hide = (byte >> (7 - bit_index)) & 1;
                    steg_image_bytes[current_steg_byte_index] &= 0xFE;
                    steg_image_bytes[current_steg_byte_index] |= bit_to_hide;
                    current_steg_byte_index += 1;
                }
            }

            DynamicImage::ImageRgba8(steg_image_rgba).save(&output_file);
        }
        Command::Decode {
            steg_file,
            output_file,
        } => {
            let steg_image = load_image(&steg_file);
            match decode_lsb(&steg_image) {
                Ok(extracted_data) => match File::create(&output_file) {
                    Ok(mut file) => match file.write_all(&extracted_data) {
                        Ok(_) => println!("Data extracted successfully: '{}'", output_file),
                        Err(e) => {
                            eprintln!("Error writing extracted data to '{}': {}", output_file, e)
                        }
                    },
                    Err(e) => eprintln!("Error creating output file '{}': {}", output_file, e),
                },
                Err(e) => eprintln!("Error decoding image: {}", e),
            }
        }
    }
}
