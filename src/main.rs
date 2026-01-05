mod bitio;
mod huff;

use std::env;
use std::fs::File;
use std::process;
use bitio::BitFile;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("\nUsage: {} {}", args[0], huff::USAGE);
        process::exit(0);
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let extra_args = args[3..].to_vec();

    // Logic to determine if we are expanding. 
    // You can check if the program name contains 'expand' or just use a flag.
    let is_expanding = args[0].contains("expand");

    if is_expanding {
        // --- DECOMPRESSION PATH ---
        let mut input_bit_file = BitFile::open(input_path, true)
            .expect("Failed to open compressed input file");

        let output_file = File::create(output_path)
            .expect("Failed to create output file");

        println!("\nExpanding {} to {}", input_path, output_path);

        if let Err(e) = huff::expand_file(&mut input_bit_file, output_file, extra_args.len() as i32, extra_args) {
            eprintln!("Error during expansion: {}", e);
            process::exit(1);
        }

        input_bit_file.close_input();
    } else {
        // --- COMPRESSION PATH ---
        let input_file = File::open(input_path)
            .expect("Failed to open input file");

        let mut output_bit_file = BitFile::open(output_path, false)
            .expect("Failed to open output bit file");

        println!("\nCompressing {} to {}", input_path, output_path);

        if let Err(e) = huff::compress_file(input_file, &mut output_bit_file, extra_args.len() as i32, extra_args) {
            eprintln!("Error during compression: {}", e);
            process::exit(1);
        }

        output_bit_file.close_output().expect("Failed to flush remaining bits to disk");
    }

    println!("\nOperation complete.");
}