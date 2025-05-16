#![deny(clippy::unwrap_used, clippy::expect_used)]
use std::{
    fs::File,
    io::Write,
    thread::{self, JoinHandle},
};

use crossbeam_channel::{Receiver, Sender};
use rand::Rng;

use crate::HashassinError;

/// Generates a specified number of random passwords with a given character length, distributed across multiple threads,
/// and writes them to either standard output or an output file.
///
/// # Arguments
///
/// * `chars` - The length of each generated password (in characters).
/// * `out_file` - The path to the output file where the passwords will be written. If set to "std", passwords are printed to standard output.
/// * `threads` - The number of threads to use for password generation.
/// * `num` - The total number of passwords to generate.
///
/// # Errors
///
/// If the number of threads is less than 1, an error message is printed and the function returns without generating any passwords.
pub fn generate_passwords(
    chars: u8,
    out_file: String,
    threads: usize,
    num: usize,
) -> Result<(), HashassinError> {
    if threads < 1 {
        return Err(HashassinError::InvalidThreadCount);
        // std::process::exit(1);
        // return;
    }

    let (tx_printer, rx_printer) = crossbeam_channel::unbounded();
    let mut num_per_threads = num;
    let mut new_thread_count = threads;
    if threads <= num {
        num_per_threads = num / threads;
    } else {
        new_thread_count = num
    }

    let mut handles: Vec<JoinHandle<()>> =
        create_gen_passwords_threads(chars, new_thread_count, tx_printer, num_per_threads)?;

    if out_file == "std" {
        match create_print_thread(rx_printer.clone()) {
            Ok(handle) => handles.push(handle),
            Err(e) => return Err(e), // If thread creation failed, return the error
        }
    } else {
        match create_print_to_file_thread(out_file, rx_printer.clone()) {
            Ok(handle) => handles.push(handle),
            Err(e) => return Err(e), // If thread creation failed, return the error
        }
    }

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

/// Creates and spawns multiple threads to generate passwords concurrently.
///
/// # Arguments
///
/// * `chars` - The length of each generated password (in characters).
/// * `new_thread_count` - The number of threads to spawn for password generation.
/// * `tx_printer` - The sender channel used to pass generated passwords to the printer threads.
/// * `num_per_thread` - The number of passwords to generate per thread.
///
/// # Returns
///
/// A vector of thread handles that can be joined to ensure all threads have completed.
fn create_gen_passwords_threads(
    chars: u8,
    new_thread_count: usize,
    tx_printer: Sender<String>,
    num_per_thread: usize,
) -> Result<Vec<JoinHandle<()>>, HashassinError> {
    let mut handles = Vec::new();

    for thread_id in 0..new_thread_count {
        match create_gen_password_thread(
            thread_id as u32,
            chars,
            tx_printer.clone(),
            num_per_thread,
        ) {
            Ok(handle) => handles.push(handle),
            Err(e) => {
                // If thread creation fails, return the error
                return Err(HashassinError::ThreadError(format!(
                    "Failed to create thread {}: {}",
                    thread_id, e
                )));
            }
        }
    }

    Ok(handles)
}

/// Creates a thread responsible for generating random passwords and sending them to the printer thread.
///
/// # Arguments
///
/// * `thread_id` - The ID of the thread (for logging purposes).
/// * `chars` - The length of each generated password (in characters).
/// * `tx_printer` - The sender channel used to pass generated passwords to the printer thread.
/// * `num_per_thread` - The number of passwords this thread will generate.
///
/// # Returns
///
/// A thread handle for the password generation thread.
fn create_gen_password_thread(
    thread_id: u32,
    chars: u8,
    tx_printer: Sender<String>,
    num_per_thread: usize,
) -> Result<JoinHandle<()>, HashassinError> {
    // Spawn the thread
    let handle = thread::spawn(move || {
        for _ in 0..num_per_thread {
            let random_string = generate_random_string(chars);
            // println!("Thread_id {} Random String: {}", thread_id, random_string);

            // Try sending the message to the printer thread
            match tx_printer.send(random_string.to_string()) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!(
                        "Thread_id {}: Error sending message to printer: {}",
                        thread_id,
                        HashassinError::SendError(e.to_string())
                    );
                    return; // Return early if sending fails (you can choose to handle this differently)
                }
            }
        }
    });

    // If the thread was spawned successfully, return the handle; otherwise, return an error
    Ok(handle)
}

/// Generates a random string of printable ASCII characters of a given length.
///
/// # Arguments
///
/// * `length` - The length of the string to generate.
///
/// # Returns
///
/// A random string of printable ASCII characters.
fn generate_random_string(length: u8) -> String {
    let mut rng = rand::rng();
    // This range covers all uppercase and lowercase letters, digits, punctuation marks, and spaces, which are all valid printable ASCII characters.

    let mut random_string = String::new();

    for _ in 0..length {
        let random_char = rng.random_range(32..=126) as u8 as char; // Generate a random ASCII character
        random_string.push(random_char); // Add the random character to the string
    }
    random_string
}

/// This thread listens for messages (generated passwords) and prints them to standard output.
///
/// # Arguments
///
/// * `rx_printer` - The receiver channel to receive the generated passwords.
fn create_print_thread(rx_printer: Receiver<String>) -> Result<JoinHandle<()>, HashassinError> {
    Ok(thread::spawn(move || {
        while let Ok(msg) = rx_printer.recv() {
            println!("{msg}");
        }
    }))
}

/// This thread listens for messages (generated passwords) and writes them to an output file.
///
/// # Arguments
///
/// * `out_file` - The path to the output file where the passwords will be written.
/// * `rx_printer` - The receiver channel to receive the generated passwords.
fn create_print_to_file_thread(
    out_file: String,
    rx_printer: Receiver<String>,
) -> Result<JoinHandle<()>, HashassinError> {
    let mut file = match File::create(out_file) {
        Ok(f) => f,
        Err(e) => {
            return Err(HashassinError::CreateFile(format!(
                "Error opening input file {e:?}"
            )));
        }
    };

    // let writer = BufWriter::new(file);
    Ok(thread::spawn(move || {
        // let mut counter = 0;
        while let Ok(msg) = rx_printer.recv() {
            // counter += 1;
            writeln!(&mut file, "{msg}").unwrap();
        }
    }))
}
