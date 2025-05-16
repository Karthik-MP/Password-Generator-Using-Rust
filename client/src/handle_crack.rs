use std::{
    fs::File,
    io::{self, BufWriter, Read, Write},
    net::TcpStream,
    path::Path,
};

/// Handles the cracking operation from the CLI by preparing the request,
/// sending it to the server, and handling the response.
///
/// # Arguments
///
/// * `server_addr` - The address of the server (e.g., "127.0.0.1:2025").
/// * `file_path` - Path to the file containing the hashes to crack.
/// * `out_file` - Optional path to save the cracked results; if not provided, prints to stdout.
///
/// # Errors
///
/// Returns `io::Result<()>` if reading, networking, or writing fails.
pub fn handle_crack(server_addr: &str, file_path: &str, out_file: Option<&str>) -> io::Result<()> {
    let hash_payload = read_file_payload(file_path)?;
    let message = build_crack_request(&hash_payload)?;
    let response = send_crack_request(server_addr, &message)?;
    write_crack_output(&response, out_file)?;
    Ok(())
}

/// Reads the entire content of the provided hashes file into memory as bytes.
///
/// # Arguments
///
/// * `file_path` - Path to the hashes file to read.
///
/// # Returns
///
/// Returns the file's contents as a `Vec<u8>`.
fn read_file_payload<P: AsRef<Path>>(file_path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Constructs a binary-formatted crack request according to the protocol.
///
/// # Arguments
///
/// * `payload` - The binary payload containing the hashes to crack.
///
/// # Returns
///
/// Returns the complete binary message as a `Vec<u8>`.
fn build_crack_request(payload: &[u8]) -> io::Result<Vec<u8>> {
    let mut message = Vec::new();

    // MAGIC WORD: "crack"
    message.extend_from_slice(b"crack");

    // VERSION: 1
    message.push(1);

    // PAYLOAD SIZE: 8-byte big-endian u64
    let payload_len = payload.len() as u64;
    message.extend_from_slice(&payload_len.to_be_bytes());

    // PAYLOAD: actual data
    message.extend_from_slice(payload);

    Ok(message)
}

/// Establishes a TCP connection with the server and sends the crack request.
///
/// # Arguments
///
/// * `server_addr` - The address of the server (e.g., "127.0.0.1:2025").
/// * `message` - The complete binary-formatted crack request to send.
///
/// # Returns
///
/// Returns the server's response as a `Vec<u8>`.
fn send_crack_request(server_addr: &str, message: &[u8]) -> io::Result<Vec<u8>> {
    let mut stream = TcpStream::connect(server_addr)?;
    stream.write_all(message)?;
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;
    Ok(response)
}

/// Writes the server's response to the specified output file or prints to stdout.
///
/// # Arguments
///
/// * `response` - The raw byte response received from the server.
/// * `out_file` - Optional path to save the response; prints to stdout if `None`.
///
/// # Errors
///
/// Returns `io::Result<()>` if writing to the file fails.
fn write_crack_output(response: &[u8], out_file: Option<&str>) -> io::Result<()> {
    match out_file {
        Some(path) => {
            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);
            writer.write_all(response)?;
        }
        None => {
            let output = String::from_utf8_lossy(response);
            println!("{output}");
        }
    }
    Ok(())
}
