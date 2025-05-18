use crate::utils::load::load_image_and_resize;

pub fn extract_from_image(hide_file: &str) -> Vec<u8> {
    let hide_image = load_image_and_resize(hide_file).to_rgb8();
    let (hide_image_width, hide_image_height) = hide_image.dimensions();
    let hide_image_pixels = hide_image.into_raw();
    let mut hide_data = Vec::new();
    hide_data.extend_from_slice(&hide_image_width.to_be_bytes());
    hide_data.extend_from_slice(&hide_image_height.to_be_bytes());
    hide_data.extend_from_slice(&hide_image_pixels);
    hide_data
}
