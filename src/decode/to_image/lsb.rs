use image::DynamicImage;
use std::fs::File;
use std::io::Write;

pub fn decode_lsb(steg_image: &DynamicImage) -> Vec<u8> {
    let steg_rgb = steg_image.to_rgb8();
    let steg_flat_samples = steg_rgb.as_flat_samples();
    let steg_bytes = steg_flat_samples.samples;

    let mut current_steg_byte_index = 0;
    let bits_in_u8 = 8;

    let mut len_bytes_arr = [0u8; 4];
    for i in 0..4 {
        let mut current_byte_of_len = 0u8;
        for bit_pos in 0..bits_in_u8 {
            let lsb = steg_bytes[current_steg_byte_index] & 1;
            current_byte_of_len |= lsb << (7 - bit_pos);
            current_steg_byte_index += 1;
        }
        len_bytes_arr[i] = current_byte_of_len;
    }
    let secret_data_len = u32::from_be_bytes(len_bytes_arr) as usize;

    let mut secret_data = Vec::with_capacity(secret_data_len);
    for _ in 0..secret_data_len {
        let mut current_secret_byte = 0u8;
        for bit_pos in 0..bits_in_u8 {
            let lsb = steg_bytes[current_steg_byte_index] & 1;
            current_secret_byte |= lsb << (7 - bit_pos);
            current_steg_byte_index += 1;
        }
        secret_data.push(current_secret_byte);
    }

    secret_data
}

pub fn reconstruct_image(data: &[u8], output_file: &str) {
    let mut width_bytes = [0u8; 4];
    let mut height_bytes = [0u8; 4];

    width_bytes.copy_from_slice(&(data[0..4]));
    height_bytes.copy_from_slice(&(data[4..8]));

    let width = u32::from_be_bytes(width_bytes);
    let height = u32::from_be_bytes(height_bytes);

    let pixels = &data[8..];

    match image::RgbImage::from_raw(width, height, pixels.to_vec()) {
        Some(img) => match DynamicImage::ImageRgb8(img).save(output_file) {
            Ok(_) => {
                println!("Image extracted successfully: '{}'", output_file)
            }
            Err(e) => {
                eprintln!("Error saving image to '{}': {}", output_file, e)
            }
        },
        None => {
            eprintln!("Could not construct image, saving raw data instead");
            match File::create(output_file) {
                Ok(mut file) => match file.write_all(data) {
                    Ok(_) => println!("Raw data saved to: '{}'", output_file),
                    Err(e) => {
                        eprintln!("Error writing data to '{}': {}", output_file, e)
                    }
                },
                Err(e) => {
                    eprintln!("Error creating output file '{}': {}", output_file, e)
                }
            }
        }
    }
}
