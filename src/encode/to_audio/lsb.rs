use crate::traits::Encodable;
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use tempfile::NamedTempFile; // For managing the temporary WAV file

const BITS_PER_SAMPLE_TO_HIDE: u32 = 1; // Hiding 1 bit per sample

pub fn encode_lsb_to_audio<T: Encodable>(
    hide_content: &T,
    carrier_audio_path_str: &str,
    output_audio_path_str: &str,
) -> Result<(), String> {
    let initial_carrier_path = Path::new(carrier_audio_path_str);
    let final_output_path = Path::new(output_audio_path_str);

    if final_output_path.extension().and_then(|s| s.to_str()) != Some("wav") {
        return Err(format!(
            "Output audio path '{}' must have a .wav extension. LSB steganography on audio produces a WAV file.",
            output_audio_path_str
        ));
    }

    let content_type = hide_content.content_type();
    let hide_data_bytes = hide_content.to_bytes();
    let hide_data_len = hide_data_bytes.len() as u32;

    let mut full_data_to_embed = Vec::new();
    full_data_to_embed.push(content_type.to_u8());
    full_data_to_embed.extend_from_slice(&hide_data_len.to_be_bytes());
    full_data_to_embed.extend_from_slice(&hide_data_bytes);

    let total_bits_to_hide = full_data_to_embed.len() * 8;

    let wav_path_for_processing: PathBuf;
    let mut _temp_wav_holder: Option<NamedTempFile> = None;

    if initial_carrier_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map_or(false, |ext| ext.eq_ignore_ascii_case("wav"))
    {
        wav_path_for_processing = initial_carrier_path.to_path_buf();
        println!(
            "Using existing WAV file as carrier: {}",
            carrier_audio_path_str
        );
    } else {
        println!(
            "Carrier '{}' is not a WAV file. Converting to a temporary WAV for LSB encoding...",
            carrier_audio_path_str
        );

        let temp_wav = NamedTempFile::new()
            .map_err(|e| format!("Failed to create temporary file for WAV conversion: {}", e))?;
        let temp_wav_path = temp_wav.path().to_path_buf();

        let ffmpeg_status = StdCommand::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(initial_carrier_path)
            .arg("-vn")
            .arg("-acodec")
            .arg("pcm_s16le")
            .arg(&temp_wav_path)
            .status()
            .map_err(|e| {
                format!(
                    "Failed to execute ffmpeg. Is it installed and in PATH? Error: {}",
                    e
                )
            })?;

        if !ffmpeg_status.success() {
            let ffmpeg_output = StdCommand::new("ffmpeg")
                .arg("-y")
                .arg("-i")
                .arg(initial_carrier_path)
                .arg("-vn")
                .arg("-acodec")
                .arg("pcm_s16le")
                .arg(&temp_wav_path)
                .output()
                .unwrap();

            return Err(format!(
                "ffmpeg failed to convert '{}' to WAV (exit code: {}). Stderr: {}",
                carrier_audio_path_str,
                ffmpeg_status.code().unwrap_or(-1),
                String::from_utf8_lossy(&ffmpeg_output.stderr)
            ));
        }
        println!(
            "Successfully converted carrier to temporary WAV: {:?}",
            temp_wav_path
        );
        wav_path_for_processing = temp_wav_path;
        _temp_wav_holder = Some(temp_wav);
    }

    let mut reader = WavReader::open(&wav_path_for_processing).map_err(|e| {
        format!(
            "Failed to open carrier WAV file '{}' for LSB processing: {}",
            wav_path_for_processing.display(),
            e
        )
    })?;
    let spec = reader.spec();

    if spec.sample_format != SampleFormat::Int || spec.bits_per_sample != 16 {
        return Err(format!(
            "The processed carrier audio ('{}') must be a 16-bit integer PCM WAV file. Detected format: {:?}",
            wav_path_for_processing.display(), spec
        ));
    }

    let samples: Vec<i16> = reader
        .samples::<i16>()
        .map(|s| s.expect("Failed to read sample from processed WAV"))
        .collect();

    let available_bits_for_hiding = samples.len() as u32 * BITS_PER_SAMPLE_TO_HIDE;

    if (total_bits_to_hide as u32) > available_bits_for_hiding {
        return Err(format!(
            "Not enough space in carrier audio ('{}'). Needed {} bits, available {} bits.",
            wav_path_for_processing.display(),
            total_bits_to_hide,
            available_bits_for_hiding
        ));
    }

    let mut modified_samples = samples.clone();
    let mut data_bit_index = 0;

    for sample_index in 0..modified_samples.len() {
        if data_bit_index >= total_bits_to_hide {
            break;
        }

        let byte_to_embed_index = data_bit_index / 8;
        let bit_in_byte_pos = 7 - (data_bit_index % 8);

        let bit_to_embed = (full_data_to_embed[byte_to_embed_index] >> bit_in_byte_pos) & 1;

        modified_samples[sample_index] &= !1; // ...xxxxxxx0
                                              // Set LSB of the sample
        modified_samples[sample_index] |= bit_to_embed as i16;

        data_bit_index += 1;
    }

    // Write the LSB-modified samples to the final output WAV file
    let mut writer = WavWriter::create(final_output_path, spec).map_err(|e| {
        format!(
            "Failed to create output WAV file '{}': {}",
            output_audio_path_str, e
        )
    })?;

    for sample in modified_samples {
        writer
            .write_sample(sample)
            .map_err(|e| format!("Failed to write sample to output WAV: {}", e))?;
    }

    writer.finalize().map_err(|e| {
        format!(
            "Failed to finalize output WAV '{}': {}",
            output_audio_path_str, e
        )
    })?;

    // If _temp_wav_holder contains a NamedTempFile, it will be dropped here,
    // and the temporary file will be automatically deleted.

    println!(
        "Successfully hid data from input carrier '{}', output to WAV file '{}'",
        carrier_audio_path_str, output_audio_path_str
    );
    Ok(())
}
