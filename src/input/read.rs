use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CarrierType {
    Image,
    Audio,
}

impl std::fmt::Display for CarrierType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CarrierType::Image => write!(f, "image"),
            CarrierType::Audio => write!(f, "audio"),
        }
    }
}

#[derive(Parser, Debug)]
pub enum Command {
    Encode {
        /// The file to hide (image or audio)
        #[clap(short = 'H', long, value_parser)]
        hide_file: String,

        /// The type of content to hide (image, audio, or auto for hide_file)
        #[clap(short = 'T', long, value_parser, default_value = "auto")]
        content_type: String,

        /// The carrier file (e.g., PNG for image, WAV for audio)
        #[clap(short = 'S', long, value_parser)]
        steg_file: String,

        /// The output path for the steganography file
        #[clap(short = 'O', long, value_parser)]
        output_file: String,

        /// Type of the carrier file (steg_file)
        #[clap(short = 'C', long, value_enum)]
        carrier_type: CarrierType,
    },
    Decode {
        /// The steganography carrier file containing the hidden data
        #[clap(short = 'S', long, value_parser)]
        steg_file: String,

        /// The output path to save the extracted hidden file (base name)
        #[clap(short = 'O', long, value_parser)]
        output_file: String,

        /// Type of the carrier file (steg_file)
        #[clap(short = 'C', long, value_enum)]
        carrier_type: CarrierType,
    },
}
