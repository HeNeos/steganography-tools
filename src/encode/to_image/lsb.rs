use crate::traits::Encodable;
use crate::utils::load::load_image;
use image::DynamicImage;

pub fn encode_lsb<T: Encodable>(
    hide_content: &T,
    steg_file: &str,
    output_file: &str,
) -> Result<(), String> {
    let content_type = hide_content.content_type();
    // let metadata = hide_content.metadata();
    let hide_data = hide_content.to_bytes();

    let hide_data_len = hide_data.len() as u32;
    let len_bytes = hide_data_len.to_be_bytes();

    let steg_image = load_image(steg_file);
    let mut steg_image_rgb = steg_image?.to_rgb8();
    let steg_image_bytes = steg_image_rgb.as_flat_samples_mut().samples;

    let total_bits_needed = (1 + 4 + hide_data.len()) * 8; // content_type + length + data
    if total_bits_needed > steg_image_bytes.len() {
        return Err(format!(
            "Image not large enough to hide {} bytes of data",
            hide_data.len()
        ));
    }

    let mut current_index = 0;

    let content_type_byte = content_type.to_u8();
    for bit_pos in 0..8 {
        let bit = (content_type_byte >> (7 - bit_pos)) & 1;
        steg_image_bytes[current_index] &= 0xFE; // Clear LSB
        steg_image_bytes[current_index] |= bit; // Set LSB
        current_index += 1;
    }

    for &byte in &len_bytes {
        for bit_pos in 0..8 {
            let bit = (byte >> (7 - bit_pos)) & 1;
            steg_image_bytes[current_index] &= 0xFE;
            steg_image_bytes[current_index] |= bit;
            current_index += 1;
        }
    }

    for &byte in &hide_data {
        for bit_pos in 0..8 {
            let bit = (byte >> (7 - bit_pos)) & 1;
            steg_image_bytes[current_index] &= 0xFE;
            steg_image_bytes[current_index] |= bit;
            current_index += 1;
        }
    }

    DynamicImage::ImageRgb8(steg_image_rgb)
        .save(output_file)
        .map_err(|e| format!("Failed to save output image: {}", e))?;

    Ok(())
}
