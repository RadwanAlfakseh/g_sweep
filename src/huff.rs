use crate::bitio::BitFile;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};

pub const END_OF_STREAM: usize = 256;
pub const COMPRESSION_NAME: &str = "static order 0 model with Huffman coding";
pub const USAGE: &str = "infile outfile [-d]\n\nSpecifying -d will dump the modeling data\n";

#[derive(Copy, Clone, Debug, Default)]
pub struct Node {
    pub count: u32,
    pub saved_count: u32,
    pub child_0: i32,
    pub child_1: i32,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Code {
    pub code: u32,
    pub code_bits: i32,
}

// --- Header Logic ---

fn output_counts(bit_file: &mut BitFile, nodes: &[Node]) -> io::Result<()> {
    let mut first: usize = 0;
    while first < 255 && nodes[first].count == 0 {
        first += 1;
    }

    let mut f = first;
    while f < 256 {
        let mut last = f + 1;
        loop {
            while last < 256 && nodes[last].count != 0 {
                last += 1;
            }
            last -= 1;

            let mut next = last + 1;
            while next < 256 && nodes[next].count == 0 {
                next += 1;
            }
            if next > 255 || (next - last) > 3 {
                break;
            }
            last = next;
        }

        // Write first, last, and the counts in between
        // Accessing the internal file directly for byte-level header
        let internal_file = &mut bit_file.get_file();
        internal_file.write_all(&[f as u8, last as u8])?;
        for i in f..=last {
            internal_file.write_all(&[nodes[i].count as u8])?;
        }

        f = next_run_start(nodes, last + 1);
    }
    bit_file.get_file().write_all(&[0])?; // Terminal 0
    Ok(())
}

fn next_run_start(nodes: &[Node], start_index: usize) -> usize {
    let mut i = start_index;
    while i < 256 && nodes[i].count == 0 {
        i += 1;
    }
    i
}

// --- Tree Building ---

pub fn build_tree(nodes: &mut [Node]) -> i32 {
    nodes[513].count = 0xffff;
    let mut next_free = END_OF_STREAM + 1;

    loop {
        let mut min_1 = 513;
        let mut min_2 = 513;

        for i in 0..next_free {
            if nodes[i].count != 0 {
                if nodes[i].count < nodes[min_1].count {
                    min_2 = min_1;
                    min_1 = i;
                } else if nodes[i].count < nodes[min_2].count {
                    min_2 = i;
                }
            }
        }

        if min_2 == 513 { break; }

        nodes[next_free].count = nodes[min_1].count + nodes[min_2].count;
        nodes[min_1].saved_count = nodes[min_1].count;
        nodes[min_1].count = 0;
        nodes[min_2].saved_count = nodes[min_2].count;
        nodes[min_2].count = 0;
        nodes[next_free].child_0 = min_1 as i32;
        nodes[next_free].child_1 = min_2 as i32;
        next_free += 1;
    }

    (next_free - 1) as i32
}

pub fn convert_tree_to_code(
    nodes: &[Node],
    codes: &mut [Code],
    code_so_far: u32,
    bits: i32,
    node: i32
) {
    if node <= END_OF_STREAM as i32 {
        codes[node as usize].code = code_so_far;
        codes[node as usize].code_bits = bits;
        return;
    }
    let n = nodes[node as usize];
    convert_tree_to_code(nodes, codes, code_so_far << 1, bits + 1, n.child_0);
    convert_tree_to_code(nodes, codes, (code_so_far << 1) | 1, bits + 1, n.child_1);
}

/// High-level compression routine
pub fn compress_file(mut input: File, output: &mut BitFile, argc: i32, argv: Vec<String>) -> io::Result<()> {
    let mut counts = vec![0u64; 256];
    let mut nodes = vec![Node::default(); 514];
    let mut codes = vec![Code::default(); 257];

    // 1. Count bytes in the file
    count_bytes(&mut input, &mut counts)?;

    // 2. Scale counts down to 0-255 range for the header
    scale_counts(&counts, &mut nodes);

    // 3. Write the model (counts) to the compressed file header
    output_counts(output, &nodes)?;

    // 4. Build the Huffman tree and generate code table
    let root_node = build_tree(&mut nodes);
    convert_tree_to_code(&nodes, &mut codes, 0, 0, root_node);

    // 5. If -d is passed, print the model (Debug)
    if argc > 0 && argv[0] == "-d" {
        print_model(&nodes, &codes);
    }

    // 6. Perform the actual bit-wise compression
    compress_data(input, output, &codes)?;

    Ok(())
}

/// High-level expansion routine
pub fn expand_file(input: &mut BitFile, output: File, argc: i32, argv: Vec<String>) -> io::Result<()> {
    let mut nodes = vec![Node::default(); 514];

    // 1. Read the counts from the header and rebuild the tree
    input_counts(input, &mut nodes)?;
    let root_node = build_tree(&mut nodes);

    // 2. If -d is passed, print the model (Debug)
    if argc > 0 && argv[0] == "-d" {
        print_model(&nodes, &[]);
    }

    // 3. Decode bits back into bytes
    expand_data(input, output, &nodes, root_node)?;

    Ok(())
}
// --- Data Processing ---

pub fn compress_data(mut input: File, output: &mut BitFile, codes: &[Code]) -> io::Result<()> {
    let mut buffer = [0u8; 1];
    while input.read(&mut buffer)? > 0 {
        let c = buffer[0] as usize;
        output.output_bits(codes[c].code as u64, codes[c].code_bits as u32)?;
    }
    // Output EOS
    output.output_bits(
        codes[END_OF_STREAM].code as u64,
        codes[END_OF_STREAM].code_bits as u32
    )?;
    Ok(())
}

pub fn expand_data(input: &mut BitFile, mut output: File, nodes: &[Node], root_node: i32) -> io::Result<()> {
    loop {
        let mut node = root_node;
        while node > END_OF_STREAM as i32 {
            if input.input_bit()? != 0 {
                node = nodes[node as usize].child_1;
            } else {
                node = nodes[node as usize].child_0;
            }
        }

        if node == END_OF_STREAM as i32 {
            break;
        }

        output.write_all(&[node as u8])?;
    }
    Ok(())
}

// --- Helper Functions ---

fn count_bytes(input: &mut File, counts: &mut [u64]) -> io::Result<()> {
    let original_pos = input.stream_position()?;
    let mut buffer = Vec::new();
    input.read_to_end(&mut buffer)?;

    for &byte in &buffer {
        counts[byte as usize] += 1;
    }
    input.seek(SeekFrom::Start(original_pos))?;
    Ok(())
}

fn scale_counts(counts: &[u64], nodes: &mut [Node]) {
    let max_count = counts.iter().max().cloned().unwrap_or(0);

    // Scale factor: ensures max value fits in a byte (1-255)
    let scale = (max_count / 255) + 1;

    for i in 0..256 {
        nodes[i].count = (counts[i] / scale) as u32;
        if nodes[i].count == 0 && counts[i] != 0 {
            nodes[i].count = 1;
        }
    }
    nodes[END_OF_STREAM].count = 1;
}

fn input_counts(input: &mut BitFile, nodes: &mut [Node]) -> io::Result<()> {
    // Clear nodes
    for i in 0..256 { nodes[i].count = 0; }

    let mut buf = [0u8; 1];
    let file = input.get_file();

    // Read first byte
    file.read_exact(&mut buf)?;
    let mut first = buf[0] as usize;

    // Read last byte
    file.read_exact(&mut buf)?;
    let mut last = buf[0] as usize;

    loop {
        for i in first..=last {
            file.read_exact(&mut buf)?;
            nodes[i].count = buf[0] as u32;
        }

        file.read_exact(&mut buf)?;
        first = buf[0] as usize;
        if first == 0 { break; } // Terminal 0 found

        file.read_exact(&mut buf)?;
        last = buf[0] as usize;
    }
    nodes[END_OF_STREAM].count = 1;
    Ok(())
}

fn print_model(nodes: &[Node], codes: &[Code]) {
    for i in 0..513 {
        if nodes[i].saved_count != 0 {
            print!("node=");
            print_char(i as i32);
            print!(" count={:3} child_0=", nodes[i].saved_count);
            print_char(nodes[i].child_0);
            print!(" child_1=");
            print_char(nodes[i].child_1);

            if !codes.is_empty() && i <= END_OF_STREAM {
                print!(" Huffman code=");
                // Use the binary print utility from your bitio
                let _ = crate::bitio::file_print_binary_to_stdout(codes[i].code, codes[i].code_bits as u32);
            }
            println!();
        }
    }
}

fn print_char(c: i32) {
    if c >= 0x20 && c < 127 {
        print!("'{}'", c as u8 as char);
    } else {
        print!("{:3}", c);
    }
}