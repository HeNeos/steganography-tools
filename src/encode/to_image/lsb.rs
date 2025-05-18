use crate::utils::load::{load_image, load_image_and_resize};
use image::DynamicImage;

pub fn encode_lsb(hide_file: &str, steg_file: &str, output_file: &str) {
    let hide_image = load_image_and_resize(hide_file).to_rgb8();
    let (hide_image_width, hide_image_height) = hide_image.dimensions();
    let hide_image_pixels = hide_image.into_raw();
    let mut hide_data = Vec::new();
    hide_data.extend_from_slice(&hide_image_width.to_be_bytes());
    hide_data.extend_from_slice(&hide_image_height.to_be_bytes());
    hide_data.extend_from_slice(&hide_image_pixels);

    let hide_data_len = hide_data.len();
    let len_bytes = (hide_data_len as u32).to_be_bytes();
    // let total_bits_to_hide = (4 + hide_data_len) * 8;

    let steg_image: DynamicImage = load_image(steg_file);
    let mut steg_image_rgb = steg_image.to_rgb8();
    let steg_image_bytes = steg_image_rgb.as_flat_samples_mut().samples;

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

    DynamicImage::ImageRgb8(steg_image_rgb).save(output_file);
}
