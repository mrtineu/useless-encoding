use rand::prelude::*;
use std::fs::File;
use std::io::Read;
use std::io::Write;

fn main() {
    let mut data_file = File::open("/home/mrtineu/Documents/exported_datas.xml").unwrap();
    let mut file_content = String::new();
    data_file.read_to_string(&mut file_content).unwrap();
    let result=encode_to_useless(encode_to_bits(&file_content));
    let mut data_fil = File::create("data.txt").expect("creation failed");

    // Write contents to the file
    data_fil.write(result.as_bytes()).expect("write failed");


}

fn encode_to_bits(input: &str) -> Vec<u8> {
    input
        .bytes() // convert the string into bytes
        .flat_map(|byte| {
            (0..8).rev().map(move |i| (byte >> i) & 1) // extract each bit
        })
        .collect()
}

fn encode_to_useless(input:Vec<u8>) -> String {
    const NUM:u32 =u32::pow(2,30);
    let mut result="".to_string();
    let mut i =0;
    let mut rng = rand::thread_rng();
    while i < input.len() {
        if input[i] == 0 {
            result.push_str((rng.gen_range(1..NUM)*2).to_string().as_str());
            result.push_str(" ");
            i=i+1;
        }else if input[i] == 1 {
            result.push_str((rng.gen_range(1..NUM)*2+1).to_string().as_str());
            result.push_str(" ");
            i=i+1;
        }
    }
    return result;
}

