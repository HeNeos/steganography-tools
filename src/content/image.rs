use crate::traits::{ContentType, Decodable, Encodable};
use crate::utils::load::load_image_and_resize;
use image::{DynamicImage, RgbImage};

pub struct ImageContent {
    pub image: DynamicImage,
}

impl ImageContent {
    pub fn new(path: &str) -> Self {
        Self {
            image: load_image_and_resize(path),
        }
    }
}

impl Encodable for ImageContent {
    fn to_bytes(&self) -> Vec<u8> {
        let rgb = self.image.to_rgb8();
        let (width, height) = rgb.dimensions();
        let pixels = rgb.into_raw();

        let mut data = Vec::new();
        data.extend_from_slice(&width.to_be_bytes());
        data.extend_from_slice(&height.to_be_bytes());
        data.extend_from_slice(&pixels);
        data
    }

    fn content_type(&self) -> ContentType {
        ContentType::Image
    }

    fn metadata(&self) -> Vec<u8> {
        vec![self.content_type().to_u8()]
    }
}

impl Decodable for ImageContent {
    fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 8 {
            return Err("Not enough data for image dimensions".to_string());
        }

        let mut width_bytes = [0u8; 4];
        let mut height_bytes = [0u8; 4];

        width_bytes.copy_from_slice(&data[0..4]);
        height_bytes.copy_from_slice(&data[4..8]);

        let width = u32::from_be_bytes(width_bytes);
        let height = u32::from_be_bytes(height_bytes);

        let pixel_data = &data[8..];
        let rgb_image = RgbImage::from_raw(width, height, pixel_data.to_vec())
            .ok_or_else(|| "Failed to reconstruct image from data".to_string())?;

        Ok(Self {
            image: DynamicImage::ImageRgb8(rgb_image),
        })
    }
}
