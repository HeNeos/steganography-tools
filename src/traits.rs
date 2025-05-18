pub trait Encodable {
    fn to_bytes(&self) -> Vec<u8>;
    fn content_type(&self) -> ContentType;
    fn metadata(&self) -> Vec<u8>;
}

pub trait Decodable: Sized {
    fn from_bytes(data: &[u8]) -> Result<Self, String>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentType {
    Image = 0,
    Audio = 1,
}

impl ContentType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(ContentType::Image),
            1 => Some(ContentType::Audio),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            ContentType::Image => "image",
            ContentType::Audio => "audio",
        }
    }

    pub fn file_extension(&self) -> &'static str {
        match self {
            ContentType::Image => "png",
            ContentType::Audio => "mp3",
        }
    }

    pub fn from_path(path: &str) -> Option<Self> {
        let path = path.to_lowercase();
        if path.ends_with(".jpg") || path.ends_with(".jpeg") || path.ends_with(".png") {
            Some(ContentType::Image)
        } else if path.ends_with(".wav")
            || path.ends_with(".mp3")
            || path.ends_with(".flac")
            || path.ends_with(".ogg")
        {
            Some(ContentType::Audio)
        } else {
            None
        }
    }
}
