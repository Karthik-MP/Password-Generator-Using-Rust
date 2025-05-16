use std::{
    fs::File,
    io::{self, Read, Write},
    net::TcpStream,
    path::Path,
};

/// Uploads a rainbow table to the specified server by constructing
/// a properly formatted message and sending it over a TCP connection.
///
/// # Arguments
///
/// * `server_addr` - The server address (e.g., "127.0.0.1:2025").
/// * `file_path` - Path to the rainbow table file to upload.
/// * `name` - A user-defined name associated with the rainbow table.
///
/// # Errors
///
/// Returns `io::Result<()>` if reading the file, building the message,
/// or communicating with the server fails.
pub fn handle_upload(server_addr: &str, file_path: &str, name: &str) -> io::Result<()> {
    let payload = read_file_payload(file_path)?;
    let message = build_upload_message(name, &payload)?;
    send_to_server(server_addr, &message)?;
    println!("Upload completed successfully.");
    Ok(())
}

/// Reads the entire contents of the provided file into memory as bytes.
///
/// # Arguments
///
/// * `file_path` - The path to the file to read.
///
/// # Returns
///
/// A `Vec<u8>` containing the file's raw bytes.
fn read_file_payload<P: AsRef<Path>>(file_path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Constructs a binary-formatted upload message with the required protocol structure.
///
/// # Arguments
///
/// * `name` - The name associated with the uploaded rainbow table.
/// * `payload` - The rainbow table file content in bytes.
///
/// # Returns
///
/// A `Vec<u8>` containing the complete message to send to the server.
///
/// # Errors
///
/// Returns `io::Error` if the name is too long to fit in a single byte length field.
fn build_upload_message(name: &str, payload: &[u8]) -> io::Result<Vec<u8>> {
    let mut message = Vec::new();

    // MAGIC WORD: "upload"
    message.extend_from_slice(b"upload");

    // VERSION: 1
    message.push(1);

    // NAME LENGTH and NAME
    let name_bytes = name.as_bytes();
    message.push(name_bytes.len() as u8); // Adds name length
    message.extend_from_slice(name_bytes); // Adds name itself

    // PAYLOAD SIZE (u64 big-endian)
    let payload_len = payload.len() as u64;
    message.extend_from_slice(&payload_len.to_be_bytes());

    // PAYLOAD: actual table data
    message.extend_from_slice(payload);

    Ok(message)
}

/// Connects to the specified server and sends the prepared upload message.
///
/// # Arguments
///
/// * `server_addr` - The server address (e.g., "127.0.0.1:2025").
/// * `message` - The complete upload message to send.
///
/// # Errors
///
/// Returns `io::Result<()>` if sending or receiving the server response fails.
fn send_to_server(server_addr: &str, message: &[u8]) -> io::Result<()> {
    let mut stream = TcpStream::connect(server_addr)?;

    stream.write_all(message)?; // Send the message
    stream.shutdown(std::net::Shutdown::Write)?; // Indicate end of transmission

    let mut response = Vec::new();
    stream.read_to_end(&mut response)?; // Read server response

    // Attempt to print the response as UTF-8 text
    if let Ok(response_str) = String::from_utf8(response) {
        println!("Response from server: {}", response_str);
    } else {
        eprintln!("Received non-UTF-8 response from server");
    }

    Ok(())
}
