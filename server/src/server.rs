use crate::ServerError;
use crate::cache_memory::Cache;
use crate::compute_threads::CrackLimiter;
use crate::crack_hashes::crack;
use crate::save_rainbow_table::upload;
use std::{result, sync::Arc};
// use std::sync::{Arc, Mutex};
use std::str;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    task,
};
/// Starts the TCP server that handles incoming client requests for uploading rainbow tables or cracking password hashes.
///
/// # Arguments
///
/// * `bind` - The IP address to bind the server to.
/// * `port` - The port number on which the server listens for incoming connections.
/// * `compute_threads` - Maximum number of concurrent password cracking threads.
/// * `_async_threads` - Placeholder for the number of async threads (not used in this synchronous implementation).
///
/// # Errors
///
/// Returns `ServerError` if the server fails to bind or handle connections.
pub async fn start_server(
    bind: String,
    port: u16,
    compute_threads: usize,
    cache_size: Option<u32>,
) -> Result<(), ServerError> {
    println!("Starting async server on {}:{}", bind, port);
    let listener = TcpListener::bind(format!("{}:{}", bind, port))
        .await
        .map_err(|_| ServerError::BindingError)?;

    let max_cache_size = match cache_size {
        Some(size) => size as usize,
        None => i32::MAX as usize,
    };

    let cache = Arc::new(Mutex::new(Cache::new_with_size(max_cache_size)));
    let limiter = Arc::new(CrackLimiter::new(compute_threads));

    loop {
        let (stream, _) = listener.accept().await.map_err(ServerError::IoError)?;
        let cache = Arc::clone(&cache);
        let limiter = Arc::clone(&limiter);

        task::spawn(async move {
            println!("Client request received");
            println!("-------------------------------------------------------------");
            match handle_client(stream, cache, limiter).await {
                Ok(_) => println!("Client handled successfully"),
                Err(e) => eprintln!("Error handling client: {}", e),
            }
            println!("-------------------------------------------------------------");
        });
    }
}
/// Handles a single client connection, routing to either `upload` or `crack` based on the magic word.
///
/// # Arguments
///
/// * `stream` - The TCP stream for the client connection.
/// * `cache` - Shared memory cache containing rainbow tables and cracked passwords.
/// * `limiter` - Thread limiter to control concurrent cracking operations.
///
/// # Errors
///
/// Returns `ServerError` if client handling fails.
async fn handle_client(
    mut stream: TcpStream,
    cache: Arc<Mutex<Cache>>,
    limiter: Arc<CrackLimiter>,
) -> Result<(), ServerError> {
    let mut magic_word = [0u8; 5];
    stream
        .read_exact(&mut magic_word)
        .await
        .map_err(ServerError::IoError)?;

    let mut magic_word_str = std::str::from_utf8(&magic_word).map_err(ServerError::Utf8Error)?;
    let mut full_buf = Vec::from(magic_word);

    if magic_word_str != "crack" {
        let mut sixth_byte = [0u8; 1];
        stream
            .read_exact(&mut sixth_byte)
            .await
            .map_err(ServerError::IoError)?;
        full_buf.push(sixth_byte[0]);
        magic_word_str = std::str::from_utf8(&full_buf).map_err(ServerError::Utf8Error)?;
    }

    extract_metadata(&mut stream, magic_word_str).await?;

    if magic_word_str == "upload" {
        let map_err = upload(&mut stream, Arc::clone(&cache))
            .await
            .map_err(|_| ServerError::CacheError);
        match map_err {
            Ok(response) => {
                stream
                    .write_all(response.as_bytes())
                    .await
                    .map_err(ServerError::IoError)?;
                stream.flush().await.map_err(ServerError::IoError)?;
            }
            Err(e) => return Err(e),
        }
    } else if magic_word_str == "crack" {
        let _ = limiter.acquire(); // still sync
        let result = crack(&mut stream, Arc::clone(&cache)).await;
        let _ = limiter.release();

        match result {
            Ok(response) => {
                let response_str = format!(
                    "Successfully Cracked Password\n{}",
                    response
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
                stream
                    .write_all(response_str.as_bytes())
                    .await
                    .map_err(ServerError::IoError)?;
                stream.flush().await.map_err(ServerError::IoError)?;
            }
            Err(e) => {
                stream
                    .write_all(format!("Error: {}", e).as_bytes())
                    .await
                    .map_err(ServerError::IoError)?;
            }
        }
    } else {
        return Err(ServerError::InvalidMagicWord);
    }

    Ok(())
}

/// Extracts protocol metadata such as version, name, and payload size from the stream.
///
/// # Arguments
///
/// * `stream` - The TCP stream to read metadata from.
/// * `magic_word_str` - The command identifier (either "upload" or "crack").
///
/// # Errors
///
/// Returns `ServerError` on I/O or parsing failure.
async fn extract_metadata(
    stream: &mut TcpStream,
    magic_word_str: &str,
) -> result::Result<(), ServerError> {
    let mut version = [0u8; 1];
    stream
        .read_exact(&mut version)
        .await
        .map_err(ServerError::IoError)?;

    if magic_word_str == "upload" {
        let mut name_len = [0u8; 1];
        stream
            .read_exact(&mut name_len)
            .await
            .map_err(ServerError::IoError)?;
        let mut name = vec![0u8; name_len[0] as usize];
        stream
            .read_exact(&mut name)
            .await
            .map_err(ServerError::IoError)?;
        let name = str::from_utf8(&name).map_err(ServerError::Utf8Error)?;
        println!("Name: {}", name);
    }

    let mut payload_size_bytes = [0u8; 8];
    stream
        .read_exact(&mut payload_size_bytes)
        .await
        .map_err(ServerError::IoError)?;
    let payload_size = u64::from_be_bytes(payload_size_bytes);

    println!("magic word: {:?}", magic_word_str);
    println!("Version: {}", version[0]);
    println!("Payload size: {}", payload_size);

    Ok(())
}
