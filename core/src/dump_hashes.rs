#![deny(clippy::unwrap_used, clippy::expect_used)]
use hex::encode;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

fn read_file_to_bytes<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn get_hash_size(algorithm: &str) -> usize {
    match algorithm.to_lowercase().as_str() {
        "md5" => 16,
        "sha256" => 32,
        "sha3_512" => 64,
        "scrypt" => 0,
        _ => panic!("Unsupported algorithm: {}", algorithm),
    }
}

pub fn dump_hashes(file_path: &str) -> io::Result<()> {
    let data = read_file_to_bytes(file_path)?;

    if data.len() < 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid file format: file too small",
        ));
    }

    let version = data[0];
    let algo_len = data[1] as usize;

    if data.len() < 2 + algo_len + 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid algorithm length",
        ));
    }

    let algorithm = String::from_utf8_lossy(&data[2..2 + algo_len]);
    let password_length = data[2 + algo_len] as usize;

    println!("VERSION: {}", version);
    println!("ALGORITHM: {}", algorithm);
    println!("PASSWORD LENGTH: {}", password_length);

    let mut offset = 3 + algo_len;

    if algorithm.to_lowercase() == "scrypt" {
        let hash_data = String::from_utf8_lossy(&data[offset..]);

        let hashes: Vec<&str> = hash_data.split("$scrypt").collect();

        for hash in hashes.iter().filter(|h| !h.is_empty()) {
            println!("$scrypt{}", hash);
        }
        return Ok(());
    }

    let hash_size = get_hash_size(&algorithm);
    while offset + hash_size <= data.len() {
        let hash_data = &data[offset..offset + hash_size];
        println!("{}", encode(hash_data));
        offset += hash_size;
    }

    if offset < data.len() {
        let remaining = &data[offset..];
        if !remaining.iter().all(|&b| b == 0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Trailing data detected after hashes",
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 || args[1] != "--in-file" {
        eprintln!("Usage: {} dump-hashes --in-file <path>", args[0]);
        std::process::exit(1);
    }

    if let Err(e) = dump_hashes(&args[2]) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
