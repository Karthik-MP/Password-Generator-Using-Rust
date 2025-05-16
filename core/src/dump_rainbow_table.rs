use crate::HashassinError;
use std::fs::File;
use std::io::{BufReader, Read};

/// Dumps the contents of a rainbow table file.
///
/// This function reads the given rainbow table file, validates its structure, and prints the metadata
/// and chains contained within the file. It expects the file to adhere to a specific binary format
/// that includes a magic word, version, algorithm name, password length, character set size, number of
/// links, and an ASCII offset. After validating the header, it prints out the table's metadata followed
/// by each password chain (start and end points of each chain).
///
/// The format of the rainbow table is expected to be as follows:
/// - Magic word (`"rainbowtable"`, 12 bytes)
/// - Version (1 byte)
/// - Algorithm length (1 byte)
/// - Algorithm (variable length, UTF-8 encoded)
/// - Password length (1 byte)
/// - Character set size (16 bytes, u128)
/// - Number of links (16 bytes, u128)
/// - ASCII offset (1 byte)
/// - Password chains (variable length, each chain consists of start and end, each of length equal to the password length in bytes)
///
/// # Parameters
/// - `in_file`: The file path to the rainbow table file. It must be a valid path to an existing file.
///
/// # Returns
/// - `Ok(())` if the rainbow table file is read and processed successfully.
/// - `Err(HashassinError)` in case of any errors encountered during file reading, validation, or processing.
///   This could include:
///   - `InvalidInput` if the input file path is empty, or the file format is invalid.
///   - `FileOpen` if the file cannot be opened.
///   - `FileRead` if there is an error while reading from the file.
///   - `InvalidInput` if there is invalid UTF-8 data or an invalid chain size.
///
pub fn dump_rainbow_table(in_file: &str) -> Result<(), HashassinError> {
    if in_file.is_empty() {
        return Err(HashassinError::InvalidInput(
            "Input file path cannot be empty".to_string(),
        ));
    }

    let file = File::open(in_file).map_err(|e| HashassinError::FileOpen(e.to_string()))?;
    let mut reader = BufReader::new(file);

    let mut magic_word = vec![0u8; 12];
    reader
        .read_exact(&mut magic_word)
        .map_err(|e| HashassinError::FileRead(e.to_string()))?;

    if magic_word != b"rainbowtable" {
        return Err(HashassinError::InvalidInput(
            "Invalid file format: missing magic word".to_string(),
        ));
    }

    // Read version (1 byte)
    let mut version = [0u8; 1];
    reader
        .read_exact(&mut version)
        .map_err(|e| HashassinError::FileRead(e.to_string()))?;
    let version = version[0];

    // Read algorithm length (1 byte)
    let mut algorithm_len = [0u8; 1];
    reader
        .read_exact(&mut algorithm_len)
        .map_err(|e| HashassinError::FileRead(e.to_string()))?;
    let algorithm_len = algorithm_len[0] as usize;

    // Read algorithm (variable length)
    let mut algorithm = vec![0u8; algorithm_len];
    reader
        .read_exact(&mut algorithm)
        .map_err(|e| HashassinError::FileRead(e.to_string()))?;
    let algorithm = String::from_utf8(algorithm)
        .map_err(|_| HashassinError::InvalidInput("Invalid UTF-8 in algorithm name".to_string()))?;

    // Read password length (1 byte)
    let mut password_length = [0u8; 1];
    reader
        .read_exact(&mut password_length)
        .map_err(|e| HashassinError::FileRead(e.to_string()))?;
    let password_length = password_length[0];

    // Read character set size (16 bytes)
    let mut char_set_size_bytes = [0u8; 16];
    reader
        .read_exact(&mut char_set_size_bytes)
        .map_err(|e| HashassinError::FileRead(e.to_string()))?;
    let char_set_size = u128::from_be_bytes(char_set_size_bytes);

    // Read number of links (16 bytes)
    let mut num_links_bytes = [0u8; 16];
    reader
        .read_exact(&mut num_links_bytes)
        .map_err(|e| HashassinError::FileRead(e.to_string()))?;
    let num_links = u128::from_be_bytes(num_links_bytes);

    // Read ASCII offset (1 byte)
    let mut ascii_offset = [0u8; 1];
    reader
        .read_exact(&mut ascii_offset)
        .map_err(|e| HashassinError::FileRead(e.to_string()))?;
    let ascii_offset = ascii_offset[0];

    // Print metadata
    println!("Hashassin Rainbow Table");
    println!("VERSION: {}", version);
    println!("ALGORITHM: {}", algorithm);
    println!("PASSWORD LENGTH: {}", password_length);
    println!("CHAR SET SIZE: {}", char_set_size);
    println!("NUM LINKS: {}", num_links);
    println!("ASCII OFFSET: {}", ascii_offset);

    // Read and print chains (rest of the file)
    let mut buffer = Vec::new();
    reader
        .read_to_end(&mut buffer)
        .map_err(|e| HashassinError::FileOpen(e.to_string()))?;

    // Each chain is password_length * 2 bytes (start + end)
    let chain_size = (password_length as usize) * 2;
    for chunk in buffer.chunks(chain_size) {
        if chunk.len() != chain_size {
            return Err(HashassinError::InvalidInput(
                "Invalid chain size in file".to_string(),
            ));
        }

        let (start, end) = chunk.split_at(password_length as usize);
        println!(
            "{}\t{}",
            String::from_utf8_lossy(start),
            String::from_utf8_lossy(end)
        );
    }

    Ok(())
}
