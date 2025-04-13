mod encoding;
mod decoding;

use std::time::Instant;
use encoding::{encode_file};
use decoding::{decode_file};
fn main() {
    let mut start = Instant::now();
    encode_file("20250409_141145.jpg",1024*64);
    println!("The encoding took: {:?}", start.elapsed());
    let mut start = Instant::now();
    decode_file("20250409_141145.useless",1024*64);
    println!("The decoding took: {:?}", start.elapsed());


}





