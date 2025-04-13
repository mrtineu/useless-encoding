
use rand::prelude::*;
use rayon::prelude::*; // Rayon for parallelism
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::Path;
use std::io::SeekFrom;



fn split_chunks(file_path: &str) -> Vec<Vec<usize>>{
    let data_file = File::open(file_path).unwrap();

    let file_metadata = data_file.metadata().unwrap();
    let file_len = file_metadata.len() as usize;
    let num_cores = num_cpus::get();
    let base_chunk_size = file_len / num_cores;
    let remainder = file_len % num_cores;
    let mut result = Vec::with_capacity(num_cores);
    let mut current_pos = 0;
    for i in 0..num_cores {
        let current_chunk_size = base_chunk_size + if i < remainder { 1 } else { 0 };
        if current_chunk_size == 0 {
            continue;
        }
        let chunk_start = current_pos;
        let chunk_end = chunk_start + current_chunk_size;
        result.push(vec![chunk_start, chunk_end]);
        current_pos = chunk_end;
    }
    result
}


pub fn encode_file(file_path: &str, buffer_size: usize) {
    let chunks = split_chunks(file_path);
    let file_name = Path::new(file_path).file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let extension = Path::new(file_path)
        .extension()
        .map(|ext| ext.to_string_lossy().to_string())
        .unwrap_or_default();
    // Create temp file names first
    let temp_file_names: Vec<String> = (0..chunks.len())
        .map(|i| format!("{file_name}_chunk__{}.useless", i))
        .collect();

    // Process chunks in parallel
    chunks.par_iter().enumerate().for_each(|(i, chunk)| {
        let start = chunk[0];
        let end = chunk[1];
        let mut data_file = File::open(file_path).unwrap();
        let mut temp_file = BufWriter::new(
            OpenOptions::new()
            .create(true)
            .write(true)
            .open(&temp_file_names[i])
            .unwrap()
        );
        // Seek to the starting position

        data_file.seek(SeekFrom::Start(start as u64)).unwrap();

        let mut buffer = vec![0u8; buffer_size];
        let mut byte_position = start;

        while byte_position < end {
            let bytes_to_read = std::cmp::min(buffer_size, end - byte_position);
            data_file.read_exact(&mut buffer[..bytes_to_read]).unwrap();

            let bits = encode_buffer_to_bits(&buffer[..bytes_to_read]);
            let encoded = encode_to_useless(bits);
            temp_file.write_all(encoded.as_bytes()).unwrap();

            byte_position += bytes_to_read;
        }
    });

    merge_temp_files(&get_useless_output_name(file_path), &temp_file_names, &extension)
}

fn encode_to_useless(input: Vec<u8>) -> String {
    const NUM: u32 = u32::pow(2, 30);
    const BATCH_SIZE: usize = 1024;
    let mut result = String::with_capacity(input.len() * 10);
    let mut rng = rand::thread_rng();

    for chunk in input.chunks(BATCH_SIZE) {
        // Generate batch of random numbers
        let rand_nums: Vec<u32> = (0..chunk.len())
            .map(|_| rng.gen_range(1..NUM))
            .collect();

        for (i, &bit) in chunk.iter().enumerate() {
            let num = if bit == 0 {
                rand_nums[i] * 2
            } else {
                rand_nums[i] * 2 + 1
            };
            result.push_str(&num.to_string());
            result.push(' ');
        }
    }

    result
}

fn encode_buffer_to_bits(buffer: &[u8]) -> Vec<u8> {
    let mut bits = Vec::with_capacity(buffer.len() * 8);
    for &byte in buffer {
        // Use a lookup table or unrolled loop for better performance
        for i in (0..8).rev() {
            bits.push((byte >> i) & 1);
        }
    }
    bits
}

fn merge_temp_files(output_path: &str, temp_paths: &[String], extension: &str) {
    let mut output = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(output_path)
            .expect("Failed to create final output file"),
    );

    for path in temp_paths {
        let mut input = BufReader::new(
            File::open(path).expect(&format!("Failed to open temp file: {}", path)),
        );

        let mut buffer = [0u8; 64*1024]; // 8KB buffer
        loop {
            let bytes_read = input.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                break;
            }
            output.write_all(&buffer[..bytes_read]).unwrap();
        }
    }
    let ext_bytes = extension.as_bytes();
    let ext_bits = encode_buffer_to_bits(ext_bytes);
    let ext_len = extension.len() as u8;
    let ext_len_bits = encode_buffer_to_bits(&[ext_len]);
    // Encode both using the useless format
    let encoded_ext = encode_to_useless(ext_bits);
    let encoded_ext_len = encode_to_useless(ext_len_bits);

    // Write the encoded extension followed by its encoded length
    output.write_all(encoded_ext.as_bytes()).unwrap();
    output.write_all(encoded_ext_len.as_bytes()).unwrap();
    output.flush().unwrap();
    for path in temp_paths{
        std::fs::remove_file(path).unwrap();
    }
}

fn get_useless_output_name(original_path: &str) -> String {
    let original = Path::new(original_path);
    let stem = original.file_stem().unwrap_or_default().to_string_lossy();
    format!("{}.useless", stem)
}