use image::{imageops::FilterType, DynamicImage, GenericImageView};

const MAX_HEIGHT: u32 = 720;

pub fn load_image(path: &str) -> Result<DynamicImage, String> {
    image::open(path).map_err(|e| format!("Failed to open image '{}': {}", path, e))
}

pub fn load_image_and_resize(path: &str) -> DynamicImage {
    let img = load_image(path).expect("Failed to open image");
    let (width, height) = img.dimensions();

    let (new_width, new_height) = if height > MAX_HEIGHT {
        (MAX_HEIGHT * width / height, MAX_HEIGHT)
    } else {
        (width, height)
    };

    if new_width != width || new_height != height {
        img.resize(new_width, new_height, FilterType::Lanczos3)
    } else {
        img
    }
}
