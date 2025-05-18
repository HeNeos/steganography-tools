mod decode;
mod encode;
mod input;
mod utils;

use clap::Parser;
use decode::to_image::lsb::{decode_lsb, reconstruct_image};
use encode::to_image::lsb::encode_lsb;
use input::read::{Args, Command};
use utils::load::load_image;

fn main() {
    let args = Args::parse();
    match args.command {
        Command::Encode {
            hide_file,
            steg_file,
            output_file,
        } => {
            encode_lsb(&hide_file, &steg_file, &output_file);
        }
        Command::Decode {
            steg_file,
            output_file,
        } => {
            let steg_image = load_image(&steg_file);
            let extracted_data = decode_lsb(&steg_image);
            reconstruct_image(&extracted_data, &output_file);
        }
    }
}
