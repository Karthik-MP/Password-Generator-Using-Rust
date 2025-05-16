use crossbeam_channel::Receiver;

use crate::HashassinError;
use std::{fs::File, io::Write, thread};

/// Opens a file at the given path and returns a `File` handle.
///
/// This function attempts to open a file in read-only mode. If the file does not exist,
/// or if it cannot be opened (due to permissions or other I/O errors), it returns a
/// `HashassinError::FileOpen` with a descriptive message.
///
/// # Parameters
///
/// - `file_path`: A string slice representing the path to the file to open.
///
/// # Returns
///
/// - `Ok(File)` if the file was successfully opened.
/// - `Err(HashassinError)` if the file could not be opened.
///
/// # Errors
///
/// Returns `HashassinError::FileOpen` if:
/// - The file does not exist.
/// - The file cannot be read due to permission issues or other OS-level errors.
///
pub fn open_file(file_path: &str) -> Result<File, HashassinError> {
    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            return Err(HashassinError::FileOpen(format!(
                "Error opening input file: {e:?}"
            )));
        }
    };
    Ok(file)
}

/// Creates a thread that writes hashed passwords to a file.
///
/// # Arguments
/// * `out_file` - A `String` representing the path to the output file where hashed passwords will be written.
/// * `rx_printer` - A `Receiver<Vec<u8>>` that receives hashed passwords to be written to the file.
///
/// # Returns
/// A Result with`thread::JoinHandle<()>` and `HashassinError` which allows you to wait for the thread to finish its execution.
///
/// # Note
pub(crate) fn create_print_to_file_thread(
    out_file: String,
    rx_printer: Receiver<Vec<u8>>,
) -> Result<thread::JoinHandle<()>, HashassinError> {
    let file = File::create(out_file); // Propagate error if file creation fails
    let file = match file {
        Ok(f) => f,
        Err(e) => {
            return Err(HashassinError::FileOpen(format!(
                "Error creating output file: {e:?}"
            )));
        }
    };
    let handle = thread::spawn(move || {
        let mut file = file;
        while let Ok(data) = rx_printer.recv() {
            if let Err(e) = file.write_all(&data) {
                eprintln!("Failed to write to file: {}", e);
                break; // Stop the loop on write failure
            }
        }
    });

    Ok(handle)
}
