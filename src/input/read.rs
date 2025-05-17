use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    Encode {
        /// The file to hide.
        #[clap(short = 'H', long, value_parser)]
        hide_file: String,

        /// The carrier image file (e.g., PNG, JPG).
        #[clap(short = 'S', long, value_parser)]
        steg_file: String,

        /// The output path for the steganography image.
        #[clap(short = 'O', long, value_parser)]
        output_file: String,
    },
    Decode {
        /// The steganography image file containing the hidden data.
        #[clap(short = 'S', long, value_parser)]
        steg_file: String,

        /// The output path to save the extracted hidden file.
        #[clap(short = 'O', long, value_parser)]
        output_file: String,
    },
}
