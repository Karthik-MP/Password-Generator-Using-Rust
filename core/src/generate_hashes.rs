// #![deny(clippy::unwrap_used, clippy::expect_used)]
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    thread::{self, JoinHandle},
};

use crossbeam_channel::{Receiver, Sender};
use scrypt::{
    Scrypt,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use sha2::Sha256;
use sha3::{Digest, Sha3_512};

use crate::HashassinError;

/// Generates hashes for passwords read from an input file and writes the results to an output file.
/// The hashing process is parallelized using multiple threads, with the specified algorithm used
/// for hashing each password.
///
/// # Arguments
///
/// * `in_file` - The path to the input file containing passwords. Each password should be on a new line.
/// * `out_file` - The path to the output file where the hashes will be written.
/// * `num_threads` - The number of threads to be used for hashing the passwords.
/// * `algorithm` - The hashing algorithm to be used. Supported values are "md5", "sha256", "sha3_512", and "scrypt".
///
/// # Errors
///
/// If the input file cannot be opened, or if the specified number of threads is less than 1, an error message is printed.
pub fn generate_hashes(
    in_file: String,
    out_file: String,
    num_threads: usize,
    algorithm: String,
) -> Result<(), HashassinError> {
    if num_threads < 1 {
        return Err(HashassinError::InvalidThreadCount);
        // return;
    }

    println!("Generating Hashes");
    println!("Reading File: {}", in_file);

    let file = match File::open(&in_file) {
        Ok(f) => f,
        Err(e) => {
            return Err(HashassinError::FileOpen(format!(
                "Error opening input file {e:?}"
            )));
        }
    };

    let (tx_encrpyter, rx_encrpyter) = crossbeam_channel::unbounded();
    let (tx_printer, rx_printer) = crossbeam_channel::unbounded();
    let mut handles = generate_hash(
        num_threads as u32,
        rx_encrpyter,
        tx_printer.clone(),
        algorithm.clone(),
    );

    let reader = BufReader::new(file);

    handles.push(create_print_to_file_thread(out_file, rx_printer));

    // Spawn the thread to send passwords
    thread::spawn(move || {
        send_passwords(reader, tx_encrpyter, tx_printer, &algorithm);
    });

    // Wait for all threads to finish
    for handle in handles {
        match handle.join() {
            Ok(_) => (),
            Err(e) => {
                return Err(HashassinError::ThreadJoin(format!(
                    "Error Joining the threads method name: generate_hashas {e:?}"
                )));
            }
        }
    }

    Ok(())
}

/// Sends passwords from the input file to encryption threads. It also sends metadata on the first iteration
/// and manages the communication between threads.
///
/// # Arguments
///
/// * `reader` - A buffered reader that reads the passwords from the input file.
/// * `tx_encrpyter` - The sender channel that sends passwords to the encryption threads.
/// * `tx_printer` - The sender channel that sends metadata to the printer thread.
/// * `algorithm` - The hashing algorithm to be used, which will be included in the metadata.
fn send_passwords<T>(
    reader: BufReader<T>,
    tx_encrpyter: Sender<String>,
    tx_printer: Sender<Vec<u8>>,
    algorithm: &str,
) where
    T: std::io::Read,
{
    let mut first_iteration = true;
    for line in reader.lines() {
        match line {
            Ok(password) => {
                if first_iteration {
                    // Write metadata first (VERSION, ALGORITHM, PASSWORD LENGTH)
                    let mut metadata = vec![];
                    metadata.push(1); // VERSION: 1 byte (constant value 1)
                    metadata.push(algorithm.len() as u8); // ALGORITHM LENGTH
                    metadata.extend_from_slice(algorithm.to_lowercase().as_bytes()); // ALGORITHM string
                    metadata.push(password.len() as u8); // PASSWORD LENGTH (assume first line represents password length)
                    if let Err(e) = tx_printer.send(metadata) {
                        eprintln!("Failed to send metadata: {}", e);
                    }
                    first_iteration = false;
                }
                if let Err(e) = tx_encrpyter.send(password) {
                    eprintln!("Failed to send password: {}", e);
                }
            }
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }
}

/// Spawns multiple threads to process the passwords concurrently, hashing them using the specified algorithm.
///
/// # Arguments
///
/// * `num_threads` - The number of threads to be spawned for processing.
/// * `rx_encrpyter` - The receiver channel to receive passwords from the main thread.
/// * `tx_printer` - The sender channel to send hashed passwords to the printer thread.
/// * `algorithm` - The hashing algorithm to be used.
///
/// # Returns
///
/// A vector of thread handles that need to be joined after all threads have been spawned.
fn generate_hash(
    num_threads: u32,
    rx_encrpyter: Receiver<String>,
    tx_printer: Sender<Vec<u8>>,
    algorithm: String,
) -> Vec<JoinHandle<()>> {
    (0..num_threads)
        .map(|_| {
            let tx_printer = tx_printer.clone();
            let rx_encrpyter = rx_encrpyter.clone();
            let algorithm = algorithm.clone();
            thread::spawn(move || {
                for _ in 0..num_threads {
                    while let Ok(password) = rx_encrpyter.recv() {
                        let hashed_password: Vec<u8> = match algorithm.as_str() {
                            "md5" => generate_md5_hash(password),
                            "sha256" => generate_sha256_hash(password),
                            "sha3_512" => generate_sha3_512_hash(password),
                            "scrypt" => generate_scrypt_hash(password),
                            _ => {
                                eprintln!("Unknown algorithm: {}", algorithm);
                                return;
                            }
                        };

                        // Padding with 0 to the password
                        // let mut result_vec = vec![0]; // Start with a vector containing 0
                        // result_vec.extend_from_slice(&hashed_password);
                        // result_vec.push(0);
                        // send to printer thread
                        tx_printer.send(hashed_password).unwrap();
                    }
                }
            })
        })
        .collect::<Vec<_>>()
}

/// Generates an MD5 hash from the provided password string.
///
/// # Arguments
/// * `password` - A `String` containing the password to be hashed.
///
/// # Returns
/// A `Vec<u8>` representing the MD5 hash of the password.
///
/// # Example
/// ```rust
/// let password = String::from("my_secret_password");
/// let md5_hash = generate_md5_hash(password);
/// ```
/// # Note
/// MD5 is a cryptographic hash function that is considered broken and unsuitable for further use in security-sensitive applications.
fn generate_md5_hash(password: String) -> Vec<u8> {
    let hash = md5::compute(&password);
    // let hash_str = format!("{:x}", hash);
    // print!("hashed passowrd {} {:?} ", password, hash_str);
    hash.to_vec() // Convert to Vec<u8>
}

/// Generates a SHA-256 hash from the provided password string.
///
/// # Arguments
/// * `password` - A `String` containing the password to be hashed.
///
/// # Returns
/// A `Vec<u8>` representing the SHA-256 hash of the password.
///
/// # Example
/// ```rust
/// let password = String::from("my_secret_password");
/// let sha256_hash = generate_sha256_hash(password);
/// ```
/// # Note
/// SHA-256 is a member of the SHA-2 family and is considered a secure and commonly used hashing algorithm.
fn generate_sha256_hash(password: String) -> Vec<u8> {
    // let hash = sha256::digest(password);
    // hash.as_bytes().to_vec() // Convert to Vec<u8>

    let mut hasher = Sha256::new();
    hasher.update(password);
    hasher.finalize().to_vec() // Convert to Vec<u8>
}

/// Generates a SHA3-512 hash from the provided password string.
///
/// # Arguments
/// * `password` - A `String` containing the password to be hashed.
///
/// # Returns
/// A `Vec<u8>` representing the SHA3-512 hash of the password.
///
/// # Example
/// ```rust
/// let password = String::from("my_secret_password");
/// let sha3_512_hash = generate_sha3_512_hash(password);
/// ```
/// # Note
/// SHA3-512 is part of the SHA-3 family, which is a newer and more secure cryptographic hash function.
fn generate_sha3_512_hash(password: String) -> Vec<u8> {
    let mut hasher = Sha3_512::new();
    hasher.update(password.as_bytes());
    hasher.finalize().to_vec() // Convert to Vec<u8>
}

/// Generates an scrypt hash from the provided password string.
///
/// # Arguments
/// * `password` - A `String` containing the password to be hashed.
///
/// # Returns
/// A `Vec<u8>` representing the scrypt password hash in PHC string format.
///
/// # Example
/// ```rust
/// let password = String::from("my_secret_password");
/// let scrypt_hash = generate_scrypt_hash(password);
/// ```
/// # Note
/// Scrypt is a key derivation function designed to make brute-force attacks expensive. The resulting hash is in the PHC string format, starting with `$scrypt$`.
/// The `SaltString` and `Scrypt` are used to handle salt and key derivation.
fn generate_scrypt_hash(password: String) -> Vec<u8> {
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = Scrypt.hash_password(password.as_bytes(), &salt).unwrap();

    // Convert the password hash to bytes and return
    password_hash.to_string().into_bytes()
}

/// Creates a thread that writes hashed passwords to a file.
///
/// # Arguments
/// * `out_file` - A `String` representing the path to the output file where hashed passwords will be written.
/// * `rx_printer` - A `Receiver<Vec<u8>>` that receives hashed passwords to be written to the file.
///
/// # Returns
/// A `thread::JoinHandle<()>` which allows you to wait for the thread to finish its execution.
///
/// # Example
/// ```rust
/// let out_file = String::from("hashed_passwords.txt");
/// let (tx, rx) = mpsc::channel();
/// let handle = create_print_to_file_thread(out_file, rx);
/// tx.send(generate_sha256_hash(String::from("password1"))).unwrap();
/// ```
/// # Note
/// This function spawns a new thread that listens for `Vec<u8>` values and writes them to the specified file.
/// It uses a `Receiver` to receive the hashed passwords. Make sure to properly handle the file path and thread synchronization as needed.
fn create_print_to_file_thread(
    out_file: String,
    rx_printer: Receiver<Vec<u8>>, // Updated to Vec<u8>
) -> thread::JoinHandle<()> {
    let mut file = File::create(out_file).unwrap();
    // let mut first_iteration = true;
    thread::spawn(move || {
        while let Ok(hashed_password) = rx_printer.recv() {
            // if first_iteration {
            // file.write_all(b"0");
            file.write_all(&hashed_password).unwrap();
            // file.write_all(b"0");
            // continue;
            // }
            // file.write_all(&hashed_password).unwrap();
            // first_iteration = false
            // Write the cleaned bytes to the file
        }
    })
}
