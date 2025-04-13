use rayon::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::Instant;

pub fn decode_file(file_path: &str, buffer_size: usize) {
    // First extract the extension and determine content size
    let mut start = Instant::now();
    let (original_extension, content_size) = extract_extension(file_path);
    println!("The extension took: {:?}", start.elapsed());
    println!("Original extension: '{}', Content size: {} bytes", original_extension, content_size);

    // Split into chunks that respect number boundaries
    let mut start = Instant::now();
    let chunks = split_chunks(file_path, content_size);
    println!("The Splitting took: {:?}", start.elapsed());
    println!("Processing with {} chunks", chunks.len());

    // Prepare output and temp files
    let mut start = Instant::now();
    let output_path = get_decoded_output_name(file_path, &original_extension);
    let temp_file_names: Vec<String> = (0..chunks.len())
        .map(|i| format!("{}.tmp{}", file_path, i))
        .collect();
    println!("The temp files name and output name plitting took: {:?}", start.elapsed());
    // Process chunks in parallel
    chunks.par_iter().enumerate().for_each(|(i, chunk)| {
        let start = chunk[0];
        let end = chunk[1];
        println!("Processing chunk {} ({} - {})", i, start, end);
        process_chunk(file_path, start, end, &temp_file_names[i], buffer_size);
    });

    // Merge results and clean up
    merge_decoded_files(&output_path, &temp_file_names);
    println!("Decoding complete. Output: {}", output_path);
}

fn process_chunk(file_path: &str, start: usize, end: usize, output_path: &str, _: usize) {
    let mut file = File::open(file_path).unwrap();
    file.seek(SeekFrom::Start(start as u64)).unwrap();

    let mut output = File::create(output_path).unwrap();
    let mut reader = BufReader::new(file.take((end - start) as u64));
    let mut bit_buffer = Vec::with_capacity(8);

    let mut number_str = String::new();
    while reader.read_to_string(&mut number_str).unwrap() > 0 {
        for number in number_str.split_whitespace() {
            bit_buffer.push((number.parse::<u64>().unwrap() % 2) as u8);

            if bit_buffer.len() == 8 {
                output.write_all(&[bits_to_byte(&bit_buffer)]).unwrap();
                bit_buffer.clear();
            }
        }
        number_str.clear();
    }
}
fn bits_to_byte(bits: &[u8]) -> u8 {
    if bits.len() != 8 {
        eprintln!("Warning: expected 8 bits, got {}", bits.len());
        return 0;
    }
    bits.iter().enumerate().fold(0, |byte, (i, &bit)| byte | ((bit & 1) << (7 - i)))
}


fn split_chunks(file_path: &str, content_size: usize) -> Vec<Vec<usize>> {
    let num_cores = num_cpus::get().max(1);
    let chunk_size = content_size/num_cores;

    chunks
}
fn find_next_space(file_path: &str, start_pos: usize, max_pos: usize) -> usize {
    let mut file = File::open(file_path).expect("Failed to open file");
    if let Err(e) = file.seek(SeekFrom::Start(start_pos as u64)) {
        eprintln!("Seek failed: {}", e);
        return start_pos;
    }

    let mut buffer = [0u8; 1];
    let mut pos = start_pos;

    while pos < max_pos {
        match file.read_exact(&mut buffer) {
            Ok(_) => {
                if buffer[0] == b' ' {
                    return pos + 1; // Return position after the space
                }
                pos += 1;
            },
            Err(_) => break
        }
    }

    pos.min(max_pos)
}

fn extract_extension(file_path: &str) -> (String, usize) {
    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            return (String::new(), 0);
        }
    };
    
    let file_size = match file.metadata() {
        Ok(m) => m.len() as usize,
        Err(e) => {
            eprintln!("Failed to get file metadata: {}", e);
            return (String::new(), 0);
        }
    };
    
    // The last part of the file contains the extension information
    // Format: [content][extension bytes][extension length - 1 byte]
    
    // Read extension length (8 numbers at the end)
    let mut buffer = vec![0u8; 1024]; // Read a chunk that should contain the end
    let pos = if file_size > buffer.len() {
        file_size - buffer.len()
    } else {
        0
    };
    
    if let Err(e) = file.seek(SeekFrom::Start(pos as u64)) {
        eprintln!("Seek failed: {}", e);
        return (String::new(), file_size);
    }
    
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Read failed: {}", e);
            return (String::new(), file_size);
        }
    };
    
    if bytes_read == 0 {
        return (String::new(), file_size);
    }
    
    // Process the tail of the file to extract extension data
    let tail_str = String::from_utf8_lossy(&buffer[..bytes_read]);
    let numbers: Vec<&str> = tail_str.split_whitespace().collect();
    
    if numbers.len() < 8 {
        return (String::new(), file_size);
    }
    
    // Extract extension length from the last 8 numbers
    let mut ext_len_bits = Vec::with_capacity(8);
    for i in (numbers.len() - 8)..numbers.len() {
        if let Ok(num) = numbers[i].parse::<u64>() {
            ext_len_bits.push((num % 2) as u8);
        } else {
            return (String::new(), file_size);
        }
    }
    
    let ext_len = bits_to_byte(&ext_len_bits) as usize;
    if ext_len == 0 {
        return (String::new(), file_size - 8);
    }
    
    // Extract extension bytes
    let mut ext_bits = Vec::with_capacity(ext_len * 8);
    let end_index = numbers.len() - 8;
    let start_index = if end_index >= ext_len * 8 {
        end_index - ext_len * 8
    } else {
        return (String::new(), file_size - 8);
    };
    
    for i in start_index..end_index {
        if let Ok(num) = numbers[i].parse::<u64>() {
            ext_bits.push((num % 2) as u8);
        } else {
            return (String::new(), file_size - 8);
        }
    }
    
    // Convert bits to extension string
    let extension_bytes: Vec<u8> = ext_bits
        .chunks_exact(8)
        .map(bits_to_byte)
        .collect();
    
    let extension = match String::from_utf8(extension_bytes) {
        Ok(ext) => ext,
        Err(_) => return (String::new(), file_size - 8 - ext_len * 8),
    };
    
    // Content size is the file size minus the extension data
    let content_size = file_size - 8 - ext_len * 8;
    (extension, content_size)
}

fn merge_decoded_files(output_path: &str, temp_paths: &[String]) {
    let mut output = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(output_path)
            .expect("Failed to create output file")
    );

    for path in temp_paths {
        let mut input = BufReader::new(File::open(path).expect("Failed to open temp file"));
        let mut buffer = [0u8; 64 * 1024]; // 64KB buffer

        loop {
            let bytes_read = input.read(&mut buffer).expect("Read failed");
            if bytes_read == 0 {
                break;
            }
            output.write_all(&buffer[..bytes_read]).expect("Write failed");
        }
    }

    output.flush().expect("Flush failed");

    // Clean up temp files
    for path in temp_paths {
        std::fs::remove_file(path).unwrap_or_else(|_| eprintln!("Failed to remove temp file: {}", path));
    }
}

fn get_decoded_output_name(encoded_path: &str, extension: &str) -> String {
    let stem = Path::new(encoded_path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();

    if extension.is_empty() {
        format!("{}_decoded", stem)
    } else {
        format!("{}_decoded.{}", stem, extension)
    }
}


