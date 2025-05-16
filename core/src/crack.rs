use crate::hash::{HashAlgorithm, hash_with_algorithm};
use crate::reduction::reduce;
use hex::encode as hex_encode;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ChainEntry {
    pub start: String,
    pub end: String,
}

#[derive(Debug)]
pub struct RainbowTable {
    pub chains: Vec<ChainEntry>,
    pub algorithm: HashAlgorithm,
    pub password_len: usize,
    pub num_links: usize,
    pub charset: Vec<u8>,
    pub ascii_offset: u8,
}

fn read_exact_or_string(file: &mut File, buf: &mut [u8]) -> Result<(), String> {
    file.read_exact(buf).map_err(|e| e.to_string())
}

pub fn load_rainbow_table(path: &str) -> Result<RainbowTable, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open rainbow table: {}", e))?;

    let mut magic = [0u8; 12];
    read_exact_or_string(&mut file, &mut magic)?;
    if &magic != b"rainbowtable" {
        return Err("Invalid magic word in rainbow table.".to_string());
    }

    let mut version = [0u8; 1];
    read_exact_or_string(&mut file, &mut version)?;

    let mut algo_len = [0u8; 1];
    read_exact_or_string(&mut file, &mut algo_len)?;
    let algo_len = algo_len[0] as usize;

    let mut algo_buf = vec![0u8; algo_len];
    read_exact_or_string(&mut file, &mut algo_buf)?;
    let algorithm = match std::str::from_utf8(&algo_buf)
        .map_err(|e| e.to_string())?
        .to_lowercase()
        .as_str()
    {
        "md5" => HashAlgorithm::Md5,
        "sha256" => HashAlgorithm::Sha256,
        "sha3_512" => HashAlgorithm::Sha3_512,
        _ => return Err("Unsupported algorithm.".to_string()),
    };

    let mut pwd_len_buf = [0u8; 1];
    read_exact_or_string(&mut file, &mut pwd_len_buf)?;
    let password_len = pwd_len_buf[0] as usize;

    let mut charset_buf = [0u8; 16];
    read_exact_or_string(&mut file, &mut charset_buf)?;
    let _charset_size = u128::from_be_bytes(charset_buf);

    let mut links_buf = [0u8; 16];
    read_exact_or_string(&mut file, &mut links_buf)?;
    let num_links = u128::from_be_bytes(links_buf) as usize;

    let mut offset_buf = [0u8; 1];
    read_exact_or_string(&mut file, &mut offset_buf)?;
    let ascii_offset = offset_buf[0];

    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|e| e.to_string())?;

    let chain_size = password_len * 2;
    let mut chains = Vec::new();
    for chunk in data.chunks_exact(chain_size) {
        let (start, end) = chunk.split_at(password_len);
        chains.push(ChainEntry {
            start: String::from_utf8(start.to_vec()).map_err(|e| e.to_string())?,
            end: String::from_utf8(end.to_vec()).map_err(|e| e.to_string())?,
        });
    }

    let charset: Vec<u8> = (32..=126).collect();
    Ok(RainbowTable {
        chains,
        algorithm,
        password_len,
        num_links,
        charset,
        ascii_offset,
    })
}

pub fn load_hashes(path: &str, algorithm: &HashAlgorithm) -> Result<Vec<String>, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open hash file: {}", e))?;
    let mut header = [0u8; 2];
    read_exact_or_string(&mut file, &mut header)?;

    let algo_len = header[1] as usize;
    let mut skip = vec![0u8; algo_len + 1];
    read_exact_or_string(&mut file, &mut skip)?;

    let hash_len = match algorithm {
        HashAlgorithm::Md5 => 16,
        HashAlgorithm::Sha256 => 32,
        HashAlgorithm::Sha3_512 => 64,
        HashAlgorithm::Scrypt => 64,
    };
    println!("Hash length: {}", hash_len);
    println!("Algorithm: {:?}", algorithm);

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
    if buffer.len() % hash_len != 0 {
        return Err("Invalid hash file length.".to_string());
    }

    Ok(buffer.chunks_exact(hash_len).map(hex_encode).collect())
}

pub fn crack_passwords(
    rainbow_table: RainbowTable,
    hashes_to_crack: Vec<String>,
    threads: usize,
    out_path: Option<&str>,
) -> Result<(), String> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .map_err(|e| format!("Failed to build thread pool: {}", e))?;

    let hash_set = Arc::new(hashes_to_crack.clone());
    let found = Arc::new(Mutex::new(HashMap::new()));

    rainbow_table.chains.par_iter().for_each(|chain| {
        for i in (0..rainbow_table.num_links).rev() {
            let mut pwd = chain.end.clone();
            for _ in i..rainbow_table.num_links {
                let hashed = hash_with_algorithm(&pwd, &rainbow_table.algorithm);
                pwd = reduce(
                    &hex_encode(&hashed),
                    rainbow_table.password_len,
                    &rainbow_table.charset,
                    rainbow_table.ascii_offset,
                );
            }

            let mut candidate = chain.start.clone();
            for _ in 0..rainbow_table.num_links {
                let hashed = hash_with_algorithm(&candidate, &rainbow_table.algorithm);
                let hash_hex = hex_encode(&hashed);
                if hash_set.contains(&hash_hex) {
                    if let Ok(mut map) = found.lock() {
                        map.entry(hash_hex.clone()).or_insert(candidate.clone());
                    }
                }
                candidate = reduce(
                    &hash_hex,
                    rainbow_table.password_len,
                    &rainbow_table.charset,
                    rainbow_table.ascii_offset,
                );
            }
        }
    });

    let result = Arc::try_unwrap(found)
        .map_err(|_| "Could not unwrap Arc (still in use)".to_string())?
        .into_inner()
        .map_err(|_| "Mutex poisoned while collecting cracked passwords".to_string())?;

    if result.is_empty() {
        return Err("No passwords found.".to_string());
    }

    match out_path {
        Some(path) => {
            let mut file = File::create(path).map_err(|e| e.to_string())?;
            for hash in &hashes_to_crack {
                if let Some(pwd) = result.get(hash) {
                    writeln!(file, "{}\t{}", hash, pwd).map_err(|e| e.to_string())?;
                }
            }
        }
        None => {
            for hash in &hashes_to_crack {
                if let Some(pwd) = result.get(hash) {
                    println!("{}\t{}", hash, pwd);
                }
            }
        }
    }

    Ok(())
}
