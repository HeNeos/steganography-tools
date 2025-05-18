use crate::content::audio::AudioContent;
use crate::content::image::ImageContent;
use crate::traits::{ContentType, Decodable};
use image::DynamicImage;
use std::fs as std_fs;
use std::path::Path as StdPath;

pub fn decode_lsb(steg_image: &DynamicImage) -> Result<(Vec<u8>, ContentType), String> {
    let steg_rgb = steg_image.to_rgb8();
    let steg_bytes = steg_rgb.as_flat_samples().samples;

    if steg_bytes.len() < 40 {
        return Err("Image too small to contain hidden data".to_string());
    }

    let mut current_index = 0;

    let mut content_type_byte = 0u8;
    for bit_pos in 0..8 {
        if current_index >= steg_bytes.len() {
            return Err("Corrupted data: unexpected end while reading content type".to_string());
        }
        let bit = steg_bytes[current_index] & 1;
        content_type_byte |= bit << (7 - bit_pos);
        current_index += 1;
    }

    let content_type = ContentType::from_u8(content_type_byte)
        .ok_or_else(|| format!("Invalid content type byte: {}", content_type_byte))?;
    println!("Detected content type: {:?}", content_type);

    let mut len_bytes = [0u8; 4];
    for i in 0..4 {
        for bit_pos in 0..8 {
            if current_index >= steg_bytes.len() {
                return Err("Corrupted data: unexpected end while reading data length".to_string());
            }
            let bit = steg_bytes[current_index] & 1;
            len_bytes[i] |= bit << (7 - bit_pos);
            current_index += 1;
        }
    }
    let data_len = u32::from_be_bytes(len_bytes) as usize;
    println!("Detected data length: {} bytes", data_len);

    if (current_index / 8 + data_len) > steg_bytes.len() / 8 + 1 {
        if current_index + data_len * 8 > steg_bytes.len() {
            return Err(format!(
                "Corrupted data: claimed data length {} ({} bits) exceeds available image data ({} bits remaining from current_index)",
                data_len, data_len * 8, steg_bytes.len() - current_index
            ));
        }
    }

    let mut data = Vec::with_capacity(data_len);
    for _ in 0..data_len {
        let mut byte = 0u8;
        for bit_pos in 0..8 {
            if current_index >= steg_bytes.len() {
                return Err("Corrupted data: unexpected end while reading hidden data".to_string());
            }
            let bit = steg_bytes[current_index] & 1;
            byte |= bit << (7 - bit_pos);
            current_index += 1;
        }
        data.push(byte);
    }

    Ok((data, content_type))
}
