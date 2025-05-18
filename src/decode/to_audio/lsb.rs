use crate::traits::ContentType;
use hound::{SampleFormat, WavReader};

const BITS_PER_SAMPLE_TO_EXTRACT: u32 = 1;

pub fn decode_lsb_from_audio(steg_audio_path: &str) -> Result<(Vec<u8>, ContentType), String> {
    let mut reader = WavReader::open(steg_audio_path).map_err(|e| {
        format!(
            "Failed to open steganographic WAV file '{}': {}",
            steg_audio_path, e
        )
    })?;

    if reader.spec().sample_format != SampleFormat::Int || reader.spec().bits_per_sample != 16 {
        return Err("Steganographic audio must be a 16-bit integer PCM WAV file.".to_string());
    }

    let samples: Vec<i16> = reader
        .samples::<i16>()
        .map(|s| s.expect("Failed to read sample from steganographic audio"))
        .collect();

    if samples.len() < (1 + 4) * 8 {
        return Err("Audio file too short to contain metadata.".to_string());
    }

    let mut extracted_bits = Vec::new();
    for sample_index in 0..samples.len() {
        let lsb = samples[sample_index] & 1;
        extracted_bits.push(lsb as u8);
    }

    if extracted_bits.len() < (1 + 4) * 8 {
        return Err("Not enough bits extracted for metadata.".to_string());
    }

    let mut content_type_byte = 0u8;
    for i in 0..8 {
        content_type_byte |= extracted_bits[i] << (7 - i);
    }
    let content_type = ContentType::from_u8(content_type_byte)
        .ok_or_else(|| format!("Invalid content type byte extracted: {}", content_type_byte))?;
    println!("Detected content type from audio: {:?}", content_type);

    let mut len_bytes = [0u8; 4];
    let mut bit_offset = 8;
    for i in 0..4 {
        for j in 0..8 {
            if bit_offset >= extracted_bits.len() {
                return Err(
                    "Corrupted data: unexpected end while reading data length from audio."
                        .to_string(),
                );
            }
            len_bytes[i] |= extracted_bits[bit_offset] << (7 - j);
            bit_offset += 1;
        }
    }
    let data_len = u32::from_be_bytes(len_bytes) as usize;
    println!("Detected data length from audio: {} bytes", data_len);

    let total_header_bits = (1 + 4) * 8;
    let required_total_bits = total_header_bits + data_len * 8;

    if required_total_bits > extracted_bits.len() {
        return Err(format!(
            "Corrupted data: claimed data length {} ({} bits) plus header ({} bits) exceeds available extracted bits ({} bits).",
            data_len, data_len * 8, total_header_bits, extracted_bits.len()
        ));
    }

    let mut hidden_data = Vec::with_capacity(data_len);
    for i in 0..data_len {
        let mut byte = 0u8;
        for j in 0..8 {
            if bit_offset >= extracted_bits.len() {
                return Err(
                    "Corrupted data: unexpected end while reading hidden data payload from audio."
                        .to_string(),
                );
            }
            byte |= extracted_bits[bit_offset] << (7 - j);
            bit_offset += 1;
        }
        hidden_data.push(byte);
    }

    Ok((hidden_data, content_type))
}
