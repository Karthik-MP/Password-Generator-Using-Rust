use crate::ServerError;
use crate::cache_memory::{Cache, Chain};
use std::result;
use std::str;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Handles the `upload` command from a TCP client.
/// This function receives a rainbow table, parses its metadata and chains,
/// and inserts them into the server's in-memory cache.
///
/// # Arguments
///
/// * `stream` - A mutable reference to the client's TCP stream.
/// * `cache` - A shared reference to the server's cache.
///
/// # Returns
///
/// A `Result` containing a success message or a `ServerError`.
/// Async handler for the `upload` command.
pub(crate) async fn upload(
    stream: &mut TcpStream,
    cache: Arc<Mutex<Cache>>,
) -> result::Result<String, ServerError> {
    // Read the fixed-length magic word ("rainbowtable")
    let mut rainbow_magic_word = vec![0u8; 12];
    stream
        .read_exact(&mut rainbow_magic_word)
        .await
        .map_err(ServerError::IoError)?;
    let rainbow_magic_word_str =
        str::from_utf8(&rainbow_magic_word).map_err(ServerError::Utf8Error)?;

    // Read the payload version byte
    let mut payload_version = [0u8; 1];
    stream
        .read_exact(&mut payload_version)
        .await
        .map_err(ServerError::IoError)?;

    // Read the algorithm name length byte
    let mut algo_len = [0u8; 1];
    stream
        .read_exact(&mut algo_len)
        .await
        .map_err(ServerError::IoError)?;

    // Read the algorithm name as a UTF-8 string
    let mut algorithm = vec![0u8; algo_len[0] as usize];
    stream
        .read_exact(&mut algorithm)
        .await
        .map_err(ServerError::IoError)?;
    let algorithm = str::from_utf8(&algorithm)
        .map_err(ServerError::Utf8Error)?
        .to_string();

    // Read the password length byte
    let mut password_len = [0u8; 1];
    stream
        .read_exact(&mut password_len)
        .await
        .map_err(ServerError::IoError)?;

    // Read the character set size as a 128-bit unsigned integer
    let mut char_set_size_bytes = [0u8; 16];
    stream
        .read_exact(&mut char_set_size_bytes)
        .await
        .map_err(ServerError::IoError)?;
    let char_set_size = u128::from_be_bytes(char_set_size_bytes);

    // Read the number of links as a 128-bit unsigned integer
    let mut num_links_bytes = [0u8; 16];
    stream
        .read_exact(&mut num_links_bytes)
        .await
        .map_err(ServerError::IoError)?;
    let num_links = u128::from_be_bytes(num_links_bytes) as u32;

    // Read the ASCII offset byte
    let mut ascii_offset = [0u8; 1];
    stream
        .read_exact(&mut ascii_offset)
        .await
        .map_err(ServerError::IoError)?;
    let ascii_offset = ascii_offset[0];

    // Print the metadata for verification
    println!("Rainbow table magic word: {:?}", rainbow_magic_word_str);
    println!("Rainbow table version: {}", payload_version[0]);
    println!("Algorithm length: {}", algo_len[0]);
    println!("Algorithm: {}", algorithm);
    println!("Password length: {}", password_len[0]);
    println!("Character set size: {:?}", char_set_size);
    println!("Number of links: {:?}", num_links);
    println!("ASCII offset: {}", ascii_offset);

    // Prepare to read chains (start and end values for each chain)
    let chain_size = password_len[0] * 2;
    let mut chain_buf = vec![0u8; chain_size.into()];
    let mut num_inserted = 0;

    // Read and insert chains until the stream ends
    loop {
        match stream.read_exact(&mut chain_buf).await {
            Ok(_) => {
                let (start, end) = chain_buf.split_at(password_len[0] as usize);
                let start = String::from_utf8_lossy(start).to_string();
                let end = String::from_utf8_lossy(end).to_string();

                let my_chain = Chain::new(start, end);

                let cache_guard = cache.lock().await;
                cache_guard.insert_chain(&algorithm, password_len[0], num_links, my_chain);
                num_inserted += 1;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                println!("End of stream reached cleanly.");
                break;
            }
            Err(e) => {
                eprintln!("Error reading chain buffer: {}", e);
                return Err(ServerError::ChainError(e));
            }
        }
    }

    // Return success message
    Ok(format!(
        "Successfully uploaded {} chains for algorithm '{}'",
        num_inserted, algorithm
    ))
}
