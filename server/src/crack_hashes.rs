use crate::ServerError;
use crate::cache_memory::{Cache, Chain, CrackedPassword};
use hashassin_core::hash::{HashAlgorithm, hash_with_algorithm};
use hashassin_core::reduction::reduce;
use hex::encode as hex_encode;
use std::collections::HashMap;
use std::result;
use std::str;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Handles the `crack` command from a TCP client, processing incoming data,
/// checking the cache for previously cracked passwords, and using the rainbow table if needed.
///
/// # Arguments
///
/// * `stream` - A mutable reference to the client's TCP stream.
/// * `cache` - A shared reference to the server's cache.
///
/// # Returns
///
/// A `Result` containing a map of cracked hashes to their corresponding passwords or a `ServerError`.
/// Async handler for the `crack` command.
pub(crate) async fn crack(
    stream: &mut TcpStream,
    cache: Arc<Mutex<Cache>>,
) -> result::Result<HashMap<String, String>, ServerError> {
    let mut hash_version = [0u8; 1];
    stream
        .read_exact(&mut hash_version)
        .await
        .map_err(ServerError::IoError)?;

    let mut algo_len = [0u8; 1];
    stream
        .read_exact(&mut algo_len)
        .await
        .map_err(ServerError::IoError)?;

    let mut algorithm = vec![0u8; algo_len[0] as usize];
    stream
        .read_exact(&mut algorithm)
        .await
        .map_err(ServerError::IoError)?;
    let algorithm_str = str::from_utf8(&algorithm)
        .map_err(ServerError::Utf8Error)?
        .to_lowercase();

    let algorithm = match algorithm_str.as_str() {
        "md5" => HashAlgorithm::Md5,
        "sha256" => HashAlgorithm::Sha256,
        "sha3_512" => HashAlgorithm::Sha3_512,
        _ => return Err(ServerError::InvalidAlgorithm),
    };

    let mut password_len = [0u8; 1];
    stream
        .read_exact(&mut password_len)
        .await
        .map_err(ServerError::IoError)?;

    let hash_len = match algorithm {
        HashAlgorithm::Md5 => 16,
        HashAlgorithm::Sha256 => 32,
        HashAlgorithm::Sha3_512 | HashAlgorithm::Scrypt => 64,
    };

    let mut buffer = Vec::new();
    stream
        .read_to_end(&mut buffer)
        .await
        .map_err(ServerError::IoError)?;

    let hashes: Vec<String> = buffer.chunks_exact(hash_len).map(hex_encode).collect();

    let cracked_password: Option<HashMap<String, String>> = {
        let cache_guard = cache.lock().await;

        let mut result = HashMap::new();
        for hash in &hashes {
            if let Ok(cracked) = cache_guard.get_cracked_password(&algorithm_str, hash) {
                result.insert(hash.clone(), cracked.password);
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    };

    match cracked_password {
        Some(cracked) => Ok(cracked),
        None => {
            let chains = {
                let cache_guard = cache.lock().await;
                cache_guard.get_all_chains(&algorithm_str, password_len[0])
            }?;

            let charset: Vec<u8> = (32..=126).collect();

            let cracked_passwords = crack_passwords(
                chains,
                hashes,
                algorithm.clone(),
                password_len[0],
                charset,
                0,
            )?;

            let cache_guard = cache.lock().await;
            for (hash, password) in cracked_passwords.iter() {
                let cracked_password = CrackedPassword::new(hash.to_string(), password.to_string());
                cache_guard.insert_cracked_password(&algorithm_str, cracked_password);
            }

            Ok(cracked_passwords)
        }
    }
}

/// Cracks hashes using provided rainbow table chains by simulating forward and backward reductions.
///
/// # Arguments
///
/// * `rainbow_table_chains` - The chains from the rainbow table.
/// * `hashes_to_crack` - The list of hashes to crack.
/// * `algorithm` - The hashing algorithm to use.
/// * `password_len` - The expected password length.
/// * `charset` - The charset to use in reduction.
/// * `ascii_offset` - The ASCII offset used in reduction.
///
/// # Returns
///
/// A `Result` containing a map of cracked hashes to passwords or a `ServerError`.
pub(crate) fn crack_passwords(
    rainbow_table_chains: HashMap<u32, Vec<Chain>>,
    hashes_to_crack: Vec<String>,
    algorithm: HashAlgorithm,
    password_len: u8,
    charset: Vec<u8>,
    ascii_offset: u8,
) -> Result<HashMap<String, String>, ServerError> {
    let hash_set: std::collections::HashSet<_> = hashes_to_crack.iter().cloned().collect();
    let mut found = HashMap::new();

    for (num_links, chains) in rainbow_table_chains {
        for chain in chains {
            // Reverse simulation from end of chain
            let mut pwd = chain.end_chain.clone();
            for _ in (0..num_links).rev() {
                let hashed = hash_with_algorithm(&pwd, &algorithm);
                pwd = reduce(
                    &hex_encode(&hashed),
                    password_len as usize,
                    &charset,
                    ascii_offset,
                );
            }

            // Forward simulation from start of chain
            let mut candidate = chain.start_chain.clone();
            for _ in 0..num_links {
                let hashed = hash_with_algorithm(&candidate, &algorithm);
                let hash_hex = hex_encode(&hashed);

                if hash_set.contains(&hash_hex) {
                    found.entry(hash_hex.clone()).or_insert(candidate.clone());
                    break;
                }

                candidate = reduce(&hash_hex, password_len as usize, &charset, ascii_offset);
            }
        }
    }

    if found.is_empty() {
        Err(ServerError::NoPasswordsFound)
    } else {
        Ok(found)
    }
}
