mod bitio;
mod huff;
mod utils;

use std::env;
use std::fs::{self,File};
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




    // --- DECOMPRESSION PATH ---
    // let mut input_bit_file = BitFile::open(input_path, true)
    //     .expect("Failed to open compressed input file");
    //
    // let output_file = File::create(output_path)
    //     .expect("Failed to create output file");
    //
    // println!("\nExpanding {} to {}", input_path, output_path);
    //
    // if let Err(e) = huff::expand_file(&mut input_bit_file, output_file, extra_args.len() as i32, extra_args) {
    //     eprintln!("Error during expansion: {}", e);
    //     process::exit(1);
    // }
    //
    // input_bit_file.close_input();

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
    // --- Statistics Calculation ---

    let input_size = fs::metadata(input_path).map(|m| m.len()).unwrap_or(0);
    let output_size = fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

    // Calculate the difference (savings)
    // Using saturating_sub to avoid errors if the output is somehow larger than input
    let savings = input_size.saturating_sub(output_size);

    // Calculate percentage
    let ratio = if input_size > 0 {
        (output_size as f64 / input_size as f64) * 100.0
    } else {
        0.0
    };

    println!("\nStatistics:");
    println!("----------------------------");
    println!("Original Size:   {} bytes", input_size);
    println!("Compressed Size: {} bytes", output_size);
    println!("Space Saved:     {} bytes", savings);
    println!("Compression %:   {:.2}% of original size", ratio);

    println!("\nOperation complete.");
}