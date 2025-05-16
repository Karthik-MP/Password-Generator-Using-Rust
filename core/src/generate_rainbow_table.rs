use crate::{
    HashassinError, algorithms,
    radix_type::Radix,
    utils::{self, create_print_to_file_thread},
};
use crossbeam_channel::{Receiver, Sender};
use log::{error, info};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    thread::{self, JoinHandle},
}; // Logging

/// Generates a rainbow table and writes it to a file.
///
/// A rainbow table is a precomputed table used for reversing cryptographic hash functions,
/// primarily for cracking password hashes. This function reads a list of plaintext values
/// from `in_file`, processes them in parallel using `num_threads`, and generates
/// `num_links` hash chains using the specified `algorithm`. The resulting rainbow table is
/// written to `out_file`.
///
/// # Parameters
///
/// - `num_links`: The number of links (hash-reduction pairs) to generate per chain.
/// - `num_threads`: The number of threads to use for parallel processing.
/// - `out_file`: The path to the output file where the rainbow table will be written.
/// - `algorithm`: The name of the hash algorithm to use (e.g., "sha256").
/// - `in_file`: The path to the input file containing plaintext values to seed the chains.
///
/// # Returns
///
/// Returns `Ok(())` if the table was successfully generated, or a `HashassinError` on failure.
///
/// # Errors
///
/// This function returns a `HashassinError` if there is an issue with reading the input file,
/// writing the output, using the specified algorithm, or during the table generation process.
pub fn generate_rainbow_table(
    num_links: usize,
    num_threads: usize,
    out_file: String,
    algorithm: String,
    in_file: String,
) -> Result<(), HashassinError> {
    info!("Starting rainbow table generation...");

    match validate_inputs(num_links, num_threads, &out_file, &algorithm, &in_file) {
        Ok(_) => {
            // Proceed with the generation of the rainbow table
            let file = utils::open_file(&in_file)?;
            let reader = BufReader::new(file);

            let (tx_password, rx_password) = crossbeam_channel::unbounded();
            let (tx_printer, rx_printer) = crossbeam_channel::unbounded();

            let mut handles = generate_rainbow_chain(
                num_links as u32,
                num_threads as u32,
                rx_password,
                tx_printer.clone(),
                algorithm.clone(),
            )?;

            handles.push(create_print_to_file_thread(out_file, rx_printer)?);

            read_passwords(num_links, reader, tx_password, tx_printer, &algorithm);

            for handle in handles {
                match handle.join() {
                    Ok(_) => info!("Thread joined successfully"),
                    Err(e) => {
                        return Err(HashassinError::ThreadJoin(format!(
                            "Error Joining the threads: {:?}",
                            e
                        )));
                    }
                }
            }

            info!("Rainbow table generation completed.");
            Ok(())
        }
        Err(e) => {
            error!("Input validation failed: {:?}", e);
            Err(e) // propagate error here
        }
    }
}

/// Validates the input parameters before generating a rainbow table.
///
/// This function checks whether the provided arguments are valid, such as ensuring that
/// the number of links and threads are positive, the specified files are accessible or writable,
/// and that the given algorithm is supported.
///
/// # Parameters
///
/// - `num_links`: The number of hash-reduction links to be generated. Must be greater than zero.
/// - `threads`: The number of threads to use. Must be greater than zero.
/// - `out_file`: Path to the file where the rainbow table will be written. Must be a valid writable path.
/// - `algorithm`: The name of the hash algorithm to use. Must be one of the supported algorithms.
/// - `in_file`: Path to the input file containing plaintexts. Must exist and be readable.
///
/// # Returns
///
/// Returns `Ok(())` if all inputs are valid, or a `HashassinError` describing the issue.
///
/// # Errors
///
/// Returns a `HashassinError` if:
/// - `num_links` or `threads` are zero.
/// - `out_file` cannot be created or written to.
/// - `in_file` does not exist or is not readable.
/// - `algorithm` is not among the supported list.
///
fn validate_inputs(
    num_links: usize,
    threads: usize,
    out_file: &str,
    algorithm: &str,
    in_file: &str,
) -> Result<(), HashassinError> {
    if num_links == 0 {
        return Err(HashassinError::InvalidInput(
            "Number of links must be greater than 0".to_string(),
        ));
    }
    if threads == 0 {
        return Err(HashassinError::InvalidThreadCount);
    }
    if out_file.is_empty() {
        return Err(HashassinError::InvalidInput(
            "Output file path cannot be empty".to_string(),
        ));
    }
    if algorithm.is_empty() {
        return Err(HashassinError::InvalidInput(
            "Algorithm cannot be empty".to_string(),
        ));
    }
    if in_file.is_empty() {
        return Err(HashassinError::InvalidInput(
            "Input file path cannot be empty".to_string(),
        ));
    }
    Ok(())
}

/// Reads passwords from an input source and sends them for processing.
///
/// This function reads plaintext passwords line-by-line from the given buffered reader.
/// For each password, it generates `num_links` hash-reduction chains using the specified
/// hash algorithm and sends the original password and resulting data through the provided
/// channels.
///
/// # Parameters
///
/// - `num_links`: The number of hash-reduction links to generate for each password.
/// - `reader`: A buffered reader over the input file containing plaintext passwords.
/// - `tx_password`: A sending channel used to transmit original plaintext passwords for further processing.
/// - `tx_printer`: A sending channel used to transmit the final byte representation of processed chains for output.
/// - `algorithm`: The hash algorithm to use (e.g., "sha256").
fn read_passwords(
    num_links: usize,
    reader: BufReader<File>,
    tx_password: Sender<String>,
    tx_printer: Sender<Vec<u8>>,
    algorithm: &str,
) {
    let mut first_iteration = true;
    for line in reader.lines() {
        match line {
            Ok(password) => {
                if first_iteration {
                    // Write metadata first (MAGIC WORD, VERSION, ALGORITHM LENGTH, ALGORITHM, PASSWORD LENGTH, CHARACTER SET SIZE, NUMBER OF LINKS, ASCII OFFSET)
                    let mut metadata: Vec<u8> = vec![];
                    // MAGIC WORD: UTF-8 "rainbowtable"
                    metadata.extend_from_slice(b"rainbowtable");
                    // VERSION: 1 byte (value 1)
                    metadata.push(1);
                    // ALGORITHM LENGTH: length of the algorithm string
                    let algo_lower = algorithm.to_lowercase();
                    metadata.push(algo_lower.len() as u8);
                    // ALGORITHM: the algorithm string in lowercase, no null terminator
                    metadata.extend_from_slice(algo_lower.as_bytes());
                    // PASSWORD LENGTH: length of the password
                    metadata.push(password.len() as u8);
                    // CHARACTER SET SIZE: 16 bytes, big-endian with leading zeros
                    let charset_size: u8 = 95;
                    let charset_bytes = charset_size.to_be_bytes(); // 2 bytes
                    let charset_padding = vec![0u8; 16 - charset_bytes.len()];
                    metadata.extend_from_slice(&charset_padding); // pad first
                    metadata.extend_from_slice(&charset_bytes); // then actual value
                    // NUMBER OF LINKS: 16 bytes, little-endian with leading zeros
                    let num_links_bytes = num_links.to_be_bytes(); // 8 bytes
                    let link_padding = vec![0u8; 16 - num_links_bytes.len()];
                    metadata.extend_from_slice(&link_padding); // pad first
                    metadata.extend_from_slice(&num_links_bytes); // then actual value
                    // ASCII OFFSET: 1 byte
                    metadata.push(32);

                    if let Err(e) = tx_printer.send(metadata) {
                        error!("Failed to send metadata: {}", e);
                    }
                    first_iteration = false;
                }
                if let Err(e) = tx_password.send(password) {
                    error!("Failed to send password: {}", e);
                }
            }
            Err(e) => error!("Error reading line: {}", e),
        }
    }
}

/// Spawns threads to generate rainbow chains in parallel.
///
/// This function creates `num_threads` worker threads, each consuming plaintext passwords
/// from the `rx_encrpyter` channel. Each thread processes passwords by applying a hash-reduction
/// chain of `num_links` iterations using the specified `algorithm`, then sends the final
/// result through the `tx_printer` channel for output or storage.
///
/// # Parameters
///
/// - `num_links`: The number of hash-reduction iterations (links) per chain.
/// - `num_threads`: The number of threads to spawn for parallel chain generation.
/// - `rx_encrpyter`: A channel receiver that provides plaintext passwords to be processed.
/// - `tx_printer`: A channel sender that receives the final result (e.g., chain endpoint or serialized data).
/// - `algorithm`: The hash algorithm to use (e.g., "sha256", "md5").
///
/// # Returns
///
/// Returns a `Result` containing a `Vec` of thread `JoinHandle<()>` objects if successful,
/// or a `HashassinError` if thread spawning or setup fails.
///
/// # Errors
///
/// Returns a `HashassinError`
fn generate_rainbow_chain(
    num_links: u32,
    num_threads: u32,
    rx_encrpyter: Receiver<String>,
    tx_printer: Sender<Vec<u8>>,
    algorithm: String,
) -> Result<Vec<JoinHandle<()>>, HashassinError> {
    let result = (0..num_threads)
        .map(|_| {
            let tx_printer = tx_printer.clone();
            let rx_encrpyter = rx_encrpyter.clone();
            let algorithm_clone = algorithm.clone();
            thread::spawn(move || {
                while let Ok(password) = rx_encrpyter.recv() {
                    let result = match algorithm_clone.as_str() {
                        "md5" => {
                            create_chain(password.clone(), num_links, algorithms::generate_md5_hash)
                        }
                        "sha256" => create_chain(
                            password.clone(),
                            num_links,
                            algorithms::generate_sha256_hash,
                        ),
                        "sha3_512" => create_chain(
                            password.clone(),
                            num_links,
                            algorithms::generate_sha3_512_hash,
                        ),
                        "scrypt" => create_chain(
                            password.clone(),
                            num_links,
                            algorithms::generate_scrypt_hash,
                        ),
                        _ => Err(HashassinError::UnknownAlgorithm(
                            algorithm_clone.to_string(),
                        )),
                    };

                    match result {
                        Ok(hashed_password) => {
                            let mut concatenated = password.clone().into_bytes(); // Convert String to Vec<u8> 
                            concatenated.extend_from_slice(&hashed_password);

                            if let Err(e) = tx_printer.send(concatenated) {
                                error!("Failed to send hashed password: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Error generating rainbow chain: {:?}", e);
                        }
                    }
                }
            })
        })
        .collect::<Vec<_>>();

    Ok(result)
}

/// Creates a rainbow chain from a given password using a specified hash function.
///
/// This function takes an initial `password` and applies a hash-reduction process
/// for `num_links` iterations. A custom hash function is provided as `hash_func`,
/// which is applied repeatedly to simulate a rainbow chain. The final result is a
/// serialized representation of the chain endpoint or intermediate data.
///
/// # Parameters
///
/// - `password`: The starting plaintext string for the rainbow chain.
/// - `num_links`: The number of hash-reduction steps to perform in the chain.
/// - `hash_func`: A function or closure that performs the hash-reduction operation.
///   It must implement `Fn(&str) -> Result<String, HashassinError>`.
fn create_chain<F>(
    mut password: String,
    num_links: u32,
    hash_func: F,
) -> Result<Vec<u8>, HashassinError>
where
    F: Fn(String) -> Vec<u8>,
{
    let radix = Radix::new(95); // 95 printable ASCII characters 
    for round in 0..num_links {
        let hash = hash_func(password.clone());
        let reduced =
            algorithms::reduction_function(hash, round as u128, password.len() as u32, &radix);
        password = reduced;
    }
    Ok(password.into_bytes())
}
