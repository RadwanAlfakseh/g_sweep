use std::path::Path;
use std::process;

pub fn usage_exit(prog_path: &str, usage: &str) {
    let prog_name = Path::new(prog_path)
        .file_stem()
        .and_then( |stem| stem.to_str())
        .unwrap_or(prog_path);

    println!("\nUsage: {} {}", prog_name, usage);
    process::exit(0);
}